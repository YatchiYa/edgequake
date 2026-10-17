"""Custom chat UI + ADK API server for the EdgeQuake agents.

    uv run python server.py            # http://127.0.0.1:8765

Serves ui/index.html at / and the standard ADK REST API (/list-apps,
/apps/.../sessions, /run_sse, artifacts) from get_fast_api_app, plus the product
API for the web UI at /v1 (edgequake_agents/common/v1.py, SPEC-165).

SECURITY: the legacy ADK routes have NO end-user authentication. Anyone who can
reach the port can chat with the agents (and spend LLM credits) using the
EdgeQuake API key in agents/.env. Keep it on 127.0.0.1; put it behind an
authenticating reverse proxy before exposing it anywhere else. /v1 requires the
X-Agents-Internal-Token shared secret and runs tools with the caller's own token.
"""

import logging
import os
from pathlib import Path

import uvicorn
from fastapi.responses import FileResponse
from google.adk.cli.fast_api import get_fast_api_app

from edgequake_agents.common.v1 import v1

logging.basicConfig(format="%(asctime)s %(levelname)s %(name)s: %(message)s")
logging.getLogger("edgequake_agents").setLevel(os.getenv("EQ_LOG_LEVEL", "INFO"))  # guardrail + LLM-call logs

HERE = Path(__file__).resolve().parent
HOST = os.getenv("EQ_UI_HOST", "127.0.0.1")
PORT = int(os.getenv("EQ_UI_PORT", "8765"))

app = get_fast_api_app(
    agents_dir=str(HERE / "edgequake_agents"),
    web=False,  # no bundled ADK dev UI; ours is served below
    host=HOST,
    port=PORT,
    # Sessions + artifacts persist under edgequake_agents/<agent>/.adk/ (ADK default).
    max_llm_calls=int(os.getenv("EQ_MAX_LLM_CALLS", "25")),  # per run, caps runaway tool loops
)
app.mount("/v1", v1)


@app.get("/", include_in_schema=False)
async def index() -> FileResponse:
    return FileResponse(HERE / "ui" / "index.html")


if __name__ == "__main__":
    uvicorn.run(app, host=HOST, port=PORT)
