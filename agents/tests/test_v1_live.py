"""Live /v1 SSE flow: real EdgeQuake (MPW workspace, read-only) + real LLM, with a
real *user* token (bootstrap admin login + switch-scope), not the API key.

Skipped unless EDGEQUAKE_API_KEY is set (agents/.env) and the repo-root .env has the
bootstrap admin credentials. ~6 LLM calls.   uv run pytest -m live -s tests/test_v1_live.py
"""

import json
import os
from pathlib import Path

import httpx
import pytest
from dotenv import dotenv_values
from fastapi import FastAPI

import edgequake_agents.common.config  # noqa: F401  (loads agents/.env)

ROOT_ENV = dotenv_values(Path(__file__).resolve().parents[2] / ".env")
TENANT, WORKSPACE = "f62e6a67-9a75-4654-b2b6-1a6168bda5ba", "55a78878-845c-4abe-a8ba-8ceb9d067b90"
pytestmark = [
    pytest.mark.live,
    pytest.mark.skipif(not os.getenv("EDGEQUAKE_API_KEY") or not ROOT_ENV.get("EDGEQUAKE_BOOTSTRAP_ADMIN_PASSWORD"),
                       reason="EDGEQUAKE_API_KEY / bootstrap admin credentials not set"),
]


def user_headers() -> dict:
    url = os.getenv("EDGEQUAKE_URL", "http://localhost:8090")
    with httpx.Client(base_url=url, timeout=30) as c:
        login = c.post("/api/v1/auth/login", json={"username": ROOT_ENV["EDGEQUAKE_BOOTSTRAP_ADMIN_USERNAME"],
                                                   "password": ROOT_ENV["EDGEQUAKE_BOOTSTRAP_ADMIN_PASSWORD"]})
        login.raise_for_status()
        body = login.json()
        scoped = c.post("/api/v1/auth/switch-scope", headers={"Authorization": f"Bearer {body['access_token']}"},
                        json={"tenant_id": TENANT, "workspace_id": WORKSPACE, "refresh_token": body["refresh_token"]})
        token = scoped.json()["access_token"] if scoped.status_code == 200 else body["access_token"]
    return {"Authorization": f"Bearer {token}", "X-Tenant-ID": TENANT, "X-Workspace-ID": WORKSPACE,
            "X-EQ-User-Id": body["user"]["user_id"], "X-Agents-Internal-Token": "live-secret"}


@pytest.fixture
async def client(monkeypatch, tmp_path):
    from edgequake_agents.common import v1

    monkeypatch.setenv("AGENTS_INTERNAL_TOKEN", "live-secret")
    monkeypatch.setenv("EQ_DATA_DIR", str(tmp_path))
    # the user's token must be what tools use: make the fallback API key unusable
    monkeypatch.setenv("EDGEQUAKE_API_KEY", "invalid-on-purpose")
    v1.services.cache_clear()
    app = FastAPI()
    app.mount("/v1", v1.v1)
    async with httpx.AsyncClient(transport=httpx.ASGITransport(app=app), base_url="http://t", timeout=300,
                                 headers=user_headers()) as c:
        yield c
    v1.services.cache_clear()


async def chat(c, agent_id: str, text: str) -> tuple[str, list[dict]]:
    conv = (await c.post("/v1/conversations", json={"agent_id": agent_id})).json()
    r = await c.post(f"/v1/conversations/{conv['id']}/messages", json={"content": text})
    assert r.status_code == 200, r.text
    events = [json.loads(line[6:]) for line in r.text.splitlines() if line.startswith("data: ")]
    print("\nEVENTS:", [(e["type"], e.get("name") or e.get("author") or "") for e in events if e["type"] != "delta"])
    print("ANSWER:", "".join(e["text"] for e in events if e["type"] == "delta")[:600])
    return conv["id"], events


async def test_support_agent_sse_with_user_token(client):
    conv_id, events = await chat(client, "support_agent", "Quel est le ticket minimum pour investir dans Odyssey 2026 ?")
    kinds = [e["type"] for e in events]
    assert "error" not in kinds and kinds[-2:] == ["title", "done"]
    assert any(e["type"] == "tool_start" and e["name"].startswith("eq_") for e in events)
    assert all(e["ok"] for e in events if e["type"] == "tool_end")
    conv = (await client.get(f"/v1/conversations/{conv_id}")).json()
    assert conv["title"].startswith("Quel est le ticket minimum")
    assert [m["role"] for m in conv["messages"]][:1] == ["user"] and conv["messages"][-1]["content"].strip()


async def test_custom_agent_renders_graph_over_rest_with_user_token(client):
    agent = (await client.post("/v1/agents", json={
        "name": "Graph drawer", "model": "mistral/mistral-medium-latest", "tools": ["render_graph"],
        "instructions": "Call render_graph with the entity names the user gives (depth 1, max_nodes 20), then "
                        "summarise the result in two sentences in the user's language."})).json()
    conv_id, events = await chat(client, agent["id"], "Dessine le graphe autour de Discovery FCPR.")
    arts = [e for e in events if e["type"] == "artifact"]
    assert "error" not in [e["type"] for e in events]
    assert {a["mime_type"] for a in arts} == {"image/png", "text/html"}
    png = next(a for a in arts if a["mime_type"] == "image/png")
    r = await client.get(f"/v1/conversations/{conv_id}/artifacts/{png['name']}?version={png['version']}")
    assert r.status_code == 200 and r.content.startswith(b"\x89PNG")
