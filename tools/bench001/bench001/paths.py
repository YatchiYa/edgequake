"""Path helpers for SPEC-001."""

from __future__ import annotations

import os
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
SPEC_DIR = REPO_ROOT / "specs" / "001-benchmark"
FIXTURES_DIR = SPEC_DIR / "fixtures"
ARTIFACTS_DIR = SPEC_DIR / "e2e" / "artifacts"

DATASET_ID = "GraphRAG-Bench/GraphRAG-Bench"
DATASET_REVISION = "dc3a111e77dbaf8bbaf51ef331f3cfc9b1b5c546"
SMOKE_FIXTURE = "smoke_question_ids_v1.txt"
FAST_SMOKE_FIXTURE = "smoke_fast_question_ids_v1.txt"
PUBLISH_FIXTURE = "medical_publish_question_ids_v1.txt"
MEDICAL_FULL_FIXTURE = "medical_full_question_ids_v1.txt"
CORE_FIXTURE = "core_question_ids_v1.txt"

# Stages that use medical-only question subsets (no novel).
MEDICAL_ONLY_FIXTURES = frozenset(
    {SMOKE_FIXTURE, FAST_SMOKE_FIXTURE, PUBLISH_FIXTURE, MEDICAL_FULL_FIXTURE}
)

QUESTION_TYPES = (
    "Fact Retrieval",
    "Complex Reasoning",
    "Contextual Summarize",
    "Creative Generation",
)


def cache_root() -> Path:
    raw = os.environ.get("EDGEQUAKE_BENCH001_CACHE") or os.environ.get("EDGEQUAKE_BENCH_CACHE")
    if raw:
        return Path(raw).expanduser() / "bench001"
    return Path.home() / ".cache" / "edgequake" / "bench001"


def dataset_root() -> Path:
    return cache_root() / "graphrag-bench"


def questions_path(subset: str) -> Path:
    return dataset_root() / "Datasets" / "Questions" / f"{subset}_questions.json"


def corpus_path(subset: str) -> Path:
    return dataset_root() / "Datasets" / "Corpus" / f"{subset}.json"


def fixture_path(name: str) -> Path:
    return FIXTURES_DIR / name


def stage_artifact_dir(stage: str) -> Path:
    d = ARTIFACTS_DIR / stage
    d.mkdir(parents=True, exist_ok=True)
    (d / "logs").mkdir(exist_ok=True)
    return d


def publish_latest_dir() -> Path:
    """Stable publish pointer for ``make bench`` business pack."""
    d = ARTIFACTS_DIR / "publish" / "latest"
    d.mkdir(parents=True, exist_ok=True)
    return d


def api_base() -> str:
    return (
        os.environ.get("EDGEQUAKE_API_URL")
        or os.environ.get("BACKEND_URL")
        or "http://localhost:8080"
    )


def lightrag_repo() -> Path | None:
    raw = os.environ.get("BENCH001_LIGHTRAG_REPO")
    if raw:
        p = Path(raw).expanduser()
        return p if p.exists() else None
    sibling = REPO_ROOT.parent / "LightRAG"
    if sibling.exists():
        return sibling
    return None
