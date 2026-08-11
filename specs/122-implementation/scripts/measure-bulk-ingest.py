#!/usr/bin/env python3
"""SPEC-122 bulk ingest measurement harness (LAW-122-5).

Usage:
  BASE_URL=http://127.0.0.1:8090 WORKSPACE_ID=<uuid> ARM=A N=5 \\
    ./measure-bulk-ingest.py

Polls GET /api/v1/documents/{id} until terminal status.
"""
from __future__ import annotations

import json
import os
import sys
import time
import uuid
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[3]
BASE_URL = os.environ.get("BASE_URL", "http://127.0.0.1:8090").rstrip("/")
WORKSPACE_ID = os.environ.get("WORKSPACE_ID", "default")
ARM = os.environ.get("ARM", "C")
N = int(os.environ.get("N", "1"))
TIMEOUT_S = int(os.environ.get("TIMEOUT_S", "1800"))
FIXTURE_DIR = Path(os.environ.get("FIXTURE_DIR", ROOT / "zz_test_docs"))
OUT_DIR = Path(os.environ.get("OUT_DIR", ROOT / "specs/122-implementation/measurements"))
TOKEN = os.environ.get("EDGEQUAKE_TOKEN", "")
STAMP = time.strftime("%Y%m%dT%H%M%SZ", time.gmtime())
RUN_DIR = OUT_DIR / f"{STAMP}-arm{ARM}-n{N}"
RUN_DIR.mkdir(parents=True, exist_ok=True)

TERMINAL = {"completed", "failed", "cancelled", "partial_failure", "indexed", "processed"}


def headers() -> dict[str, str]:
    h = {
        "Content-Type": "application/json",
        "X-Workspace-Id": WORKSPACE_ID,
        "Accept": "application/json",
    }
    if TOKEN:
        h["Authorization"] = f"Bearer {TOKEN}"
    return h


def http_json(method: str, path: str, body: dict | None = None) -> dict:
    data = None if body is None else json.dumps(body).encode()
    req = Request(f"{BASE_URL}{path}", data=data, headers=headers(), method=method)
    try:
        with urlopen(req, timeout=120) as resp:
            raw = resp.read().decode()
            return json.loads(raw) if raw else {}
    except HTTPError as e:
        err = e.read().decode()
        raise RuntimeError(f"{method} {path} -> {e.code}: {err[:500]}") from e
    except URLError as e:
        raise RuntimeError(f"{method} {path} failed: {e}") from e


def snap_metrics(label: str) -> dict:
    try:
        m = http_json("GET", "/api/v1/pipeline/queue-metrics")
    except Exception as e:  # noqa: BLE001
        m = {"error": str(e)}
    (RUN_DIR / f"metrics-{label}.json").write_text(json.dumps(m, indent=2))
    return m


def build_fixtures() -> list[Path]:
    srcs = [
        FIXTURE_DIR / "test_injection.txt",
        FIXTURE_DIR / "test_injection.md",
        FIXTURE_DIR / "test-document.md",
    ]
    out: list[Path] = []
    for i in range(1, N + 1):
        src = srcs[(i - 1) % len(srcs)]
        dst = RUN_DIR / f"fixture-{i}.txt"
        body = (
            f"SPEC-122 ARM={ARM} N={N} INDEX={i} TS={STAMP}\n"
            f"UNIQUE={uuid.uuid4()}\n"
            + src.read_text(encoding="utf-8", errors="replace")
        )
        dst.write_text(body, encoding="utf-8")
        out.append(dst)
    return out


def admit(path: Path) -> dict:
    payload = {
        "content": path.read_text(encoding="utf-8"),
        "file_source": path.name,
        "async_processing": True,
    }
    return http_json("POST", "/api/v1/documents", payload)


def doc_status(doc_id: str) -> dict:
    return http_json("GET", f"/api/v1/documents/{doc_id}")


def main() -> int:
    print(f"SPEC-122 measure ARM={ARM} N={N} WS={WORKSPACE_ID} → {RUN_DIR}")
    try:
        health = http_json("GET", "/health")
        (RUN_DIR / "health.json").write_text(json.dumps(health, indent=2))
    except Exception as e:  # noqa: BLE001
        (RUN_DIR / "health.json").write_text(json.dumps({"error": str(e)}))

    pre = snap_metrics("pre")
    fixtures = build_fixtures()

    t0 = time.time()
    docs: list[dict] = []
    for f in fixtures:
        resp = admit(f)
        docs.append(resp)
        (RUN_DIR / "admits.jsonl").open("a").write(json.dumps(resp) + "\n")
        print(f"admitted {resp.get('document_id')} track={resp.get('track_id')}")
    admit_s = round(time.time() - t0, 3)
    snap_metrics("post-admit")

    doc_ids = [d.get("document_id") for d in docs if d.get("document_id")]
    done: dict[str, str] = {}
    first_s: float | None = None
    all_s: float | None = None
    max_proc = 0
    deadline = t0 + TIMEOUT_S

    while True:
        now = time.time()
        if now > deadline:
            (RUN_DIR / "timeout.txt").write_text(f"TIMEOUT after {TIMEOUT_S}s\n")
            print("TIMEOUT")
            break

        processing = 0
        for did in doc_ids:
            if did in done:
                continue
            st = doc_status(did)
            (RUN_DIR / "status-polls.jsonl").open("a").write(
                json.dumps({"t": now - t0, "id": did, **{k: st.get(k) for k in ("status", "display_status", "chunks_count", "chunk_count", "error")}})
                + "\n"
            )
            status = str(st.get("display_status") or st.get("status") or "").lower()
            if status in {"processing", "pending", "running", "converting"}:
                if status == "processing":
                    processing += 1
            if status in TERMINAL or status == "completed":
                # treat indexed as completed
                if status == "indexed":
                    status = "completed"
                done[did] = status
                if first_s is None:
                    first_s = round(now - t0, 3)

        max_proc = max(max_proc, processing)
        snap_metrics("mid")
        print(f"progress {len(done)}/{N} processing={processing} max_proc={max_proc}")
        if len(done) >= N:
            all_s = round(now - t0, 3)
            break
        time.sleep(2)

    final_metrics = snap_metrics("final")
    docs_per_min = None if not all_s else round((N / all_s) * 60.0, 3)
    summary = {
        "arm": ARM,
        "n": N,
        "base_url": BASE_URL,
        "workspace_id": WORKSPACE_ID,
        "stamp": STAMP,
        "admit_s": admit_s,
        "t_first_complete_s": first_s,
        "t_all_complete_s": all_s,
        "docs_per_min": docs_per_min,
        "max_concurrent_processing_observed": max_proc,
        "final_statuses": done,
        "document_ids": doc_ids,
        "pre_max_tasks_per_tenant": pre.get("max_tasks_per_tenant"),
        "final_park_waiters": final_metrics.get("tenant_park_waiters"),
        "llm_provider": (json.loads((RUN_DIR / "health.json").read_text()).get("llm_provider_name")
                         if (RUN_DIR / "health.json").exists() else None),
        "run_dir": str(RUN_DIR),
    }
    (RUN_DIR / "summary.json").write_text(json.dumps(summary, indent=2))
    print(json.dumps(summary, indent=2))
    return 0 if all_s is not None else 2


if __name__ == "__main__":
    sys.exit(main())
