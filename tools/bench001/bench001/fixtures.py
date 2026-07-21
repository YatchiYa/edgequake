"""Fixture helpers: read / verify question ID lists."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .paths import (
    CORE_FIXTURE,
    FAST_SMOKE_FIXTURE,
    SMOKE_FIXTURE,
    corpus_path,
    fixture_path,
    questions_path,
)


def read_ids(fixture_name: str) -> list[str]:
    path = fixture_path(fixture_name)
    lines = path.read_text(encoding="utf-8").splitlines()
    return [ln.strip() for ln in lines if ln.strip() and not ln.startswith("#")]


def load_questions(subset: str) -> list[dict[str, Any]]:
    path = questions_path(subset)
    with path.open(encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, list):
        raise ValueError(f"expected list in {path}")
    return data


def load_corpus(subset: str) -> list[dict[str, Any]]:
    path = corpus_path(subset)
    with path.open(encoding="utf-8") as f:
        data = json.load(f)
    if not isinstance(data, list):
        raise ValueError(f"expected list in {path}")
    return data


def index_questions(*subsets: str) -> dict[str, dict[str, Any]]:
    out: dict[str, dict[str, Any]] = {}
    for subset in subsets:
        for q in load_questions(subset):
            qid = q["id"]
            q["_subset"] = subset
            out[qid] = q
    return out


def select_questions(fixture_name: str) -> list[dict[str, Any]]:
    ids = read_ids(fixture_name)
    # smoke / smoke-fast = medical only; core = medical + novel
    medical_only = fixture_name in {SMOKE_FIXTURE, FAST_SMOKE_FIXTURE}
    subsets = ("medical",) if medical_only else ("medical", "novel")
    by_id = index_questions(*subsets)
    missing = [i for i in ids if i not in by_id]
    if missing:
        raise KeyError(f"{len(missing)} fixture IDs missing (e.g. {missing[:3]})")
    return [by_id[i] for i in ids]


def verify_fixtures() -> dict[str, Any]:
    smoke = read_ids(SMOKE_FIXTURE)
    fast = read_ids(FAST_SMOKE_FIXTURE) if fixture_path(FAST_SMOKE_FIXTURE).exists() else []
    core = read_ids(CORE_FIXTURE)
    return {
        "smoke_n": len(smoke),
        "fast_n": len(fast),
        "core_n": len(core),
        "smoke_path": str(fixture_path(SMOKE_FIXTURE)),
        "fast_path": str(fixture_path(FAST_SMOKE_FIXTURE)),
        "core_path": str(fixture_path(CORE_FIXTURE)),
        "smoke_exists": fixture_path(SMOKE_FIXTURE).exists(),
        "fast_exists": fixture_path(FAST_SMOKE_FIXTURE).exists(),
        "core_exists": fixture_path(CORE_FIXTURE).exists(),
    }


def freeze_smoke_verify() -> None:
    """Ensure committed smoke IDs resolve against downloaded questions."""
    qs = select_questions(SMOKE_FIXTURE)
    from collections import Counter

    counts = Counter(q["question_type"] for q in qs)
    print(f"smoke verified n={len(qs)} by_type={dict(counts)}")
    for t, n in counts.items():
        if n != 10:
            raise AssertionError(f"expected 10 of {t}, got {n}")
