"""Create a read+query EdgeQuake API key for the agents and write agents/.env.

Reads the bootstrap admin credentials (and MISTRAL_API_KEY) from the repo-root
.env. Never prints secrets. Re-running is a no-op unless --force is passed
(a new key is created each time; revoke old ones in EdgeQuake if needed).

    uv run python scripts/bootstrap_key.py [--tenant ID] [--workspace ID] [--force]
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

import httpx
from dotenv import dotenv_values

AGENTS_DIR = Path(__file__).resolve().parents[1]
ROOT_ENV = AGENTS_DIR.parent / ".env"
AGENTS_ENV = AGENTS_DIR / ".env"
MPW_TENANT = "f62e6a67-9a75-4654-b2b6-1a6168bda5ba"
MPW_WORKSPACE = "55a78878-845c-4abe-a8ba-8ceb9d067b90"


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--url", default="http://localhost:8090")
    p.add_argument("--tenant", default=MPW_TENANT)
    p.add_argument("--workspace", default=MPW_WORKSPACE)
    p.add_argument("--force", action="store_true", help="create a new key even if agents/.env has one")
    args = p.parse_args()

    existing = dotenv_values(AGENTS_ENV) if AGENTS_ENV.exists() else {}
    if existing.get("EDGEQUAKE_API_KEY") and not args.force:
        print(f"{AGENTS_ENV} already has EDGEQUAKE_API_KEY; use --force to replace it.")
        return 0

    root = dotenv_values(ROOT_ENV)
    user = root.get("EDGEQUAKE_BOOTSTRAP_ADMIN_USERNAME")
    password = root.get("EDGEQUAKE_BOOTSTRAP_ADMIN_PASSWORD")
    if not user or not password:
        print(f"Admin credentials missing in {ROOT_ENV}", file=sys.stderr)
        return 1

    with httpx.Client(base_url=args.url, timeout=30) as c:
        r = c.post("/api/v1/auth/login", json={"username": user, "password": password})
        if r.status_code != 200:
            print(f"Login failed: HTTP {r.status_code}", file=sys.stderr)
            return 1
        # API keys belong to the user (not a tenant); the workspace is chosen
        # per request with X-Tenant-ID / X-Workspace-ID.
        headers = {"Authorization": f"Bearer {r.json()['access_token']}"}
        r = c.post("/api/v1/api-keys", headers=headers, json={"name": "adk-agents", "scopes": ["read", "query"]})
        if r.status_code not in (200, 201):
            print(f"API key creation failed: HTTP {r.status_code} {r.text[:200]}", file=sys.stderr)
            return 1
        key = r.json()

    values = {
        "EDGEQUAKE_URL": args.url,
        "EDGEQUAKE_API_KEY": key["api_key"],
        "EDGEQUAKE_TENANT_ID": args.tenant,
        "EDGEQUAKE_WORKSPACE_ID": args.workspace,
        "MISTRAL_API_KEY": root.get("MISTRAL_API_KEY") or "",
    }
    # Keep any other settings the user added (e.g. EQ_AGENT_MODEL).
    merged = {**{k: v for k, v in existing.items() if v is not None}, **values}
    AGENTS_ENV.write_text("".join(f"{k}={v}\n" for k, v in merged.items()))
    AGENTS_ENV.chmod(0o600)
    print(f"Created key {key['prefix']}… (id {key['key_id']}, scopes {key['scopes']}); wrote {AGENTS_ENV}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
