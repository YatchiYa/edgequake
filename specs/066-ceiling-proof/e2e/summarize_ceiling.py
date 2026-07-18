#!/usr/bin/env python3
"""Summarize SPEC-066 ceiling JSONL into CEILING_SUMMARY.md."""
from __future__ import annotations

import json
import sys
from pathlib import Path


def main() -> None:
    if len(sys.argv) != 5:
        print("usage: summarize_ceiling.py REPORT DST PROFILE LABEL", file=sys.stderr)
        sys.exit(2)
    src, dst, profile, label = (
        Path(sys.argv[1]),
        Path(sys.argv[2]),
        sys.argv[3],
        sys.argv[4],
    )
    single = stress = recall = graph = None
    for line in src.read_text().splitlines():
        try:
            o = json.loads(line)
        except json.JSONDecodeError:
            continue
        op = o.get("op", "")
        if op == "ceiling_wave2_single":
            single = o
        elif op == "ceiling_wave2_stress":
            stress = o
        elif op == "ceiling_wave2_recall":
            recall = o
        elif op == "ceiling_graph_g1_degrees":
            graph = o
    rows = (single or {}).get("detail", "")
    slo = (single or {}).get("pass")
    text = [
        f"# SPEC-066 ceiling summary — {profile} / {label}",
        "",
        f"- single: p95={(single or {}).get('p95_ms')} pass={slo}",
        f"- stress: p95={(stress or {}).get('p95_ms')} pass={(stress or {}).get('pass')}",
        f"- recall: {(recall or {}).get('detail', 'n/a')}",
        f"- graph_g1: p95={(graph or {}).get('p95_ms')} pass={(graph or {}).get('pass')} "
        f"detail={(graph or {}).get('detail', 'n/a')}",
        f"- detail_single: `{rows}`",
        "",
        "See RUN_NOTES.md for highest_green_N / first_fail_N promotion.",
        "",
    ]
    dst.write_text("\n".join(text))
    print(f"OK wrote {dst}")


if __name__ == "__main__":
    main()
