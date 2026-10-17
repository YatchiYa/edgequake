"""Environment-driven settings shared by every agent."""
# No __init__.py on purpose: ADK lists every package under edgequake_agents/ as an
# agent, so this shared code lives in a namespace package.

from __future__ import annotations

import os
from contextvars import ContextVar
from dataclasses import dataclass
from pathlib import Path

from dotenv import load_dotenv

# agents/.env — explicit env vars always win over the file.
load_dotenv(Path(__file__).resolve().parents[2] / ".env", override=False)

DEFAULT_MODEL = "mistral/mistral-medium-latest"


@dataclass(frozen=True)
class Settings:
    url: str
    api_key: str
    tenant_id: str
    workspace_id: str
    model: str

    @property
    def headers(self) -> dict[str, str]:
        """Auth + workspace scoping headers, valid for both REST and /mcp."""
        h = {"Authorization": f"Bearer {self.api_key}"} if self.api_key else {}
        if self.tenant_id:
            h["X-Tenant-ID"] = self.tenant_id
        if self.workspace_id:
            h["X-Workspace-ID"] = self.workspace_id
        return h


def settings() -> Settings:
    return Settings(
        url=os.getenv("EDGEQUAKE_URL", "http://localhost:8090").rstrip("/"),
        api_key=os.getenv("EDGEQUAKE_API_KEY", ""),
        tenant_id=os.getenv("EDGEQUAKE_TENANT_ID", ""),
        workspace_id=os.getenv("EDGEQUAKE_WORKSPACE_ID", ""),
        model=os.getenv("EQ_AGENT_MODEL", DEFAULT_MODEL),
    )


# Per-request EdgeQuake credentials (the /v1 API sets them around each run: the
# signed-in user's bearer + scope). Unset = legacy ADK routes -> configured API key.
# Lives only in memory for the duration of the run; never written to state/DB/logs.
request_headers: ContextVar[dict[str, str] | None] = ContextVar("eq_request_headers", default=None)


def data_dir() -> Path:
    """EQ_DATA_DIR (/data in the image), default agents/.data. Created on demand."""
    d = Path(os.getenv("EQ_DATA_DIR") or Path(__file__).resolve().parents[2] / ".data")
    d.mkdir(parents=True, exist_ok=True)
    return d


def eq_headers() -> dict[str, str]:
    """Headers every EdgeQuake call (MCP + REST) must use right now."""
    h = request_headers.get()
    return dict(h) if h is not None else settings().headers


def model(name: str | None = None, *, agent: str | None = None):
    """ADK model for `name`, else EQ_MODEL_<AGENT> (e.g. EQ_MODEL_SUPPORT_AGENT),
    else EQ_AGENT_MODEL, else mistral/mistral-medium-latest.

    Bare Gemini ids ("gemini-2.5-flash") use ADK's native Gemini client — AI
    Studio or Vertex AI depending on GOOGLE_GENAI_USE_VERTEXAI. Anything else is
    treated as a LiteLLM id ("mistral/mistral-medium-latest", "openai/gpt-4o",
    "vertex_ai/gemini-2.5-pro", ...).
    """
    name = name or (agent and os.getenv(f"EQ_MODEL_{agent.upper()}")) or settings().model
    if name.startswith("gemini") and "/" not in name:
        return name
    from google.adk.models.lite_llm import LiteLlm

    return LiteLlm(model=name)
