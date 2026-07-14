"""Smoke/core subset selection and Q&A loading."""

from __future__ import annotations

import ast
import hashlib
from pathlib import Path
from typing import Any

import pandas as pd

from .download import ensure_qa
from .paths import FIXTURES_DIR


def load_qa_df() -> pd.DataFrame:
    path = ensure_qa()
    df = pd.read_parquet(path)
    # normalize string columns
    for c in df.columns:
        if df[c].dtype == object:
            df[c] = df[c].astype(str)
    return df


def parse_list_field(raw: str) -> list[Any]:
    raw = (raw or "").strip()
    if not raw:
        return []
    try:
        val = ast.literal_eval(raw)
        if isinstance(val, list):
            return val
        return [val]
    except Exception:
        return []


def read_doc_ids(fixture_name: str) -> list[str]:
    path = FIXTURES_DIR / fixture_name
    if not path.exists():
        raise FileNotFoundError(path)
    ids = []
    for line in path.read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#") or line.startswith("PLACEHOLDER"):
            continue
        ids.append(line)
    if not ids:
        raise SystemExit(f"Fixture {path} has no doc_ids — run: bench047 freeze-smoke")
    return ids


def questions_for_docs(df: pd.DataFrame, doc_ids: list[str]) -> pd.DataFrame:
    return df[df["doc_id"].isin(doc_ids)].copy()


def freeze_smoke(n: int = 10, seed: str = "047-smoke-v1") -> list[str]:
    """Stratified greedy cover → write fixtures/smoke_doc_ids_v1.txt."""
    df = load_qa_df()
    rng = int(hashlib.sha256(seed.encode()).hexdigest()[:8], 16)

    def score_doc(doc_id: str) -> tuple:
        sub = df[df["doc_id"] == doc_id]
        pages = [parse_list_field(x) for x in sub["evidence_pages"]]
        sources = [parse_list_field(x) for x in sub["evidence_sources"]]
        flat_src = {s for lst in sources for s in lst}
        has_cross = any(len(p) > 1 for p in pages)
        has_unans = any(a == "Not answerable" for a in sub["answer"])
        has_chart = any("Chart" in s or "Image" in s or "figure" in s.lower() for s in flat_src)
        has_table = any("Table" in s for s in flat_src)
        doc_type = sub["doc_type"].iloc[0]
        n_q = len(sub)
        # prefer moderate question counts; diversity flags
        return (
            int(has_cross),
            int(has_unans),
            int(has_chart),
            int(has_table),
            n_q,
            doc_type,
            (hash(doc_id) ^ rng) & 0xFFFFFFFF,
        )

    docs = sorted(df["doc_id"].unique(), key=score_doc, reverse=True)
    selected: list[str] = []
    seen_types: set[str] = set()
    # pass 1: maximize type diversity
    for d in docs:
        if len(selected) >= n:
            break
        dt = df[df["doc_id"] == d]["doc_type"].iloc[0]
        if dt not in seen_types:
            selected.append(d)
            seen_types.add(dt)
    # pass 2: fill with highest scores
    for d in docs:
        if len(selected) >= n:
            break
        if d not in selected:
            selected.append(d)

    selected = selected[:n]
    out = FIXTURES_DIR / "smoke_doc_ids_v1.txt"
    lines = [
        "# SPEC-047 smoke_doc_ids_v1 — FROZEN",
        f"# seed={seed}",
        f"# n={len(selected)}",
        "# Do not edit without bumping to _v2.",
        "",
    ]
    lines.extend(selected)
    out.write_text("\n".join(lines) + "\n")

    rationale = FIXTURES_DIR / "smoke_selection_rationale_v1.md"
    rows = []
    for i, d in enumerate(selected, 1):
        sub = df[df["doc_id"] == d]
        pages = [parse_list_field(x) for x in sub["evidence_pages"]]
        sources = {s for lst in (parse_list_field(x) for x in sub["evidence_sources"]) for s in lst}
        rows.append(
            f"| {i} | `{d}` | {sub['doc_type'].iloc[0]} | {len(sub)} | "
            f"{'Y' if any(len(p)>1 for p in pages) else 'N'} | "
            f"{'Y' if any(a=='Not answerable' for a in sub['answer']) else 'N'} | "
            f"{'Y' if any('Chart' in s or 'Image' in s for s in sources) else 'N'} | |"
        )
    rationale.write_text(
        "\n".join(
            [
                "# Smoke selection rationale (v1) — FROZEN",
                "",
                f"**Seed:** `{seed}`",
                f"**Dataset:** yubo2333/MMLongBench-Doc parquet ({len(df)} questions, {df['doc_id'].nunique()} docs)",
                "",
                "| # | doc_id | doc_type | #Qs | cross-page? | unans? | chart/img? | notes |",
                "|---|--------|----------|-----|-------------|--------|------------|-------|",
                *rows,
                "",
                "Smoke is biased toward diversity, not an unbiased full-set estimator.",
                "",
            ]
        )
    )
    print(f"Wrote {out}")
    print(f"Wrote {rationale}")
    return selected


def freeze_core(n: int = 40, seed: str = "047-core-v1") -> list[str]:
    smoke = read_doc_ids("smoke_doc_ids_v1.txt")
    df = load_qa_df()
    # extend smoke with more diversity
    extra = freeze_smoke(n=n, seed=seed)  # temporarily overwrites smoke file — fix below
    # restore smoke and write core as smoke ∪ top extras
    # Re-run proper core: start from smoke, add until n
    selected = list(smoke)
    for d in extra:
        if d not in selected:
            selected.append(d)
        if len(selected) >= n:
            break
    # also pull remaining high-diversity
    all_docs = list(df["doc_id"].unique())
    for d in all_docs:
        if len(selected) >= n:
            break
        if d not in selected:
            selected.append(d)
    selected = selected[:n]
    # restore smoke file
    freeze_smoke(n=10, seed="047-smoke-v1")
    out = FIXTURES_DIR / "core_doc_ids_v1.txt"
    out.write_text(
        "# SPEC-047 core_doc_ids_v1 — FROZEN\n"
        f"# seed={seed}\n# includes smoke_doc_ids_v1\n\n"
        + "\n".join(selected)
        + "\n"
    )
    print(f"Wrote {out} ({len(selected)} docs)")
    return selected
