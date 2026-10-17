"""Callbacks shared by every agent (defence in depth on top of tool_filter).

    LlmAgent(..., **guardrails.CALLBACKS)

- guard_tool (before_tool_callback): refuses any EdgeQuake tool that is not in
  READ_TOOLS (write/delete/ingest/upload, even if a toolset was misconfigured)
  and pins the workspace (drops a model-supplied workspace_id).
- count_llm_call (before_model_callback): per-session LLM-call budget
  (EQ_SESSION_LLM_BUDGET, default 150), kept in session state "llm_calls".
- log_llm_response (after_model_callback): one log line per completed LLM call.
"""

from __future__ import annotations

import logging
import os
from typing import Any

from google.adk.models import LlmResponse
from google.genai import types

from .mcp import READ_TOOLS, pin_workspace

log = logging.getLogger("edgequake_agents")
EQ_PREFIXES = ("eq_", "edgequake_")


def guard_tool(tool, args: dict[str, Any], tool_context) -> dict | None:
    if tool.name.startswith(EQ_PREFIXES) and tool.name not in READ_TOOLS:
        log.warning("blocked EdgeQuake tool %s (agent=%s)", tool.name, tool_context.agent_name)
        return {"status": "error", "error": f"Tool {tool.name} is not allowed: agents are read-only."}
    if "workspace_id" in args:
        log.warning("dropped model-supplied workspace_id for %s", tool.name)
        pin_workspace(tool, args, tool_context)
    return None


def session_budget() -> int:
    return int(os.getenv("EQ_SESSION_LLM_BUDGET", "150"))


def count_llm_call(callback_context, llm_request) -> LlmResponse | None:
    # ponytail: read-modify-write on session state; parallel branches may undercount
    # by a few calls. Move to a plugin with a lock if the budget must be exact.
    used = callback_context.state.get("llm_calls", 0)
    if used >= session_budget():
        log.warning("LLM budget exhausted (%d calls, agent=%s)", used, callback_context.agent_name)
        return LlmResponse(content=types.Content(role="model", parts=[types.Part(
            text="This session has used its LLM budget. Please start a new session.")]))
    callback_context.state["llm_calls"] = used + 1
    return None


def log_llm_response(callback_context, llm_response) -> None:
    if llm_response.partial:
        return None
    u = llm_response.usage_metadata
    log.info("llm agent=%s call=%s tokens_in=%s tokens_out=%s", callback_context.agent_name,
             callback_context.state.get("llm_calls"), u and u.prompt_token_count, u and u.candidates_token_count)
    return None


CALLBACKS = dict(before_tool_callback=guard_tool, before_model_callback=count_llm_call,
                 after_model_callback=log_llm_response)
