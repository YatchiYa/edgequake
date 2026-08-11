#!/usr/bin/env python3
"""SPEC-115 Arms A/B — wrap EdgeQuake example `spec115_mistral_ingest`.

Requires MISTRAL_API_KEY. Uses in-memory library path (no Postgres).
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
EQ = REPO / "edgequake"


def main() -> int:
    if not os.environ.get("MISTRAL_API_KEY"):
        print("MISTRAL_API_KEY required", file=sys.stderr)
        return 2
    env = os.environ.copy()
    env.setdefault("MISTRAL_MODEL", "mistral-small-latest")
    env.setdefault("MISTRAL_EMBEDDING_MODEL", "mistral-embed")
    env.setdefault("RUST_LOG", "info")
    cmd = [
        "cargo",
        "run",
        "--example",
        "spec115_mistral_ingest",
        "--release",
    ]
    print("Running:", " ".join(cmd), "in", EQ, flush=True)
    return subprocess.call(cmd, cwd=EQ, env=env)


if __name__ == "__main__":
    raise SystemExit(main())
