"""Product API for the web UI (SPEC-165), mounted at /v1 by server.py.

Only reachable through the Next.js proxy: every request carries the shared secret
X-Agents-Internal-Token (env AGENTS_INTERNAL_TOKEN; unset = /v1 disabled, 503) and
the caller identity X-EQ-User-Id. Runs use the forwarded `Authorization` bearer +
X-Tenant-ID / X-Workspace-ID for every EdgeQuake call (config.request_headers);
tokens live only in memory for the duration of the run.

Persistence (EQ_DATA_DIR, default agents/.data, /data in the image):
  agents.db     custom agents + conversation index (store.py)
  sessions.db   ADK sessions/events (SqliteSessionService), app_name = agent id
                ("custom-<id>" for custom agents), user_id = X-EQ-User-Id, session id = conversation id
  artifacts/    ADK FileArtifactService (bytes + mime type per version)
"""

from __future__ import annotations

import asyncio
import hmac
import json
import logging
import os
import time
from collections import defaultdict, deque
from contextlib import aclosing, suppress
from dataclasses import dataclass
from datetime import datetime, timezone
from functools import lru_cache
from pathlib import Path
from typing import Literal, Optional

from fastapi import Depends, FastAPI, Request
from fastapi.exceptions import RequestValidationError
from fastapi.responses import JSONResponse, Response, StreamingResponse
from google.adk.agents.run_config import RunConfig, StreamingMode
from google.adk.artifacts import FileArtifactService
from google.adk.runners import Runner
from google.adk.sessions.sqlite_session_service import SqliteSessionService
from google.genai import types
from pydantic import BaseModel, Field, ValidationError, field_validator
from starlette.exceptions import HTTPException

from . import architect, catalog, knowledge
from .config import data_dir, request_headers
from .store import Store

log = logging.getLogger("edgequake_agents")
MAX_MESSAGE_BYTES = 32 * 1024
MAX_BODY_BYTES = 256 * 1024
RUNS_PER_MINUTE = 20
NEW_TITLE = "New chat"


# ---------------------------------------------------------------- services


@dataclass
class Services:
    store: Store
    sessions: SqliteSessionService
    artifacts: FileArtifactService


@lru_cache(maxsize=1)
def services() -> Services:
    data = data_dir()
    return Services(Store(data / "agents.db"), SqliteSessionService(str(data / "sessions.db")),
                    FileArtifactService(data / "artifacts"))


# ---------------------------------------------------------------- identity


@dataclass(frozen=True)
class Caller:
    user_id: str
    workspace_id: str | None
    tenant_id: str | None
    eq_headers: dict  # Authorization + scope, forwarded to EdgeQuake tools. Never persisted.


def err(status: int, message: str):
    return HTTPException(status, message)


async def caller(request: Request) -> Caller:
    expected = os.getenv("AGENTS_INTERNAL_TOKEN", "")
    if not expected:
        raise err(503, "agents API disabled: AGENTS_INTERNAL_TOKEN is not set")
    got = request.headers.get("x-agents-internal-token", "")
    if not hmac.compare_digest(got.encode(), expected.encode()):
        raise err(401, "invalid internal token")
    user = request.headers.get("x-eq-user-id", "").strip()
    if not user:
        raise err(401, "missing X-EQ-User-Id")
    if int(request.headers.get("content-length") or 0) > MAX_BODY_BYTES:
        raise err(413, "request too large")
    h = request.headers
    eq = {k: h[k.lower()] for k in ("Authorization", "X-Tenant-ID", "X-Workspace-ID") if h.get(k.lower())}
    return Caller(user, h.get("x-workspace-id") or None, h.get("x-tenant-id") or None, eq)


v1 = FastAPI(title="EdgeQuake agents API", dependencies=[Depends(caller)], docs_url=None, redoc_url=None)


@v1.exception_handler(HTTPException)
async def _http_error(_req, e: HTTPException):
    return JSONResponse({"error": e.detail}, status_code=e.status_code)


@v1.exception_handler(RequestValidationError)
@v1.exception_handler(ValidationError)
async def _validation_error(_req, e):
    msg = "; ".join(f"{'.'.join(str(p) for p in x['loc'] if p != 'body')}: {x['msg']}" for x in e.errors())
    return JSONResponse({"error": msg or "invalid request"}, status_code=400)


# ---------------------------------------------------------------- agents


def visible_agent(agent_id: str, who: Caller) -> dict:
    if agent_id in catalog.BUILTINS:
        return catalog.BUILTINS[agent_id]
    a = services().store.get_agent(agent_id)
    if a and ((a["visibility"] == "private" and a["owner_id"] == who.user_id)
              or (a["visibility"] == "workspace" and a["workspace_id"] and a["workspace_id"] == who.workspace_id)):
        return a
    raise err(404, "agent not found")


def agent_json(a: dict, who: Caller) -> dict:
    return {**catalog.public(a), "editable": a["kind"] == "custom" and a["owner_id"] == who.user_id}


class AgentIn(BaseModel):
    name: str = Field(min_length=1, max_length=60)
    description: Optional[str] = Field("", max_length=1000)
    icon: Optional[str] = Field("🤖", max_length=16)
    instructions: str = Field(max_length=20_000)
    model: str
    tools: list[str]
    starters: list[str] = Field(default_factory=list, max_length=4)
    temperature: float = Field(0.2, ge=0, le=1)
    visibility: Literal["private", "workspace"] = "private"

    @field_validator("name")
    @classmethod
    def _name(cls, v):
        if not v.strip():
            raise ValueError("name is empty")
        return v.strip()

    @field_validator("model")
    @classmethod
    def _model(cls, v):
        if v not in {m["id"] for m in catalog.allowed_models()}:
            raise ValueError(f"unknown model {v!r}; see /v1/models")
        return v

    @field_validator("tools")
    @classmethod
    def _tools(cls, v):
        unknown = sorted(set(v) - catalog.TOOLS.keys())
        if unknown:
            raise ValueError(f"unknown tools {unknown}; see /v1/tools")
        return list(dict.fromkeys(v))

    @field_validator("starters")
    @classmethod
    def _starters(cls, v):
        if any(len(s) > 200 for s in v):
            raise ValueError("each starter is at most 200 characters")
        return [s.strip() for s in v if s.strip()]

    def fields(self, who: Caller) -> dict:
        d = self.model_dump()
        d["description"], d["icon"] = d["description"] or "", d["icon"] or "🤖"
        if self.visibility == "workspace" and not who.workspace_id:
            raise err(400, "visibility 'workspace' needs a current workspace (X-Workspace-ID)")
        d["workspace_id"] = who.workspace_id if self.visibility == "workspace" else None
        return d


@v1.get("/agents")
async def list_agents(include: str | None = None, who: Caller = Depends(caller)):
    """System agents (the architect) only with ?include=system."""
    customs = services().store.list_agents(who.user_id, who.workspace_id)
    builtins = [a for a in catalog.BUILTINS.values() if a["kind"] == "builtin" or include == "system"]
    return {"items": [agent_json(a, who) for a in [*builtins, *customs]]}


@v1.get("/agents/{agent_id}")
async def get_agent(agent_id: str, who: Caller = Depends(caller)):
    return agent_json(visible_agent(agent_id, who), who)


@v1.post("/agents", status_code=201)
async def create_agent(body: AgentIn, who: Caller = Depends(caller)):
    return agent_json(services().store.save_agent(body.fields(who), owner_id=who.user_id), who)


def owned_agent(agent_id: str, who: Caller) -> dict:
    a = visible_agent(agent_id, who)
    if a["kind"] != "custom" or a["owner_id"] != who.user_id:
        raise err(403, "only the owner can modify this agent")
    return a


@v1.put("/agents/{agent_id}")
async def update_agent(agent_id: str, body: AgentIn, who: Caller = Depends(caller)):
    owned_agent(agent_id, who)
    return agent_json(services().store.save_agent(body.fields(who), agent_id=agent_id), who)


@v1.delete("/agents/{agent_id}", status_code=204)
async def delete_agent(agent_id: str, who: Caller = Depends(caller)):
    owned_agent(agent_id, who)
    services().store.delete_agent(agent_id)
    return Response(status_code=204)


@v1.post("/agents/{agent_id}/duplicate", status_code=201)
async def duplicate_agent(agent_id: str, who: Caller = Depends(caller)):
    a = visible_agent(agent_id, who)
    if a["kind"] == "system":
        raise err(400, "system agents cannot be duplicated")
    allowed = {m["id"] for m in catalog.allowed_models()}
    copy = AgentIn(
        name=(a["name"][:53] + " (copy)"), description=a["description"][:1000], icon=a["icon"],
        instructions=a["instructions"][:20_000], model=a["model"] if a["model"] in allowed else catalog.default_model(),
        tools=a["tools"], starters=a["starters"], temperature=a["temperature"], visibility="private",
    )
    return agent_json(services().store.save_agent(copy.fields(who), owner_id=who.user_id), who)


@v1.get("/tools")
async def list_tools():
    return {"items": [t.public() for t in catalog.TOOLS.values()]}


@v1.get("/models")
async def list_models():
    return {"items": catalog.allowed_models()}


# ---------------------------------------------------------------- conversations


def conv_json(c: dict) -> dict:
    return {k: c[k] for k in ("id", "agent_id", "title", "created_at", "updated_at")}


def owned_conversation(conv_id: str, who: Caller) -> dict:
    """The caller's conversation in the caller's CURRENT workspace; anything else is 404
    (another user's or another workspace's: no existence leak)."""
    c = services().store.get_conversation(who.user_id, conv_id, who.tenant_id, who.workspace_id)
    if not c:
        raise err(404, "conversation not found")
    return c


@v1.get("/conversations")
async def list_conversations(agent_id: str | None = None, q: str | None = None, limit: int = 50,
                             who: Caller = Depends(caller)):
    rows = services().store.list_conversations(who.user_id, agent_id, q, min(max(limit, 1), 200),
                                               who.tenant_id, who.workspace_id)
    return {"items": [conv_json(c) for c in rows]}


class ConversationIn(BaseModel):
    agent_id: str


@v1.post("/conversations", status_code=201)
async def create_conversation(body: ConversationIn, who: Caller = Depends(caller)):
    visible_agent(body.agent_id, who)
    s = services()
    conv = s.store.create_conversation(who.user_id, body.agent_id, who.tenant_id, who.workspace_id)
    await s.sessions.create_session(app_name=catalog.app_name(body.agent_id), user_id=who.user_id, session_id=conv["id"])
    return conv_json(conv)


async def _mime(app: str, user: str, session: str, name: str, version: int) -> str:
    v = await services().artifacts.get_artifact_version(app_name=app, user_id=user, session_id=session,
                                                        filename=name, version=version)
    return (v and v.mime_type) or "application/octet-stream"


def _iso(ts: float) -> str:
    return datetime.fromtimestamp(ts, timezone.utc).isoformat()


# Internal sub-agents whose text is plumbing, not part of the answer (the planner's JSON plan).
HIDDEN_AUTHORS = {"planner"}


def _text(event) -> str:
    if event.author in HIDDEN_AUTHORS:
        return ""
    parts = (event.content and event.content.parts) or []
    return "".join(p.text for p in parts if p.text and not p.thought and not p.text.startswith(architect.CONTEXT_MARK))


def _ui_events(event) -> list[dict]:
    """UI events an agent attached to this event (investigator: plan/step/sources), plus a
    `sources` event for every successful knowledge_query/knowledge_context response."""
    out = [] if event.partial else list((event.custom_metadata or {}).get("eq_ui") or [])
    for r in [] if event.partial else event.get_function_responses():
        if r.name in knowledge.TOOL_NAMES and isinstance(r.response, dict) and r.response.get("sources"):
            out.append({"type": "sources", "items": knowledge.ui_sources(r.response["sources"], r.response.get("query", ""))})
    return out


def fold_ui(msg: dict, e: dict) -> None:
    """Applies one plan/step/sources event to a history message (same rules as the web reducer)."""
    if e["type"] == "plan":
        msg["plan"] = e["items"]
    elif e["type"] == "step":
        step = {k: v for k, v in e.items() if k != "type"}
        steps = msg.setdefault("steps", [])
        i = next((i for i, x in enumerate(steps) if x["id"] == step["id"]), None)
        if i is None:
            steps.append(step)
        else:
            steps[i] = {**steps[i], **step}
    elif e["type"] == "sources":
        have = msg.setdefault("sources", [])
        keys = {(x.get("chunk_id"), x.get("document_id")) for x in have}
        for x in e["items"]:
            if (k := (x.get("chunk_id"), x.get("document_id"))) not in keys:
                keys.add(k)
                have.append(x)


def _tool_ok(response) -> bool:
    r = response.response
    return not (isinstance(r, dict) and ("error" in r or r.get("status") == "error"))


def _agent_spec(response) -> dict | None:
    """A successful propose_agent_spec call -> the `agent_spec` payload."""
    r = response.response
    if response.name == architect.TOOL_NAME and isinstance(r, dict) and r.get("status") == "ok":
        return {"spec": r.get("spec"), "rationale": r.get("rationale", ""), "tests": r.get("tests", [])}
    return None


async def build_messages(events, mime=None) -> list[dict]:
    """Session events -> chat messages: one per user turn and ONE assistant message per
    turn, whatever the number of (sub-)agents involved: `author` is the first agent that
    produced something, `agents` every agent in order; text joined, tools from
    function_call/response, artifacts from artifact_delta, plan/steps/sources from UI
    events. `mime(name, version)` resolves artifact mime types."""
    messages: list[dict] = []
    cur, tools = None, {}
    for ev in events:
        if ev.partial:
            continue
        if ev.author == "user":
            cur = None
            if text := _text(ev):
                messages.append({"id": ev.id, "role": "user", "author": "user", "content": text, "tools": [],
                                 "artifacts": [], "created_at": _iso(ev.timestamp)})
            continue
        calls, responses = ev.get_function_calls(), ev.get_function_responses()
        text, delta, ui = _text(ev), ev.actions.artifact_delta or {}, _ui_events(ev)
        if not (text or calls or responses or delta or ui):
            continue
        if cur is None:
            cur = {"id": ev.id, "role": "assistant", "author": ev.author, "agents": [], "content": "", "tools": [],
                   "artifacts": [], "created_at": _iso(ev.timestamp)}
            messages.append(cur)
        if ev.author not in cur["agents"]:
            cur["agents"].append(ev.author)
        if text:
            cur["content"] = f"{cur['content']}\n\n{text}" if cur["content"] else text
        for c in calls:
            tools[c.id] = {"id": c.id, "name": c.name, "status": "done"}
            cur["tools"].append(tools[c.id])
        for r in responses:
            if r.id in tools and not _tool_ok(r):
                tools[r.id]["status"] = "error"
            if proposal := _agent_spec(r):
                cur["agent_spec"] = proposal  # latest proposal of this turn (architect only)
        for e in ui:
            fold_ui(cur, e)
        for name, version in delta.items():
            cur["artifacts"].append({"name": name, "mime_type": await mime(name, version) if mime else None,
                                     "version": version})
    return messages


@v1.get("/conversations/{conv_id}")
async def get_conversation(conv_id: str, who: Caller = Depends(caller)):
    c = owned_conversation(conv_id, who)
    app = catalog.app_name(c["agent_id"])
    session = await services().sessions.get_session(app_name=app, user_id=who.user_id, session_id=conv_id)
    events = session.events if session else []
    msgs = await build_messages(events, lambda n, v: _mime(app, who.user_id, conv_id, n, v))
    return {**conv_json(c), "messages": msgs}


class TitleIn(BaseModel):
    title: str = Field(min_length=1, max_length=200)


@v1.patch("/conversations/{conv_id}")
async def rename_conversation(conv_id: str, body: TitleIn, who: Caller = Depends(caller)):
    owned_conversation(conv_id, who)
    if not body.title.strip():
        raise err(400, "title is empty")
    services().store.update_conversation(conv_id, title=body.title.strip())
    return conv_json(owned_conversation(conv_id, who))


@v1.delete("/conversations/{conv_id}", status_code=204)
async def delete_conversation(conv_id: str, who: Caller = Depends(caller)):
    c = owned_conversation(conv_id, who)
    s, app = services(), catalog.app_name(c["agent_id"])
    for name in await s.artifacts.list_artifact_keys(app_name=app, user_id=who.user_id, session_id=conv_id):
        await s.artifacts.delete_artifact(app_name=app, user_id=who.user_id, session_id=conv_id, filename=name)
    with suppress(Exception):  # session may never have been created
        await s.sessions.delete_session(app_name=app, user_id=who.user_id, session_id=conv_id)
    s.store.delete_conversation(conv_id)
    return Response(status_code=204)


@v1.get("/conversations/{conv_id}/artifacts/{name:path}")
async def get_artifact(conv_id: str, name: str, version: int | None = None, who: Caller = Depends(caller)):
    c = owned_conversation(conv_id, who)
    part = await services().artifacts.load_artifact(app_name=catalog.app_name(c["agent_id"]), user_id=who.user_id,
                                                    session_id=conv_id, filename=name, version=version)
    if not part or not part.inline_data:
        raise err(404, "artifact not found")
    return Response(part.inline_data.data, media_type=part.inline_data.mime_type or "application/octet-stream")


# ---------------------------------------------------------------- runs (SSE)

_runs: dict[str, deque] = defaultdict(deque)  # ponytail: in-process limiter, per replica
_active: set[str] = set()


def rate_limited(user_id: str) -> bool:
    q, now = _runs[user_id], time.monotonic()
    while q and now - q[0] > 60:
        q.popleft()
    if len(q) >= RUNS_PER_MINUTE:
        return True
    q.append(now)
    return False


class ContextIn(BaseModel):
    """Builder form state for the architect (data, not instructions)."""
    draft_agent: Optional[dict] = None
    editing_agent_id: Optional[str] = Field(None, max_length=100)


class MessageIn(BaseModel):
    content: str = Field(min_length=1)
    context: Optional[ContextIn] = None


def sse(obj: dict) -> str:
    return f"data: {json.dumps(obj, ensure_ascii=False)}\n\n"


async def translate(events, mime):
    """ADK events (SSE streaming mode) -> SPEC-165 stream objects."""
    current, streamed = None, set()
    async for ev in events:
        if ev.author == "user":
            continue
        text = _text(ev)
        calls = [] if ev.partial else ev.get_function_calls()
        responses = [] if ev.partial else ev.get_function_responses()
        delta = {} if ev.partial else (ev.actions.artifact_delta or {})
        ui = _ui_events(ev)
        if ev.partial and text:
            streamed.add(ev.author)
        elif not ev.partial and ev.author in streamed:
            streamed.discard(ev.author)
            text = ""  # already streamed as partial deltas; this is the aggregate
        if (text or calls or responses or delta or ui) and ev.author != current:
            current = ev.author
            yield {"type": "message_start", "author": current}
        if text:
            yield {"type": "delta", "author": ev.author, "text": text}
        for c in calls:
            yield {"type": "tool_start", "id": c.id, "author": ev.author, "name": c.name, "args": c.args or {}}
        for r in responses:
            yield {"type": "tool_end", "id": r.id, "ok": _tool_ok(r)}
            if proposal := _agent_spec(r):
                yield {"type": "agent_spec", **proposal}
        for e in ui:
            yield e
        for name, version in delta.items():
            yield {"type": "artifact", "name": name, "mime_type": await mime(name, version), "version": version}
        if ev.error_message and not ev.partial:
            yield {"type": "error", "message": ev.error_message}


@v1.post("/conversations/{conv_id}/messages")
async def post_message(conv_id: str, request: Request, who: Caller = Depends(caller)):
    raw = await request.body()
    if len(raw) > MAX_MESSAGE_BYTES:
        raise err(413, f"message too large (max {MAX_MESSAGE_BYTES // 1024} KB)")
    try:
        body = MessageIn.model_validate_json(raw or b"{}")
    except ValidationError as e:
        raise err(400, "body must be {\"content\": \"...\", \"context\"?: {...}} with non-empty content") from e
    content = body.content.strip()
    if not content:
        raise err(400, "content is empty")
    c = owned_conversation(conv_id, who)
    spec = visible_agent(c["agent_id"], who)
    extra = None
    if body.context and spec["id"] == architect.AGENT_ID:  # other agents ignore the builder context
        if body.context.editing_agent_id:
            visible_agent(body.context.editing_agent_id, who)  # 404 if the caller cannot see it
        try:
            extra = architect.context_part(body.context.draft_agent, body.context.editing_agent_id)
        except ValidationError as e:
            raise err(400, "context.draft_agent: invalid fields") from e
    if not who.eq_headers.get("Authorization", "").lower().startswith("bearer "):
        raise err(401, "missing Authorization bearer for EdgeQuake")
    if conv_id in _active:
        raise err(409, "a reply is already being generated in this conversation")
    if rate_limited(who.user_id):
        raise err(429, f"rate limit: {RUNS_PER_MINUTE} messages per minute")

    s, app = services(), catalog.app_name(c["agent_id"])
    runner = Runner(app_name=app, agent=catalog.build_agent(spec), session_service=s.sessions,
                    artifact_service=s.artifacts, auto_create_session=True)
    run_config = RunConfig(streaming_mode=StreamingMode.SSE, max_llm_calls=int(os.getenv("EQ_MAX_LLM_CALLS", "25")))
    first = c["title"] == NEW_TITLE

    async def stream():
        _active.add(conv_id)  # here, not before: an unstarted generator never reaches `finally`
        token = request_headers.set(who.eq_headers)  # what every MCP/REST tool call of this run uses
        abort = asyncio.Event()

        async def watch_disconnect():
            while not await request.is_disconnected():
                await asyncio.sleep(0.5)
            abort.set()

        watcher = asyncio.create_task(watch_disconnect())
        designed = None
        try:
            msg = types.Content(role="user", parts=[*([extra] if extra else []), types.Part(text=content)])
            events = runner.run_async(user_id=who.user_id, session_id=conv_id, new_message=msg,
                                      run_config=run_config, abort_signal=abort)
            async with aclosing(events) as events:
                async for out in translate(events, lambda n, v: _mime(app, who.user_id, conv_id, n, v)):
                    if abort.is_set():
                        return
                    if out["type"] == "agent_spec" and (out.get("spec") or {}).get("name"):
                        designed = out["spec"]["name"]
                    yield sse(out)
            s.store.update_conversation(conv_id)
            if designed:  # architect conversations are named after the agent being designed
                title = f"Conception : {designed}"[:120]
                s.store.update_conversation(conv_id, title=title)
                yield sse({"type": "title", "title": title})
            elif first:
                title = " ".join(content.split())[:60]
                s.store.update_conversation(conv_id, title=title)
                yield sse({"type": "title", "title": title})
        except Exception as e:  # noqa: BLE001 - surfaced to the client, run is over
            log.exception("run failed (app=%s conversation=%s)", app, conv_id)
            yield sse({"type": "error", "message": f"{type(e).__name__}: {e}"[:500]})
        finally:
            watcher.cancel()
            _active.discard(conv_id)
            with suppress(ValueError):
                request_headers.reset(token)
        yield sse({"type": "done"})

    return StreamingResponse(stream(), media_type="text/event-stream",
                             headers={"Cache-Control": "no-cache", "X-Accel-Buffering": "no"})
