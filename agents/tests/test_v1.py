"""Offline tests for the /v1 product API (SPEC-165): no EdgeQuake, no LLM."""

import asyncio
import json
from functools import partial
from types import SimpleNamespace

import httpx
import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient
from google.adk.events import Event, EventActions
from google.adk.models import BaseLlm, LlmResponse
from google.adk.tools.mcp_tool import McpToolset
from google.genai import types

from edgequake_agents.common import catalog, guardrails, rest, v1
from edgequake_agents.common.config import eq_headers

TOKEN = "internal-secret"
MODEL = "mistral/mistral-small-latest"


def hdr(user="u1", ws="w1", bearer="user-token-1"):
    h = {"X-Agents-Internal-Token": TOKEN, "X-EQ-User-Id": user, "X-Tenant-ID": "t1"}
    if ws:
        h["X-Workspace-ID"] = ws
    if bearer:
        h["Authorization"] = f"Bearer {bearer}"
    return h


@pytest.fixture
def app(monkeypatch, tmp_path):
    monkeypatch.setenv("AGENTS_INTERNAL_TOKEN", TOKEN)
    monkeypatch.setenv("EQ_DATA_DIR", str(tmp_path))
    monkeypatch.delenv("EQ_ALLOWED_MODELS", raising=False)
    v1.services.cache_clear()
    v1._runs.clear()
    v1._active.clear()
    a = FastAPI()
    a.mount("/v1", v1.v1)
    yield a
    v1.services.cache_clear()


@pytest.fixture
def client(app):
    return TestClient(app)


def agent_body(**kw):
    return {"name": "Pricing helper", "instructions": "Answer {pricing} questions.", "model": MODEL,
            "tools": ["eq_retrieve", "render_graph"], "starters": ["How much?"], "temperature": 0.3, **kw}


# ---------------------------------------------------------------- auth


def test_auth_headers_enforced(client, monkeypatch):
    assert client.get("/v1/agents").status_code == 401
    assert client.get("/v1/agents", headers={**hdr(), "X-Agents-Internal-Token": "nope"}).json() == {"error": "invalid internal token"}
    assert client.get("/v1/agents", headers={"X-Agents-Internal-Token": TOKEN}).status_code == 401  # no user id
    assert client.get("/v1/agents", headers=hdr()).status_code == 200
    monkeypatch.delenv("AGENTS_INTERNAL_TOKEN")
    assert client.get("/v1/agents", headers=hdr()).status_code == 503


# ---------------------------------------------------------------- agents


def test_builtins_tools_models(client):
    items = client.get("/v1/agents", headers=hdr()).json()["items"]
    assert [a["id"] for a in items] == ["support_agent", "explorer_agent", "investigator_agent", "edgequake_assistant"]
    assert all(a["kind"] == "builtin" and not a["editable"] and a["instructions"] for a in items)
    assert "_agent" not in items[0]
    tools = {t["id"]: t for t in client.get("/v1/tools", headers=hdr()).json()["items"]}
    assert {"eq_retrieve", "render_graph", "save_markdown_report", "open_support_ticket"} <= tools.keys()
    assert {t["category"] for t in tools.values()} == {"knowledge", "graph", "documents", "artifacts"}
    models = client.get("/v1/models", headers=hdr()).json()["items"]
    assert len(models) == 3 and sum(m["default"] for m in models) == 1


@pytest.mark.parametrize("bad", [
    {"name": ""}, {"name": "x" * 61}, {"model": "openai/gpt-4o"}, {"tools": ["eq_document_delete"]},
    {"starters": ["a"] * 5}, {"starters": ["x" * 201]}, {"temperature": 1.5}, {"instructions": "x" * 20_001},
    {"visibility": "public"},
])
def test_agent_validation(client, bad):
    r = client.post("/v1/agents", headers=hdr(), json=agent_body(**bad))
    assert r.status_code == 400 and r.json()["error"]


def test_agent_crud_and_ownership(client):
    r = client.post("/v1/agents", headers=hdr(), json=agent_body())
    assert r.status_code == 201
    a = r.json()
    assert a["kind"] == "custom" and a["editable"] and a["owner_id"] == "u1" and a["workspace_id"] is None
    assert client.get(f"/v1/agents/{a['id']}", headers=hdr(user="u2")).status_code == 404  # private
    assert client.put(f"/v1/agents/{a['id']}", headers=hdr(user="u2"), json=agent_body()).status_code == 404

    r = client.put(f"/v1/agents/{a['id']}", headers=hdr(), json=agent_body(name="Shared", visibility="workspace"))
    assert r.status_code == 200 and r.json()["workspace_id"] == "w1" and r.json()["name"] == "Shared"
    other = client.get(f"/v1/agents/{a['id']}", headers=hdr(user="u2")).json()
    assert other["editable"] is False  # same workspace sees it, read-only
    assert client.get(f"/v1/agents/{a['id']}", headers=hdr(user="u2", ws="w2")).status_code == 404
    assert client.put(f"/v1/agents/{a['id']}", headers=hdr(user="u2"), json=agent_body()).status_code == 403
    assert client.delete(f"/v1/agents/{a['id']}", headers=hdr(user="u2")).status_code == 403
    assert any(x["id"] == a["id"] for x in client.get("/v1/agents", headers=hdr(user="u2")).json()["items"])

    assert client.put("/v1/agents/support_agent", headers=hdr(), json=agent_body()).status_code == 403
    assert client.post("/v1/agents", headers=hdr(ws=None), json=agent_body(visibility="workspace")).status_code == 400
    assert client.delete(f"/v1/agents/{a['id']}", headers=hdr()).status_code == 204
    assert client.get(f"/v1/agents/{a['id']}", headers=hdr()).status_code == 404


def test_duplicate_builtin(client):
    r = client.post("/v1/agents/explorer_agent/duplicate", headers=hdr())
    assert r.status_code == 201
    d, src = r.json(), catalog.BUILTINS["explorer_agent"]
    assert d["kind"] == "custom" and d["editable"] and d["name"] == "Explorer (copy)"
    assert d["instructions"] == src["instructions"] and d["tools"] == src["tools"]
    assert client.post("/v1/agents/nope/duplicate", headers=hdr()).status_code == 404


# ---------------------------------------------------------------- tool catalog / runtime


def test_tool_catalog_mapping():
    spec = {"kind": "custom", "name": "Pricing helper!", "description": "", "instructions": "Use {state} literally",
            "model": MODEL, "tools": ["render_graph", "eq_search", "eq_retrieve", "save_markdown_report"],
            "temperature": 0.4}
    agent = catalog.build_agent(spec)
    assert agent.name == "pricing_helper"
    mcp = [t for t in agent.tools if isinstance(t, McpToolset)]
    assert len(mcp) == 1 and mcp[0].tool_filter == ["eq_retrieve", "eq_search"]
    assert [t.__name__ for t in agent.tools if not isinstance(t, McpToolset)] == ["render_graph", "save_markdown_report"]
    assert catalog.build_agent(spec).tools[0] is mcp[0]  # toolsets are shared across runs
    assert agent.generate_content_config.temperature == 0.4
    assert agent.instruction(None) == "Use {state} literally"  # provider: no state templating
    assert agent.before_tool_callback is guardrails.guard_tool
    assert catalog.build_agent(catalog.BUILTINS["support_agent"]) is catalog.BUILTINS["support_agent"]["_agent"]
    assert catalog.agent_name("123") == "agent_123" and catalog.agent_name("Été") == "ete"
    assert catalog.agent_name("Planner") == "agent_planner"  # never hidden like the investigator's planner
    assert catalog.app_name("support_agent") == "support_agent" and catalog.app_name("abc") == "custom-abc"


async def test_save_markdown_report():
    saved = {}

    async def save_artifact(name, part):
        saved[name] = part
        return 0

    out = await catalog.save_markdown_report("Q3 Résumé", "Body", SimpleNamespace(save_artifact=save_artifact))
    assert out["artifact"] == "report-q3-resume.md"
    part = saved["report-q3-resume.md"].inline_data
    assert part.mime_type == "text/markdown" and part.data.decode() == "# Q3 Résumé\n\nBody"


# ---------------------------------------------------------------- conversations


def test_conversation_crud(client, app):
    c1 = client.post("/v1/conversations", headers=hdr(), json={"agent_id": "support_agent"})
    assert c1.status_code == 201 and c1.json()["title"] == "New chat"
    c2 = client.post("/v1/conversations", headers=hdr(), json={"agent_id": "explorer_agent"}).json()
    assert client.post("/v1/conversations", headers=hdr(), json={"agent_id": "nope"}).status_code == 404
    cid = c1.json()["id"]

    assert client.patch(f"/v1/conversations/{cid}", headers=hdr(), json={"title": "Fees 100%_x"}).json()["title"] == "Fees 100%_x"
    items = client.get("/v1/conversations", headers=hdr()).json()["items"]
    assert [i["id"] for i in items] == [cid, c2["id"]]  # newest (updated) first
    assert [i["id"] for i in client.get("/v1/conversations?q=fees", headers=hdr()).json()["items"]] == [cid]
    assert client.get("/v1/conversations?q=%25", headers=hdr()).json()["items"][0]["id"] == cid  # % is literal
    assert [i["id"] for i in client.get("/v1/conversations?agent_id=explorer_agent", headers=hdr()).json()["items"]] == [c2["id"]]
    assert client.get("/v1/conversations", headers=hdr(user="u2")).json()["items"] == []
    assert client.get(f"/v1/conversations/{cid}", headers=hdr(user="u2")).status_code == 404
    assert client.get(f"/v1/conversations/{cid}", headers=hdr()).json()["messages"] == []

    s = v1.services()
    asyncio.run(s.artifacts.save_artifact(app_name="support_agent", user_id="u1", session_id=cid, filename="a.md",
                                          artifact=types.Part.from_bytes(data=b"# hi", mime_type="text/markdown")))
    r = client.get(f"/v1/conversations/{cid}/artifacts/a.md?version=0", headers=hdr())
    assert r.status_code == 200 and r.content == b"# hi" and r.headers["content-type"].startswith("text/markdown")
    assert client.get(f"/v1/conversations/{cid}/artifacts/a.md", headers=hdr(user="u2")).status_code == 404

    assert client.delete(f"/v1/conversations/{cid}", headers=hdr(user="u2")).status_code == 404
    assert client.delete(f"/v1/conversations/{cid}", headers=hdr()).status_code == 204
    assert client.get(f"/v1/conversations/{cid}", headers=hdr()).status_code == 404
    assert asyncio.run(s.sessions.get_session(app_name="support_agent", user_id="u1", session_id=cid)) is None
    assert asyncio.run(s.artifacts.list_artifact_keys(app_name="support_agent", user_id="u1", session_id=cid)) == []


def ev(author, *parts, partial=None, delta=None, id="e"):
    return Event(id=id, author=author, invocation_id="i", partial=partial, timestamp=1_700_000_000.0,
                 content=types.Content(role="user" if author == "user" else "model", parts=list(parts)) if parts else None,
                 actions=EventActions(artifact_delta=delta or {}))


def call(name, cid):
    return types.Part(function_call=types.FunctionCall(id=cid, name=name, args={}))


def resp(name, cid, response):
    return types.Part(function_response=types.FunctionResponse(id=cid, name=name, response=response))


async def test_message_reconstruction():
    events = [
        ev("user", types.Part(text="Hello"), id="u"),
        ev("edgequake_assistant", call("transfer_to_agent", "c0"), id="a"),
        ev("edgequake_assistant", resp("transfer_to_agent", "c0", {"result": None})),
        ev("explorer_agent", types.Part(text="Let me look."), call("render_graph", "c1"), id="x"),
        ev("explorer_agent", types.Part(text="Le"), partial=True),  # partials are never part of history
        ev("explorer_agent", resp("render_graph", "c1", {"status": "ok"}), delta={"g.png": 0, "g.html": 0}),
        ev("explorer_agent", call("find_path", "c2")),
        ev("explorer_agent", resp("find_path", "c2", {"status": "error", "error": "boom"})),
        ev("explorer_agent", types.Part(text="Here is the graph."), types.Part(text="hidden", thought=True)),
        ev("explorer_agent"),  # state-only event
        ev("user", types.Part(text="Thanks"), id="u2"),
    ]

    async def mime(name, version):
        return "image/png" if name.endswith(".png") else "text/html"

    msgs = await v1.build_messages(events, mime)
    # one assistant message per turn: the routing hop and the specialist share it
    assert [(m["role"], m["author"]) for m in msgs] == [
        ("user", "user"), ("assistant", "edgequake_assistant"), ("user", "user")]
    bot = msgs[1]
    assert bot["id"] == "a" and bot["agents"] == ["edgequake_assistant", "explorer_agent"]
    assert bot["content"] == "Let me look.\n\nHere is the graph."
    assert bot["tools"] == [{"id": "c0", "name": "transfer_to_agent", "status": "done"},
                            {"id": "c1", "name": "render_graph", "status": "done"},
                            {"id": "c2", "name": "find_path", "status": "error"}]
    assert bot["artifacts"] == [{"name": "g.png", "mime_type": "image/png", "version": 0},
                                {"name": "g.html", "mime_type": "text/html", "version": 0}]
    assert msgs[2]["content"] == "Thanks" and "plan" not in bot and "sources" not in bot
    assert msgs[0]["created_at"].startswith("2023-11-14")


async def test_ui_events_folded_into_history_and_planner_hidden():
    src = {"chunk_id": "d1-chunk-0", "document_id": "d1", "title": "a.md", "snippet": "s" * 2000, "score": 0.5}
    events = [
        ev("user", types.Part(text="Q"), id="u"),
        ev("planner", types.Part(text='{"question": "Q"}'), id="p"),  # structured plan: never shown
        Event(id="s1", author="investigator_agent", invocation_id="i", custom_metadata={"eq_ui": [
            {"type": "plan", "items": [{"id": "r1", "label": "a?"}]},
            {"type": "step", "id": "r1", "label": "a?", "status": "running"}]}),
        Event(id="s2", author="investigator_agent", invocation_id="i", custom_metadata={"eq_ui": [
            {"type": "step", "id": "r1", "label": "a?", "status": "done", "sources_count": 1}]}),
        ev("support_agent", call("knowledge_query", "k1"), id="k"),
        ev("support_agent", resp("knowledge_query", "k1", {"status": "ok", "answer": "A", "sources": [src, src]})),
        ev("synthesizer", types.Part(text="Answer [1]"), id="y"),
    ]
    out = [e async for e in v1.translate(_aiter(events), None)]
    assert {"type": "message_start", "author": "planner"} not in out
    assert [e["type"] for e in out if e["type"] in ("plan", "step", "sources")] == ["plan", "step", "step", "sources"]
    sources = next(e for e in out if e["type"] == "sources")["items"]
    assert sources[0]["title"] == "a.md" and len(sources[0]["content"]) == 2000 and len(sources[0]["snippet"]) <= 280

    (msg,) = [m for m in await v1.build_messages(events) if m["role"] == "assistant"]
    assert msg["author"] == "investigator_agent" and msg["content"] == "Answer [1]"
    assert msg["plan"] == [{"id": "r1", "label": "a?"}]
    assert msg["steps"] == [{"id": "r1", "label": "a?", "status": "done", "sources_count": 1}]
    assert [s["chunk_id"] for s in msg["sources"]] == ["d1-chunk-0"]  # deduped


async def _aiter(items):
    for x in items:
        yield x


# ---------------------------------------------------------------- SSE runs (fake LLM)

SEEN: list[tuple[str, str, str]] = []  # (run's bearer, REST bearer, MCP header_provider bearer)


class FakeLlm(BaseLlm):
    """graph_overview -> save_markdown_report -> streamed text; records the headers it sees."""
    model: str = "fake"

    async def generate_content_async(self, llm_request, stream=False):
        mcp = await catalog.shared_toolset(("eq_search",)).header_provider(None)
        await asyncio.sleep(0.02)  # let the concurrent run interleave
        SEEN.append(("llm", eq_headers().get("Authorization"), mcp.get("Authorization")))
        done = [p.function_response.name for c in llm_request.contents for p in c.parts or [] if p.function_response]
        if "graph_overview" not in done:
            yield LlmResponse(content=types.Content(role="model", parts=[call("graph_overview", f"go-{id(llm_request)}")]))
        elif "save_markdown_report" not in done:
            yield LlmResponse(content=types.Content(role="model", parts=[types.Part(function_call=types.FunctionCall(
                id=f"r-{id(llm_request)}", name="save_markdown_report", args={"title": "Report", "markdown": "# R"}))]))
        else:
            for chunk in ("Do", "ne."):
                yield LlmResponse(content=types.Content(role="model", parts=[types.Part(text=chunk)]), partial=True)
            yield LlmResponse(content=types.Content(role="model", parts=[types.Part(text="Done.")]))


@pytest.fixture
def fake_runtime(monkeypatch):
    SEEN.clear()
    monkeypatch.setattr(catalog, "model", lambda name=None, **_: FakeLlm())

    def handler(request: httpx.Request):
        SEEN.append(("rest", request.headers.get("authorization"), request.headers.get("x-workspace-id")))
        body = {"/api/v1/graph/facets": {"entity_types": [{"name": "ORG", "count": 2}], "relationship_types": []},
                "/api/v1/graph/labels/popular": {"labels": []},
                "/api/v1/graph/communities": {"communities": []}}[request.url.path]
        return httpx.Response(200, json=body)

    real = httpx.AsyncClient
    monkeypatch.setattr(rest, "httpx", SimpleNamespace(AsyncClient=partial(real, transport=httpx.MockTransport(handler))))


def parse_sse(text):
    return [json.loads(line[6:]) for line in text.splitlines() if line.startswith("data: ")]


async def test_sse_flow_and_per_request_header_isolation(app, fake_runtime):
    transport = httpx.ASGITransport(app=app)
    async with httpx.AsyncClient(transport=transport, base_url="http://t") as c:
        agent = (await c.post("/v1/agents", headers=hdr(), json=agent_body(
            tools=["graph_overview", "save_markdown_report"], visibility="workspace"))).json()
        convs = [(await c.post("/v1/conversations", headers=hdr(user=u), json={"agent_id": agent["id"]})).json()
                 for u in ("u1", "u2")]

        async def run(user, token, conv):
            r = await c.post(f"/v1/conversations/{conv['id']}/messages", headers=hdr(user=user, bearer=token),
                             json={"content": "  Give me   the overview " + "x" * 80})
            assert r.headers["content-type"].startswith("text/event-stream")
            return parse_sse(r.text)

        out1, out2 = await asyncio.gather(run("u1", "tok-A", convs[0]), run("u2", "tok-B", convs[1]))

        # each run's LLM calls, REST calls and MCP headers carry that run's own bearer
        bearers = {s[1] for s in SEEN}
        assert bearers == {"Bearer tok-A", "Bearer tok-B"}
        assert all(s[1] == s[2] for s in SEEN if s[0] == "llm")
        assert sum(s[0] == "rest" for s in SEEN) == 6 and {s[2] for s in SEEN if s[0] == "rest"} == {"w1"}
        assert eq_headers().get("Authorization") != "Bearer tok-A"  # nothing leaks outside the run

        types_ = [e["type"] for e in out1]
        assert types_[0] == "message_start" and types_[-2:] == ["title", "done"]
        assert types_.count("tool_start") == 2 and types_.count("tool_end") == 2 and "error" not in types_
        assert "".join(e["text"] for e in out1 if e["type"] == "delta") == "Done."  # aggregate not repeated
        art = next(e for e in out1 if e["type"] == "artifact")
        assert art == {"type": "artifact", "name": "report-report.md", "mime_type": "text/markdown", "version": 0}
        title = next(e["title"] for e in out1 if e["type"] == "title")
        assert title == ("Give me the overview " + "x" * 80)[:60]
        assert [e["type"] for e in out2][-1] == "done"

        conv = (await c.get(f"/v1/conversations/{convs[0]['id']}", headers=hdr())).json()
        assert conv["title"] == title
        user_msg, bot = conv["messages"]
        assert user_msg["role"] == "user" and bot["author"] == "pricing_helper" and bot["content"] == "Done."
        assert [t["name"] for t in bot["tools"]] == ["graph_overview", "save_markdown_report"]
        assert bot["artifacts"] == [{"name": "report-report.md", "mime_type": "text/markdown", "version": 0}]
        r = await c.get(f"/v1/conversations/{convs[0]['id']}/artifacts/report-report.md", headers=hdr())
        assert r.text == "# R" and r.headers["content-type"].startswith("text/markdown")

        # second message: no new title
        again = parse_sse((await c.post(f"/v1/conversations/{convs[0]['id']}/messages", headers=hdr(bearer="tok-A"),
                                        json={"content": "more"})).text)
        assert "title" not in [e["type"] for e in again]


def test_message_limits(client, monkeypatch):
    conv = client.post("/v1/conversations", headers=hdr(), json={"agent_id": "support_agent"}).json()
    url = f"/v1/conversations/{conv['id']}/messages"
    assert client.post(url, headers=hdr(), content=json.dumps({"content": "x" * 33_000})).status_code == 413
    assert client.post(url, headers=hdr(), json={"content": "  "}).status_code == 400
    assert client.post(url, headers=hdr(bearer=None), json={"content": "hi"}).status_code == 401
    assert client.post(url, headers=hdr(user="u2"), json={"content": "hi"}).status_code == 404
    v1._active.add(conv["id"])
    assert client.post(url, headers=hdr(), json={"content": "hi"}).status_code == 409
    v1._active.clear()
    for _ in range(v1.RUNS_PER_MINUTE):
        assert not v1.rate_limited("u9")
    assert v1.rate_limited("u9")


class FakeEdgeQuakeOAuth:
    """MockTransport handler for PRM / register / authorize / token."""

    def __init__(self):
        self.calls, self.known, self.challenge = [], set(), None

    def __call__(self, request: httpx.Request):
        import base64
        import hashlib
        from urllib.parse import parse_qs

        path = request.url.path
        self.calls.append(path)
        if path.startswith("/.well-known"):
            return httpx.Response(200, json={"resource": "https://eq/mcp"})
        if path == "/oauth/register":
            cid = f"cid{self.calls.count('/oauth/register')}"
            self.known.add(cid)
            return httpx.Response(201, json={"client_id": cid})
        form = {k: v[0] for k, v in parse_qs(request.content.decode()).items()}
        if form["client_id"] not in self.known:
            return httpx.Response(400, json={"message": f"Bad request: unknown oauth client_id: {form['client_id']}"})
        if path == "/oauth/authorize":
            assert request.headers["authorization"] == "Bearer a.b.c" and form["resource"] == "https://eq/mcp"
            assert form["approve"] == "1" and form["scope"] == "edgequake:read edgequake:query"
            self.challenge = form["code_challenge"]
            return httpx.Response(303, headers={"location": f"{form['redirect_uri']}?code=CODE&state=s"})
        assert form["code"] == "CODE"
        challenge = base64.urlsafe_b64encode(hashlib.sha256(form["code_verifier"].encode()).digest()).rstrip(b"=")
        assert challenge.decode() == self.challenge  # PKCE S256
        return httpx.Response(200, json={"access_token": f"mcp-tok-{form['client_id']}", "expires_in": 900})


@pytest.fixture
def fake_oauth(monkeypatch, tmp_path):
    from edgequake_agents.common import mcp

    eq = FakeEdgeQuakeOAuth()
    real = httpx.AsyncClient
    monkeypatch.setattr(mcp, "httpx", SimpleNamespace(AsyncClient=partial(real, transport=httpx.MockTransport(eq))))
    monkeypatch.setattr(mcp, "_resource", {})
    monkeypatch.setattr(mcp, "_tokens", {})
    monkeypatch.setenv("EQ_DATA_DIR", str(tmp_path))
    monkeypatch.setenv("EDGEQUAKE_URL", "http://eq-a")
    return mcp, eq


async def test_mcp_headers_swap_session_jwt_for_mcp_token(fake_oauth, monkeypatch):
    from edgequake_agents.common.config import request_headers

    mcp, eq = fake_oauth
    monkeypatch.setenv("EDGEQUAKE_API_KEY", "eq_key")
    assert (await mcp.mcp_headers())["Authorization"] == "Bearer eq_key"  # legacy route: API key, no exchange
    tok = request_headers.set({"Authorization": "Bearer a.b.c", "X-Workspace-ID": "w"})
    try:
        assert await mcp.mcp_headers() == {"Authorization": "Bearer mcp-tok-cid1", "X-Workspace-ID": "w"}
        assert (await mcp.mcp_headers())["Authorization"] == "Bearer mcp-tok-cid1"  # cached
        assert eq.calls.count("/oauth/token") == 1 and eq.calls.count("/oauth/register") == 1
        request_headers.set({"Authorization": "Bearer eq_userkey"})
        assert (await mcp.mcp_headers())["Authorization"] == "Bearer eq_userkey"  # API keys pass through
    finally:
        request_headers.reset(tok)


async def test_oauth_client_persisted_reused_and_reregistered(fake_oauth, monkeypatch):
    mcp, eq = fake_oauth
    assert (await mcp._exchange("a.b.c"))[0] == "mcp-tok-cid1"
    assert mcp._stored_client("http://eq-a")["client_id"] == "cid1"

    monkeypatch.setattr(mcp, "_resource", {})  # "process restart": only agents.db survives
    assert (await mcp._exchange("a.b.c"))[0] == "mcp-tok-cid1"
    assert eq.calls.count("/oauth/register") == 1  # reused, no new row in EdgeQuake

    eq.known.clear()  # EdgeQuake forgot the client (e.g. database reset) -> re-register once
    assert (await mcp._exchange("a.b.c"))[0] == "mcp-tok-cid2"
    assert eq.calls.count("/oauth/register") == 2
    assert mcp._stored_client("http://eq-a")["client_id"] == "cid2"  # replaced in agents.db

    monkeypatch.setenv("EDGEQUAKE_URL", "http://eq-b")  # other EdgeQuake base URL -> its own client
    assert (await mcp._exchange("a.b.c"))[0] == "mcp-tok-cid3"
    assert eq.calls.count("/oauth/register") == 3
    assert mcp._stored_client("http://eq-a")["client_id"] == "cid2"
    assert mcp._stored_client("http://eq-b")["client_id"] == "cid3"


# ---------------------------------------------------------------- agent architect (system agent)

from edgequake_agents.common import architect  # noqa: E402


def good_spec(**kw):
    return {"name": "Fund FAQ", "icon": "💼", "description": "Answers fund questions.", "model": MODEL,
            "instructions": "## Role\nYou answer.", "tools": ["eq_retrieve"], "starters": ["Fees?"],
            "temperature": 0.1, "visibility": "private", **kw}


def test_architect_hidden_from_listing(client):
    ids = [a["id"] for a in client.get("/v1/agents", headers=hdr()).json()["items"]]
    assert architect.AGENT_ID not in ids
    items = client.get("/v1/agents?include=system", headers=hdr()).json()["items"]
    arch = next(a for a in items if a["id"] == architect.AGENT_ID)
    assert arch["kind"] == "system" and not arch["editable"] and "_agent" not in arch
    assert MODEL in arch["instructions"] and "save_markdown_report" in arch["instructions"]  # catalog injected
    assert client.get(f"/v1/agents/{architect.AGENT_ID}", headers=hdr()).status_code == 200
    assert client.post(f"/v1/agents/{architect.AGENT_ID}/duplicate", headers=hdr()).status_code == 400
    assert client.put(f"/v1/agents/{architect.AGENT_ID}", headers=hdr(), json=agent_body()).status_code == 403


async def test_propose_agent_spec_validation(monkeypatch):
    from edgequake_agents.common.config import request_headers

    bad = await architect.propose_agent_spec(good_spec(model="openai/gpt-4o", tools=["eq_document_delete"],
                                                       starters=["a"] * 5), "r", [], None)
    assert bad["status"] == "error" and len(bad["errors"]) == 3
    assert any("model" in e for e in bad["errors"]) and any("tools" in e for e in bad["errors"])
    tok = request_headers.set({})
    try:
        ws = await architect.propose_agent_spec(good_spec(visibility="workspace"), "r", [], None)
        assert ws["status"] == "error" and "workspace" in ws["errors"][0]
    finally:
        request_headers.reset(tok)
    ok = await architect.propose_agent_spec(architect.AgentSpec(**good_spec(name="  Fund FAQ ", tools=["eq_retrieve"] * 2)),
                                            "Because.", ["Q1", " ", "Q2", "Q3", "Q4"], None)
    assert ok["status"] == "ok" and ok["spec"]["name"] == "Fund FAQ" and ok["spec"]["tools"] == ["eq_retrieve"]
    assert ok["tests"] == ["Q1", "Q2", "Q3"] and ok["rationale"] == "Because."


ARCH_SEEN: list = []


class FakeArchitectLlm(BaseLlm):
    """Bad spec -> (error) -> good spec -> short text. Records each request's contents."""
    model: str = "fake-architect"

    async def generate_content_async(self, llm_request, stream=False):
        ARCH_SEEN.append(llm_request.contents)
        done = [p.function_response for c in llm_request.contents for p in c.parts or [] if p.function_response]
        def propose(spec, cid):
            return LlmResponse(content=types.Content(role="model", parts=[types.Part(function_call=types.FunctionCall(
                id=cid, name="propose_agent_spec", args={"spec": spec, "rationale": "Grounded.", "tests": ["T1", "T2"]}))]))
        if not done or done[-1].name != "propose_agent_spec":
            yield propose(good_spec(model="nope"), f"p1-{len(ARCH_SEEN)}")
        elif done[-1].response.get("status") == "error":
            yield propose(good_spec(), f"p2-{len(ARCH_SEEN)}")
        else:
            yield LlmResponse(content=types.Content(role="model", parts=[types.Part(text="Voici l'agent.")]))


@pytest.fixture
def fake_architect(monkeypatch):
    ARCH_SEEN.clear()
    agent = catalog.BUILTINS[architect.AGENT_ID]["_agent"]
    monkeypatch.setattr(agent, "model", FakeArchitectLlm())
    monkeypatch.setattr(agent, "tools", [t for t in agent.tools if not isinstance(t, McpToolset)])  # no network


async def test_architect_sse_agent_spec_and_context(app, fake_architect):
    async with httpx.AsyncClient(transport=httpx.ASGITransport(app=app), base_url="http://t") as c:
        conv = (await c.post("/v1/conversations", headers=hdr(), json={"agent_id": architect.AGENT_ID})).json()
        url = f"/v1/conversations/{conv['id']}/messages"
        draft = {"name": "Draft", "tools": ["eq_retrieve"], "temperature": 7, "junk": "x", "instructions": "i" * 30_000}
        out = parse_sse((await c.post(url, headers=hdr(), json={"content": "Un agent FAQ fonds",
                                                                "context": {"draft_agent": draft}})).text)
        kinds = [e["type"] for e in out]
        assert kinds.count("tool_start") == 2 and kinds.count("agent_spec") == 1 and "error" not in kinds
        ends = [e["ok"] for e in out if e["type"] == "tool_end"]
        assert ends == [False, True]  # validation error went back to the model, which fixed the spec
        spec_ev = next(e for e in out if e["type"] == "agent_spec")
        assert kinds.index("agent_spec") == len(kinds) - 1 - kinds[::-1].index("tool_end") + 1  # right after tool_end
        assert spec_ev["spec"]["name"] == "Fund FAQ" and spec_ev["spec"]["model"] == MODEL
        assert spec_ev["rationale"] == "Grounded." and spec_ev["tests"] == ["T1", "T2"]
        assert next(e["title"] for e in out if e["type"] == "title") == "Conception : Fund FAQ"

        # the model saw the context as a separate, capped data part of the user turn
        user_parts = [p.text for p in ARCH_SEEN[0][0].parts]
        assert user_parts[0].startswith(architect.CONTEXT_MARK) and user_parts[1] == "Un agent FAQ fonds"
        ctx = json.loads(user_parts[0].split("```json\n")[1].split("\n```")[0])["draft_agent"]
        assert ctx["temperature"] == 1.0 and "junk" not in ctx and len(ctx["instructions"]) == 20_000

        detail = (await c.get(f"/v1/conversations/{conv['id']}", headers=hdr())).json()
        assert detail["title"] == "Conception : Fund FAQ"
        user, bot = detail["messages"]
        assert user["content"] == "Un agent FAQ fonds"  # context part hidden from history
        assert bot["agent_spec"]["spec"]["name"] == "Fund FAQ" and bot["content"] == "Voici l'agent."

        # context validation
        bad = await c.post(url, headers=hdr(), json={"content": "x", "context": {"draft_agent": {"tools": "nope"}}})
        assert bad.status_code == 400
        assert (await c.post(url, headers=hdr(), json={"content": "x", "context": {"editing_agent_id": "nope"}})).status_code == 404
        big = {"content": "x", "context": {"draft_agent": {"instructions": "x" * 40_000}}}
        assert (await c.post(url, headers=hdr(), json=big)).status_code == 413


def test_context_ignored_for_other_agents(client, fake_runtime):
    agent = client.post("/v1/agents", headers=hdr(), json=agent_body(tools=["graph_overview"])).json()
    conv = client.post("/v1/conversations", headers=hdr(), json={"agent_id": agent["id"]}).json()
    # unknown editing_agent_id would be a 404 for the architect; other agents ignore the context
    r = client.post(f"/v1/conversations/{conv['id']}/messages", headers=hdr(user="u1", ws="w1"),
                    json={"content": "hi", "context": {"editing_agent_id": "nope"}})
    assert r.status_code == 200 and "error" not in [e["type"] for e in parse_sse(r.text)]


# ---------------------------------------------------------------- workspace isolation


def test_conversations_scoped_to_workspace(client):
    cid = client.post("/v1/conversations", headers=hdr(ws="w1"), json={"agent_id": "support_agent"}).json()["id"]
    assert [c["id"] for c in client.get("/v1/conversations", headers=hdr(ws="w1")).json()["items"]] == [cid]
    assert client.get("/v1/conversations", headers=hdr(ws="w2")).json()["items"] == []
    other = hdr(ws="w2")
    assert client.get(f"/v1/conversations/{cid}", headers=other).status_code == 404
    assert client.patch(f"/v1/conversations/{cid}", headers=other, json={"title": "x"}).status_code == 404
    assert client.post(f"/v1/conversations/{cid}/messages", headers=other, json={"content": "hi"}).status_code == 404
    assert client.get(f"/v1/conversations/{cid}/artifacts/a.md", headers=other).status_code == 404
    assert client.delete(f"/v1/conversations/{cid}", headers=other).status_code == 404
    assert client.get(f"/v1/conversations/{cid}", headers={**hdr(ws="w1"), "X-Tenant-ID": "t2"}).status_code == 404
    assert client.get(f"/v1/conversations/{cid}", headers=hdr(ws="w1")).status_code == 200


def test_legacy_conversations_claimed_by_first_workspace(client):
    s = v1.services().store
    with s._db() as db:  # a row from before workspace scoping
        db.execute("INSERT INTO conversations (id, user_id, agent_id, title, created_at, updated_at) "
                   "VALUES ('old', 'u1', 'support_agent', 'Old', '2026-01-01', '2026-01-01')")
    assert client.get("/v1/conversations", headers=hdr(user="u2", ws="w9")).json()["items"] == []  # never another user's
    assert [c["id"] for c in client.get("/v1/conversations", headers=hdr(ws="w2")).json()["items"]] == ["old"]
    assert client.get("/v1/conversations/old", headers=hdr(ws="w1")).status_code == 404  # now lives in w2
    assert client.get("/v1/conversations/old", headers=hdr(ws="w2")).status_code == 200


def test_store_migrates_old_schema(tmp_path):
    import sqlite3
    from edgequake_agents.common.store import Store

    db = sqlite3.connect(tmp_path / "old.db")
    db.execute("CREATE TABLE conversations (id TEXT PRIMARY KEY, user_id TEXT NOT NULL, agent_id TEXT NOT NULL, "
               "title TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)")
    db.execute("INSERT INTO conversations VALUES ('c', 'u', 'a', 't', 'x', 'x')")
    db.commit()
    db.close()
    st = Store(tmp_path / "old.db")
    Store(tmp_path / "old.db")  # idempotent
    assert st.get_conversation("u", "c", "t1", "w1")["workspace_id"] == "w1"


# ---------------------------------------------------------------- knowledge tools + investigator pipeline

LOG: list[tuple] = []  # ("llm", agent) / ("http", path, bearer) in call order
DOCS = {"documents": [{"id": "d1", "title": "altaroc_produits.md"}, {"id": "d2", "title": "altaroc_doc.md"}]}


def eq_handler(request: httpx.Request):
    path, auth = request.url.path, request.headers.get("authorization")
    LOG.append(("http", path, auth))
    body = json.loads(request.content or b"{}")
    if path == "/api/v1/query":
        return httpx.Response(200, json={"answer": "Odyssey 2026 est un fonds [1].", "sources": [
            {"source_type": "chunk", "id": "d1-chunk-3", "score": 2.5, "snippet": "Ticket 100 000 €", "reference_id": 1},
            {"source_type": "entity", "id": "ALTAROC_ODYSSEY_2026", "entity_type": "FUND", "snippet": "6th vintage",
             "document_id": "d1", "score": 1.0}]})
    if path == "/api/v1/query/context":
        n = "1" if "frais" in body["query"] else "2"
        return httpx.Response(200, json={"bundle": {
            "chunks": [{"id": f"d{n}-chunk-0", "content": f"passage about {body['query']}", "score": 1.0 + int(n)},
                       {"id": "d1-chunk-9", "content": "shared passage", "score": 0.5}],
            "documents": [{"document_id": "d2", "title": "altaroc_doc.md"}],
            "subgraph": {"entities": [{"name": "Odyssey 2026", "entity_type": "FUND", "description": "a fund"}],
                         "relationships": [{"source_label": "Altaroc", "target_label": "Odyssey 2026",
                                            "relation_type": "MANAGES", "description": "manages"}]}}})
    if path == "/api/v1/documents":
        return httpx.Response(200, json=DOCS)
    if path == "/api/v1/graph":
        node = request.url.params["start_node"]
        return httpx.Response(200, json={"nodes": [{"id": node, "label": node, "node_type": "FUND", "degree": {"total": 1}},
                                                   {"id": "B", "label": "B", "node_type": "ORG", "degree": {"total": 1}}],
                                         "edges": [{"source": node, "target": "B", "relationship_type": "PART_OF"}]})
    return httpx.Response(404, json={"error": path})


@pytest.fixture
def fake_eq(monkeypatch):
    LOG.clear()
    from edgequake_agents.common import knowledge

    knowledge._titles.clear()
    real = httpx.AsyncClient
    monkeypatch.setattr(rest, "httpx", SimpleNamespace(AsyncClient=partial(real, transport=httpx.MockTransport(eq_handler))))


async def test_knowledge_tools_use_full_pipeline_and_map_sources(fake_eq, monkeypatch):
    from edgequake_agents.common import config, knowledge

    token = config.request_headers.set({"Authorization": "Bearer user-jwt", "X-Workspace-ID": "w1"})
    try:
        ctx = SimpleNamespace(invocation_id="inv-1")
        q = await knowledge.knowledge_query("c'est quoi Odyssey 2026 ?", ctx)
        c = await knowledge.knowledge_context("Odyssey 2026 frais", ctx)
        again = await knowledge.knowledge_context("Odyssey 2026 frais", ctx)
    finally:
        config.request_headers.reset(token)
    assert q["answer"].startswith("Odyssey 2026") and q["entities"][0]["name"] == "ALTAROC_ODYSSEY_2026"
    assert q["sources"] == [{"chunk_id": "d1-chunk-3", "document_id": "d1", "title": "altaroc_produits.md",
                             "snippet": "Ticket 100 000 €", "score": 2.5, "ref": 1}]
    assert [s["title"] for s in c["sources"]] == ["altaroc_produits.md", "altaroc_produits.md"]
    # one citation numbering per turn across tool calls; EdgeQuake's own [n] are remapped
    assert [s["ref"] for s in c["sources"]] == [2, 3] and q["answer"] == "Odyssey 2026 est un fonds [1]."
    assert [s["ref"] for s in again["sources"]] == [2, 3] and again["query"] == "Odyssey 2026 frais"
    assert c["relationships"][0] == {"source": "Altaroc", "target": "Odyssey 2026", "type": "MANAGES", "description": "manages"}
    assert {x[2] for x in LOG} == {"Bearer user-jwt"}  # the user's own credentials
    assert [x[1] for x in LOG].count("/api/v1/documents") == 1  # titles cached per workspace

    # the catalog exposes them (custom agents, architect) as plain Python tools
    for tid in knowledge.TOOL_NAMES:
        assert catalog.TOOLS[tid].fn is getattr(knowledge, tid) and catalog.TOOLS[tid].category == "knowledge"
    custom = catalog.build_agent({"kind": "custom", "name": "K", "description": "", "instructions": "x",
                                  "model": MODEL, "tools": ["knowledge_query"], "temperature": 0})
    assert custom.tools == [knowledge.knowledge_query]
    support = catalog.BUILTINS["support_agent"]
    assert support["tools"][:2] == ["knowledge_context", "knowledge_query"]
    assert {knowledge.knowledge_query, knowledge.knowledge_context} <= set(support["_agent"].sub_agents[0].tools)
    assert "knowledge_context" in catalog.BUILTINS[architect_id()]["tools"]

    def boom(_):
        return httpx.Response(500, json={})
    monkeypatch.setattr(rest, "httpx", SimpleNamespace(AsyncClient=partial(httpx.AsyncClient, transport=httpx.MockTransport(boom))))
    assert (await knowledge.knowledge_query("x", SimpleNamespace(invocation_id="i2")))["status"] == "error"


def architect_id():
    from edgequake_agents.common import architect
    return architect.AGENT_ID


class FakeInvestigatorLlm(BaseLlm):
    """planner -> JSON plan; synthesizer -> streamed answer. Logs each call."""
    model: str = "fake"

    async def generate_content_async(self, llm_request, stream=False):
        system = str(llm_request.config.system_instruction or "")
        if "You plan research" in system:
            LOG.append(("llm", "planner"))
            plan = {"question": "Odyssey 2026 ?", "language": "French", "key_entities": ["Odyssey 2026"],
                    "sub_questions": ["Odyssey 2026 structure", "Odyssey 2026 frais", "Odyssey 2026 TRI"]}
            yield LlmResponse(content=types.Content(role="model", parts=[types.Part(text=json.dumps(plan))]))
            return
        LOG.append(("llm", "synthesizer"))
        assert "[1] (altaroc_doc.md)" in system and "Altaroc —MANAGES→ Odyssey 2026" in system
        for chunk in ("Odyssey ", "2026 [1]."):
            yield LlmResponse(content=types.Content(role="model", parts=[types.Part(text=chunk)]), partial=True)
        yield LlmResponse(content=types.Content(role="model", parts=[types.Part(text="Odyssey 2026 [1].")]))


async def test_investigator_pipeline_events_and_history(app, fake_eq, monkeypatch):
    from edgequake_agents.investigator_agent import agent as inv

    monkeypatch.setattr(inv, "model", lambda *a, **k: FakeInvestigatorLlm())
    monkeypatch.setitem(catalog.BUILTINS["investigator_agent"], "_agent", inv.create_agent())
    transport = httpx.ASGITransport(app=app)
    async with httpx.AsyncClient(transport=transport, base_url="http://t") as c:
        conv = (await c.post("/v1/conversations", headers=hdr(), json={"agent_id": "investigator_agent"})).json()
        out = parse_sse((await c.post(f"/v1/conversations/{conv['id']}/messages", headers=hdr(bearer="tok-I"),
                                      json={"content": "c'est quoi Odyssey 2026 ?"})).text)
        hist = (await c.get(f"/v1/conversations/{conv['id']}", headers=hdr())).json()

    # 2 LLM calls; every retrieval runs between them, none during retrieval
    kinds = [x[1] if x[0] == "llm" else x[1].removeprefix("/api/v1") for x in LOG]
    assert kinds.count("planner") == 1 and kinds.count("synthesizer") == 1
    p, s = kinds.index("planner"), kinds.index("synthesizer")
    assert [k for k in kinds[p + 1:s] if k != "/documents"] == ["/query/context"] * 3
    assert all(k.startswith("/graph") for k in kinds[s + 1:])
    assert {x[2] for x in LOG if x[0] == "http"} == {"Bearer tok-I"}

    types_ = [e["type"] for e in out]
    assert "error" not in types_
    order = [t for t in types_ if t in ("plan", "sources", "delta")]
    assert order[0] == "plan" and order.index("sources") < order.index("delta")
    assert types_.index("plan") < types_.index("sources") < types_.index("delta")
    steps = [(e["id"], e["status"]) for e in out if e["type"] == "step"]
    assert steps[:2] == [("plan", "running"), ("plan", "done")]
    assert {("r1", "done"), ("r2", "done"), ("r3", "done"), ("synth", "done"), ("graph", "done")} <= set(steps)
    assert steps.index(("synth", "running")) > max(steps.index((f"r{i}", "done")) for i in (1, 2, 3))
    assert "".join(e["text"] for e in out if e["type"] == "delta") == "Odyssey 2026 [1]."
    assert not any("sub_questions" in e.get("text", "") for e in out)  # planner JSON never streamed
    sources = next(e for e in out if e["type"] == "sources")["items"]
    assert [x["chunk_id"] for x in sources] == ["d2-chunk-0", "d1-chunk-0", "d1-chunk-9"]
    assert sources[2]["steps"] == ["r1", "r2", "r3"] and sources[0]["ref"] == 1
    arts = {e["name"] for e in out if e["type"] == "artifact"}
    assert {"graph-odyssey-2026.png", "graph-odyssey-2026.html", "investigation-odyssey-2026.md"} <= arts

    users = [m for m in hist["messages"] if m["role"] == "user"]
    (bot,) = [m for m in hist["messages"] if m["role"] == "assistant"]
    assert len(users) == 1 and bot["content"] == "Odyssey 2026 [1]."
    assert [i["label"] for i in bot["plan"]] == ["Odyssey 2026 structure", "Odyssey 2026 frais", "Odyssey 2026 TRI"]
    assert {st["id"]: st["status"] for st in bot["steps"]} == {
        "plan": "done", "r1": "done", "r2": "done", "r3": "done", "synth": "done", "graph": "done"}
    assert sum(st.get("sources_count", 0) for st in bot["steps"]) == 6
    assert [x["chunk_id"] for x in bot["sources"]] == [x["chunk_id"] for x in sources]
    assert {a["name"] for a in bot["artifacts"]} == arts


def test_ui_source_excerpt_is_clean_and_query_focused():
    from edgequake_agents.common import knowledge

    chunk = ("# ALTAROC — CATALOGUE\n## GAMME 1 — ODYSSEY\n### 💧 Concept\n\n---\nCorps du message :\n```\n"
             "| Critère | Odyssey | Discovery |\n|---|---|---|\n| **Frais** | 2,5% | 1,1% |\n```\n"
             "- **TRI cible** : 13% par an avant fiscalité. Multiple cible 1,7x.\n" + "Autre phrase longue. " * 40)
    (s,) = knowledge.ui_sources([{"chunk_id": "d-chunk-0", "document_id": "d", "title": "t", "score": 1.0,
                                  "snippet": chunk}], "Quel est le TRI cible d'Odyssey ?")
    assert s["section"] == "GAMME 1 — ODYSSEY › Concept" and s["content"] == chunk
    assert s["snippet"].startswith("TRI cible : 13% par an avant fiscalité.") and len(s["snippet"]) <= 280
    assert not any(x in s["snippet"] for x in ("#", "|", "```", "**", "💧"))


class FakeSupportLlm(BaseLlm):
    """Answers in one call; checks that the retrieved passages are in its instruction."""
    model: str = "fake"

    async def generate_content_async(self, llm_request, stream=False):
        system = str(llm_request.config.system_instruction or "")
        LOG.append(("llm", "support_writer"))
        assert "[1] (altaroc_doc.md)\npassage about c'est quoi Odyssey 2026 ?" in system and "MANAGES" in system
        yield LlmResponse(content=types.Content(role="model", parts=[types.Part(text="Odyssey [1].")]), partial=True)
        yield LlmResponse(content=types.Content(role="model", parts=[types.Part(text="Odyssey [1].")]))


async def test_support_retrieves_before_its_single_llm_call(app, fake_eq, monkeypatch):
    from edgequake_agents.support_agent import agent as sup

    monkeypatch.setattr(sup, "model", lambda *a, **k: FakeSupportLlm())
    monkeypatch.setitem(catalog.BUILTINS["support_agent"], "_agent", sup.create_agent())
    transport = httpx.ASGITransport(app=app)
    async with httpx.AsyncClient(transport=transport, base_url="http://t") as c:
        conv = (await c.post("/v1/conversations", headers=hdr(), json={"agent_id": "support_agent"})).json()
        out = parse_sse((await c.post(f"/v1/conversations/{conv['id']}/messages", headers=hdr(bearer="tok-S"),
                                      json={"content": "c'est quoi Odyssey 2026 ?"})).text)
        hist = (await c.get(f"/v1/conversations/{conv['id']}", headers=hdr())).json()
    kinds = [x[1] for x in LOG]
    assert kinds.index("/api/v1/query/context") < kinds.index("support_writer") and kinds.count("support_writer") == 1
    types_ = [e["type"] for e in out]
    assert types_.index("sources") < types_.index("delta") and "tool_start" not in types_
    assert next(e for e in out if e["type"] == "sources")["items"][0]["ref"] == 1
    (bot,) = [m for m in hist["messages"] if m["role"] == "assistant"]
    assert bot["author"] == "support_agent" and bot["content"] == "Odyssey [1]."
    assert bot["steps"][0]["status"] == "done" and bot["steps"][0]["sources_count"] == 2 and len(bot["sources"]) == 2
