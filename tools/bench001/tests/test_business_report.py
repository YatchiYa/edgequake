"""Business publish pack honesty language (make bench)."""

from __future__ import annotations

import json
from pathlib import Path

from bench001.business_report import (
    build_business_report_md,
    build_exec_summary,
    can_claim_beats_lightrag,
    ci_includes_zero,
    verdict_label,
    write_publish_pack,
)

FIXTURE = (
    Path(__file__).resolve().parents[3]
    / "specs"
    / "001-benchmark"
    / "e2e"
    / "artifacts"
    / "history"
    / "smoke-20260719T151125Z"
    / "scorecard.json"
)


def _load_scorecard() -> dict:
    return json.loads(FIXTURE.read_text(encoding="utf-8"))


def test_tie_verdict_from_real_scorecard():
    sc = _load_scorecard()
    assert ci_includes_zero(sc) is True
    assert "STATISTICAL TIE" in verdict_label(sc)
    assert can_claim_beats_lightrag(sc) is False


def test_business_report_contains_decision_layers_and_honesty():
    sc = _load_scorecard()
    md = build_business_report_md(sc, archive_rel="specs/001-benchmark/e2e/artifacts/history/x")
    assert "STATISTICAL TIE" in md
    assert "Answer quality (Acc)" in md
    assert "Evidence coverage" in md
    assert "Context cleanliness" in md
    assert "Speed (query p50)" in md
    assert "HippoRAG2" in md
    assert "make bench" in md
    assert "Forbidden" in md or "forbidden" in md.lower()
    assert "SOTA" in md
    assert "beats LightRAG" not in md.lower().split("forbidden")[0] or "Peer" in md
    # Must not claim beats when CI includes 0
    assert "EdgeQuake beats LightRAG on Acc under these pins" not in md


def test_exec_summary_peer_claim():
    sc = _load_scorecard()
    txt = build_exec_summary(sc)
    assert "STATISTICAL TIE" in txt or "peer" in txt.lower()
    assert "not SOTA" in txt.lower() or "not SOTA win" in txt
    assert "make bench" in txt


def test_write_publish_pack(tmp_path: Path):
    sc = _load_scorecard()
    stage = tmp_path / "smoke"
    stage.mkdir()
    (stage / "scorecard.json").write_text(json.dumps(sc), encoding="utf-8")
    (stage / "SUMMARY.md").write_text("# tech\n", encoding="utf-8")
    hist = tmp_path / "history" / "smoke-test"
    hist.mkdir(parents=True)
    latest = write_publish_pack(sc, stage_dir=stage, archive_dir=hist)
    assert (stage / "BUSINESS_REPORT.md").is_file()
    assert (stage / "EXEC_SUMMARY.txt").is_file()
    assert (hist / "BUSINESS_REPORT.md").is_file()
    assert (latest / "BUSINESS_REPORT.md").is_file()
    assert (latest / "scorecard.json").is_file()
    assert "STATISTICAL TIE" in (latest / "BUSINESS_REPORT.md").read_text(encoding="utf-8")


def test_beats_claim_only_when_ci_and_l2():
    sc = _load_scorecard()
    # Force a fake Acc win with CI excluding 0 and strong L2
    sc["metrics"]["eq"]["overall_acc"] = 0.90
    sc["metrics"]["lr"]["overall_acc"] = 0.70
    sc["metrics"]["delta_eq_minus_lr"]["overall_acc"] = 0.20
    sc["metrics"]["delta_eq_minus_lr"]["overall_acc_delta_ci"] = {
        "ci_low": 0.05,
        "ci_high": 0.30,
        "n": 40,
    }
    sc["metrics"]["eq"]["retrieval"]["overall_context_relevancy"] = 0.55
    sc["metrics"]["eq"]["retrieval"]["overall_evidence_recall"] = 0.96
    sc["metrics"]["lr"]["retrieval"]["overall_evidence_recall"] = 0.97
    assert ci_includes_zero(sc) is False
    assert can_claim_beats_lightrag(sc) is True
    assert "ahead" in verdict_label(sc).lower()
