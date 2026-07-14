"""Path helpers for SPEC-047."""

from __future__ import annotations

import os
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
SPEC_DIR = REPO_ROOT / "specs" / "047-rag-evaluation"
FIXTURES_DIR = SPEC_DIR / "fixtures"
ARTIFACTS_DIR = SPEC_DIR / "e2e" / "artifacts"
EVAL_SCORE_SHA = "cafa268358b08521cfc9a805c67d86732d943677e4408c5751f336123d09631f"


def cache_root() -> Path:
    raw = os.environ.get("EDGEQUAKE_BENCH_CACHE")
    if raw:
        return Path(raw).expanduser()
    return Path.home() / ".cache" / "edgequake" / "bench047"


def dataset_root() -> Path:
    return cache_root() / "mmlongbench-doc"


def documents_dir() -> Path:
    return dataset_root() / "documents"


def qa_parquet_path() -> Path:
    # huggingface_hub local_dir layout
    p = dataset_root() / "data" / "train-00000-of-00001.parquet"
    if p.exists():
        return p
    # alternate
    alt = dataset_root() / "train-00000-of-00001.parquet"
    return alt if alt.exists() else p


def stage_artifact_dir(stage: str) -> Path:
    d = ARTIFACTS_DIR / stage
    d.mkdir(parents=True, exist_ok=True)
    (d / "logs").mkdir(exist_ok=True)
    return d


def api_base() -> str:
    return os.environ.get("EDGEQUAKE_API_URL") or os.environ.get("BACKEND_URL") or "http://localhost:8090"
