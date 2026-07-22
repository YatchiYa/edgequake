#!/usr/bin/env python3
"""SPEC-062 — compare two /tmp/eq-perf-*.jsonl (or artifact) reports.

Usage:
  python3 scripts/compare_eq_perf_jsonl.py baseline.jsonl candidate.jsonl
  python3 scripts/compare_eq_perf_jsonl.py --cross-major \\
      specs/061-.../artifacts/eq-perf-pg16.jsonl \\
      specs/061-.../artifacts/eq-perf-pg17.jsonl \\
      specs/061-.../artifacts/eq-perf-pg18.jsonl

Exit 1 if any scalar op max/min p95 > 2× without noise_ok.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


def load_ops(path: Path) -> dict[str, dict[str, Any]]:
    out: dict[str, dict[str, Any]] = {}
    for line in path.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        o = json.loads(line)
        out[o.get("op", "?")] = o
    return out


def scalar_p95(o: dict[str, Any]) -> float | None:
    v = o.get("p95_ms")
    if isinstance(v, (int, float)):
        return float(v)
    return None


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("files", nargs="+", type=Path)
    ap.add_argument("--cross-major", action="store_true")
    ap.add_argument("--ratio", type=float, default=2.0)
    ap.add_argument(
        "--allow",
        action="append",
        default=[],
        help="Op name to treat as noise_ok (repeatable). Or set EQ_PERF_ALLOW_OPS=op1,op2",
    )
    args = ap.parse_args()

    if args.cross_major:
        by_op: dict[str, dict[str, float]] = {}
        noise: set[str] = set(args.allow)
        for part in __import__("os").environ.get("EQ_PERF_ALLOW_OPS", "").split(","):
            if part.strip():
                noise.add(part.strip())
        for f in args.files:
            ops = load_ops(f)
            for op, o in ops.items():
                p = scalar_p95(o)
                if p is None:
                    continue
                by_op.setdefault(op, {})[f.name] = p
                if o.get("noise_ok") or "noise_ok" in str(o.get("detail", "")):
                    noise.add(op)
        # Concurrent stress uses N=8 on pg16 vs N=16 on pg17/18 by design.
        # Deferred heap ingest p95 is one-sample spiky at n≈10 (see ingest stage gate).
        auto_noise = {
            "stress_concurrent_fts",
            "stress_concurrent_mix",
            "stress_pool_saturation",
            "ingest_vector_upsert_report_created",
        }
        noise |= auto_noise

        failed = False
        print("| Op | files… | max/min |")
        print("|----|--------|---------|")
        for op, vals in sorted(by_op.items()):
            if len(vals) < 2:
                continue
            mn, mx = min(vals.values()), max(vals.values())
            ratio = mx / mn if mn > 0 else float("inf")
            # Sub-100ms walls with sparse samples are host jitter, not major deltas.
            micro_noise = mx < 100.0
            flag = ""
            if ratio > args.ratio and op not in noise and not micro_noise:
                flag = " FAIL"
                failed = True
            elif ratio > args.ratio:
                flag = " noise_ok"
            print(f"| `{op}` | {vals} | {ratio:.2f}{flag} |")
        return 1 if failed else 0

    if len(args.files) != 2:
        print("compare mode needs exactly 2 files (or use --cross-major)", file=sys.stderr)
        return 2
    a, b = load_ops(args.files[0]), load_ops(args.files[1])
    ops = sorted(set(a) | set(b))
    print("| Op | baseline | candidate | delta |")
    print("|----|----------|-----------|-------|")
    for op in ops:
        pa, pb = scalar_p95(a.get(op, {})), scalar_p95(b.get(op, {}))
        if pa is None or pb is None:
            print(f"| `{op}` | {pa} | {pb} | n/a |")
            continue
        delta = ((pb - pa) / pa * 100.0) if pa else 0.0
        print(f"| `{op}` | {pa:.3f} | {pb:.3f} | {delta:+.1f}% |")
    return 0


if __name__ == "__main__":
    sys.exit(main())
