"""bench047 run.py parallel query wiring."""

from __future__ import annotations

from pathlib import Path

from bench047.profiles import get_profile
from bench047.score import build_scorecard


def test_build_scorecard_includes_query_workers():
    profile = get_profile("P0_mm_ite")
    card = build_scorecard(
        stage="smoke",
        profile=profile,
        samples=[{"score": 1.0, "evidence_pages": "[]", "evidence_sources": "[]", "answer": "x", "doc_type": "t"}],
        pins_extra={"fixture_id": "smoke_doc_ids_v1"},
        ops={"query_workers": 10, "n_docs": 1, "ingest_coverage": 1.0},
    )
    assert card["ops"]["query_workers"] == 10


def test_run_py_parallel_query_uses_worker_client_not_shadowing():
    """Regression: _run_one_query must not assign `client = EdgeQuakeClient(...)` (UnboundLocalError)."""
    src = (Path(__file__).resolve().parents[1] / "bench047" / "run.py").read_text()
    assert "run_workspace_id = client.workspace_id" in src
    assert "worker = EdgeQuakeClient(base_url=base_url, workspace_id=run_workspace_id)" in src
    assert "client = EdgeQuakeClient(base_url=base_url, workspace_id=client.workspace_id)" not in src


def test_run_py_resume_reuses_workspace_from_meta():
    src = (Path(__file__).resolve().parents[1] / "bench047" / "run.py").read_text()
    assert "resume workspace:" in src
    assert "if resume and meta_path.exists():" in src


def test_run_py_force_reindex_only_on_fresh_runs():
    """Regression: force_reindex=True on resume re-runs full vision (hours on 117-page PDFs)."""
    src = (Path(__file__).resolve().parents[1] / "bench047" / "run.py").read_text()
    assert "force_reindex = not resume" in src
    assert "force_reindex=True" not in src or "force_reindex = not resume" in src
