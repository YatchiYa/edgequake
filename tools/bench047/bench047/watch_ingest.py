"""Live ingest telemetry watcher for SPEC-047 soft-resume runs.

Polls document/PDF status and tails backend log for P6 markers
(`unique-before-embed`, parallel sub-batches, single-flight).

Usage:
  python -m bench047.watch_ingest --workspace <uuid> [--log /tmp/edgequake-backend.log]
"""

from __future__ import annotations

import argparse
import re
import time
from pathlib import Path
from typing import Any

from .client import EdgeQuakeClient

P6_PATTERNS = (
    re.compile(r"unique-before-embed", re.I),
    re.compile(r"unique_entities=\d+", re.I),
    re.compile(r"unique_relationships=\d+", re.I),
    re.compile(r"Embedding sub-batches in parallel", re.I),
    re.compile(r"Single-flight:", re.I),
    re.compile(r"Entity embed:", re.I),
    re.compile(r"Relationship embed:", re.I),
    re.compile(r"Merger: starting global batch merge", re.I),
)


def _fmt_doc(d: dict[str, Any]) -> str:
    stage = d.get("current_stage") or ""
    prog = d.get("stage_progress")
    msg = (d.get("stage_message") or d.get("warning_message") or "")[:120]
    bits = [
        f"status={d.get('status')}",
        f"stage={stage}" if stage else "",
        f"progress={float(prog):.0%}" if prog is not None else "",
        f"chunks={d.get('chunk_count')}",
        f"ents={d.get('entity_count')}",
    ]
    line = " ".join(b for b in bits if b)
    if msg:
        line += f"\n      {msg}"
    return line


def watch(
    *,
    api: str,
    workspace_id: str,
    log_path: Path,
    interval_s: float = 15.0,
) -> None:
    client = EdgeQuakeClient(base_url=api, workspace_id=workspace_id)
    log_offset = log_path.stat().st_size if log_path.exists() else 0
    print(f"watch_ingest workspace={workspace_id} log={log_path} interval={interval_s}s")
    print("P6 markers: unique-before-embed | parallel sub-batches | Single-flight")
    while True:
        try:
            r = client.client.get(
                f"{client.base}/api/v1/documents",
                headers=client.headers(),
                params={"page_size": 50},
            )
            r.raise_for_status()
            data = r.json()
            docs = data.get("documents") or []
            counts = data.get("status_counts") or {}
            print(f"\n[{time.strftime('%H:%M:%S')}] status_counts={counts}")
            for d in docs:
                name = (d.get("file_name") or d.get("title") or d.get("id") or "")[:48]
                print(f"  {name}: {_fmt_doc(d)}")
        except Exception as e:
            print(f"  poll err: {e}")

        if log_path.exists():
            with log_path.open("r", errors="replace") as f:
                f.seek(log_offset)
                chunk = f.read()
                log_offset = f.tell()
            for line in chunk.splitlines():
                if any(p.search(line) for p in P6_PATTERNS):
                    # Strip ANSI / truncate
                    clean = re.sub(r"\x1b\[[0-9;]*m", "", line)[-220:]
                    print(f"  LOG {clean}")

        time.sleep(interval_s)


def main() -> None:
    p = argparse.ArgumentParser(description="Watch SPEC-047 ingest telemetry")
    p.add_argument("--api", default="http://127.0.0.1:8090")
    p.add_argument("--workspace", required=True)
    p.add_argument("--log", default="/tmp/edgequake-backend.log")
    p.add_argument("--interval", type=float, default=15.0)
    args = p.parse_args()
    watch(
        api=args.api,
        workspace_id=args.workspace,
        log_path=Path(args.log),
        interval_s=args.interval,
    )


if __name__ == "__main__":
    main()
