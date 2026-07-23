"""Fixture helpers: read / verify question ID lists."""

from __future__ import annotations

import json
from collections import Counter
from pathlib import Path
from typing import Any

from .paths import (
    CORE_FIXTURE,
    FAST_SMOKE_FIXTURE,
    MEDICAL_FULL_FIXTURE,
    MEDICAL_ONLY_FIXTURES,
    PUBLISH_FIXTURE,
    QUESTION_TYPES,
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
    # smoke / smoke-fast / medical-publish = medical only; core = medical + novel
    medical_only = fixture_name in MEDICAL_ONLY_FIXTURES
    subsets = ("medical",) if medical_only else ("medical", "novel")
    by_id = index_questions(*subsets)
    missing = [i for i in ids if i not in by_id]
    if missing:
        raise KeyError(f"{len(missing)} fixture IDs missing (e.g. {missing[:3]})")
    return [by_id[i] for i in ids]


def verify_fixtures() -> dict[str, Any]:
    smoke = read_ids(SMOKE_FIXTURE)
    fast = read_ids(FAST_SMOKE_FIXTURE) if fixture_path(FAST_SMOKE_FIXTURE).exists() else []
    publish = (
        read_ids(PUBLISH_FIXTURE) if fixture_path(PUBLISH_FIXTURE).exists() else []
    )
    medical_full = (
        read_ids(MEDICAL_FULL_FIXTURE)
        if fixture_path(MEDICAL_FULL_FIXTURE).exists()
        else []
    )
    core = read_ids(CORE_FIXTURE)
    return {
        "smoke_n": len(smoke),
        "fast_n": len(fast),
        "publish_n": len(publish),
        "medical_full_n": len(medical_full),
        "core_n": len(core),
        "smoke_path": str(fixture_path(SMOKE_FIXTURE)),
        "fast_path": str(fixture_path(FAST_SMOKE_FIXTURE)),
        "publish_path": str(fixture_path(PUBLISH_FIXTURE)),
        "medical_full_path": str(fixture_path(MEDICAL_FULL_FIXTURE)),
        "core_path": str(fixture_path(CORE_FIXTURE)),
        "smoke_exists": fixture_path(SMOKE_FIXTURE).exists(),
        "fast_exists": fixture_path(FAST_SMOKE_FIXTURE).exists(),
        "publish_exists": fixture_path(PUBLISH_FIXTURE).exists(),
        "medical_full_exists": fixture_path(MEDICAL_FULL_FIXTURE).exists(),
        "core_exists": fixture_path(CORE_FIXTURE).exists(),
    }


def freeze_smoke_verify() -> None:
    """Ensure committed smoke IDs resolve against downloaded questions."""
    qs = select_questions(SMOKE_FIXTURE)
    counts = Counter(q["question_type"] for q in qs)
    print(f"smoke verified n={len(qs)} by_type={dict(counts)}")
    for t in QUESTION_TYPES:
        n = counts.get(t, 0)
        if n != 10:
            raise AssertionError(f"expected 10 of {t}, got {n}")


def freeze_publish_verify() -> None:
    """Ensure medical-mid publish IDs resolve; 50/type; smoke is a subset."""
    qs = select_questions(PUBLISH_FIXTURE)
    counts = Counter(q["question_type"] for q in qs)
    print(f"medical-publish verified n={len(qs)} by_type={dict(counts)}")
    if len(qs) != 200:
        raise AssertionError(f"expected n=200, got {len(qs)}")
    for t in QUESTION_TYPES:
        n = counts.get(t, 0)
        if n != 50:
            raise AssertionError(f"expected 50 of {t}, got {n}")
    smoke_ids = set(read_ids(SMOKE_FIXTURE))
    publish_ids = set(read_ids(PUBLISH_FIXTURE))
    missing = smoke_ids - publish_ids
    if missing:
        raise AssertionError(
            f"publish fixture must be a superset of smoke; missing {len(missing)} "
            f"(e.g. {sorted(missing)[:3]})"
        )


def freeze_medical_full_verify() -> None:
    """Ensure medical-full IDs resolve; supersets medical-mid publish."""
    qs = select_questions(MEDICAL_FULL_FIXTURE)
    counts = Counter(q["question_type"] for q in qs)
    print(f"medical-full verified n={len(qs)} by_type={dict(counts)}")
    if len(qs) < 2000:
        raise AssertionError(f"expected n≈2062 medical-full, got {len(qs)}")
    publish_ids = set(read_ids(PUBLISH_FIXTURE))
    full_ids = set(read_ids(MEDICAL_FULL_FIXTURE))
    missing = publish_ids - full_ids
    if missing:
        raise AssertionError(
            f"medical-full must be a superset of medical-mid; missing {len(missing)} "
            f"(e.g. {sorted(missing)[:3]})"
        )
    print(f"medical-full supersets medical-mid (n_mid={len(publish_ids)})")
