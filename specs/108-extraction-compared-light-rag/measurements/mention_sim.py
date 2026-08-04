#!/usr/bin/env python3
"""SPEC-108 mention (M) vs unique (U) simulation — illustrates LAW-X1 / H1.

Models: N chunks × per-chunk yield with cross-chunk name reuse → M ≫ U.
Not a live LLM run; labeled mock-extract.
"""

from __future__ import annotations

import json
import math
from dataclasses import asdict, dataclass
from pathlib import Path

OUT = Path(__file__).resolve().parent


@dataclass
class SimRow:
    label: str
    n_chunks: int
    yield_per_chunk: int
    reuse_fraction: float
    mentions_M: int
    unique_U: int
    m_over_u: float
    mode: str = "mock-extract"


def simulate(n_chunks: int, yield_per_chunk: int, reuse_fraction: float) -> SimRow:
    """reuse_fraction: share of names that already appeared in prior chunks."""
    seen: set[str] = set()
    mentions = 0
    for c in range(n_chunks):
        for i in range(yield_per_chunk):
            mentions += 1
            if seen and (i / max(1, yield_per_chunk)) < reuse_fraction:
                # reuse an existing name
                name = next(iter(seen))
            else:
                name = f"E_{c}_{i}"
                seen.add(name)
    u = len(seen)
    return SimRow(
        label=f"N={n_chunks} y={yield_per_chunk} reuse={reuse_fraction}",
        n_chunks=n_chunks,
        yield_per_chunk=yield_per_chunk,
        reuse_fraction=reuse_fraction,
        mentions_M=mentions,
        unique_U=u,
        m_over_u=round(mentions / u, 2) if u else 0.0,
    )


def partner_envelope() -> SimRow:
    """N≈309, yield=40 → M≈12360; with 70% reuse U collapses."""
    return simulate(309, 40, 0.70)


def main() -> None:
    rows = [
        simulate(8, 25, 0.40),  # S1-md fair small
        simulate(12, 25, 0.40),  # S1-md product
        simulate(159, 30, 0.55),  # S2 fair
        simulate(317, 30, 0.55),  # S2 product adaptive
        partner_envelope(),
    ]
    lines = [
        "# SPEC-108 M vs U simulation (mock-extract)",
        "",
        "> Illustrates LAW-X1: document card stores M; graph stores U.",
        "",
        "| label | N | yield | reuse | M | U | M/U |",
        "|-------|--:|------:|------:|--:|--:|----:|",
    ]
    for r in rows:
        lines.append(
            f"| {r.label} | {r.n_chunks} | {r.yield_per_chunk} | {r.reuse_fraction} | "
            f"{r.mentions_M} | {r.unique_U} | {r.m_over_u} |"
        )
    lines.extend(
        [
            "",
            "## Partner read",
            "",
            f"Envelope row M={rows[-1].mentions_M} ≈ partner 12367; U={rows[-1].unique_U} "
            f"(M/U={rows[-1].m_over_u}). UI showing M without U looks like “12k entities”.",
            "",
        ]
    )
    md = "\n".join(lines)
    (OUT / "mention_vs_unique.md").write_text(md, encoding="utf-8")
    (OUT / "mention_vs_unique.json").write_text(
        json.dumps([asdict(r) for r in rows], indent=2), encoding="utf-8"
    )
    print(md)


if __name__ == "__main__":
    main()
