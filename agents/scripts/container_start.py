"""Container entrypoint: ensure a read-only EdgeQuake API key, then serve the agents.

The key is created once with the bootstrap admin (EDGEQUAKE_ADMIN_USERNAME/PASSWORD)
and kept in /data/api_key, so restarts never pile up keys. An explicit
EDGEQUAKE_API_KEY always wins. The admin password is dropped from the environment
before any agent code runs.

The /v1 product API (web UI) is enabled only when AGENTS_INTERNAL_TOKEN is set;
its data (agents.db, sessions.db, artifacts/) lives in EQ_DATA_DIR (/data).
"""

import os
import sys
import time
from pathlib import Path

import httpx

KEY_FILE = Path(os.getenv("EQ_KEY_FILE", "/data/api_key"))


def wait_ready(base: str) -> None:
    for _ in range(120):
        try:
            if httpx.get(f"{base}/health", timeout=5).status_code == 200:
                return
        except httpx.HTTPError:
            pass
        time.sleep(5)
    sys.exit("EdgeQuake API never became healthy")


def create_key(base: str) -> str:
    login = httpx.post(
        f"{base}/api/v1/auth/login",
        json={
            "username": os.environ.get("EDGEQUAKE_ADMIN_USERNAME", "admin"),
            "password": os.environ["EDGEQUAKE_ADMIN_PASSWORD"],
        },
        timeout=30,
    )
    login.raise_for_status()
    created = httpx.post(
        f"{base}/api/v1/api-keys",
        headers={"Authorization": f"Bearer {login.json()['access_token']}"},
        json={"name": "adk-agents", "scopes": ["edgequake:read", "edgequake:query"]},
        timeout=30,
    )
    created.raise_for_status()
    body = created.json()
    return body.get("api_key") or body.get("key")


def main() -> None:
    base = os.getenv("EDGEQUAKE_URL", "http://api:8080").rstrip("/")
    if not os.getenv("EDGEQUAKE_API_KEY"):
        if not KEY_FILE.exists():
            wait_ready(base)
            KEY_FILE.write_text(create_key(base))
            KEY_FILE.chmod(0o600)
            print("created read-only EdgeQuake API key 'adk-agents'")
        os.environ["EDGEQUAKE_API_KEY"] = KEY_FILE.read_text().strip()
    os.environ.pop("EDGEQUAKE_ADMIN_PASSWORD", None)
    os.environ.setdefault("EQ_DATA_DIR", str(KEY_FILE.parent))
    if not os.getenv("AGENTS_INTERNAL_TOKEN"):
        print("AGENTS_INTERNAL_TOKEN is not set: /v1 (web UI agents API) answers 503")

    sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
    import uvicorn
    import server

    uvicorn.run(server.app, host=server.HOST, port=server.PORT, proxy_headers=True, forwarded_allow_ips="*")


if __name__ == "__main__":
    main()
