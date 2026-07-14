"""Unit tests for W0 retrieval diagnostics (page_hit@k) + 020 refusal/arm gates."""

from pathlib import Path

from bench047.diagnostics import (
    aggregate_arm_gate_metrics,
    aggregate_false_refusal_metrics,
    aggregate_page_hit_metrics,
    build_retrieval_diagnostics,
    chunk_sources,
    is_false_refusal,
    is_not_answerable_pred,
    page_hit_at_k,
    page_recall_at_k,
    retrieved_pages_ordered,
)
from bench047.profiles import get_profile
from bench047.score import build_scorecard, write_summary


def test_page_hit_basic():
    assert page_hit_at_k([3, 7], [1, 2, 3, 9], 3) is True
    assert page_hit_at_k([3, 7], [1, 2, 9], 3) is False
    assert page_hit_at_k([3, 7], [1, 2, 9, 7], 3) is False
    assert page_hit_at_k([3, 7], [1, 2, 9, 7], 4) is True


def test_page_recall():
    assert page_recall_at_k([3, 7], [3, 1, 7], 3) == 1.0
    assert page_recall_at_k([3, 7], [3, 1], 2) == 0.5
    assert page_recall_at_k([], [1], 1) == 0.0


def test_chunk_sources_skips_entities():
    sources = [
        {"source_type": "entity", "id": "E1", "page_start": None},
        {"source_type": "chunk", "id": "c1", "page_start": 4, "snippet": "x"},
        {"source_type": "relationship", "id": "r1"},
        {"source_type": "chunk", "id": "c2", "page_start": 4, "snippet": "y"},
        {"source_type": "chunk", "id": "c3", "page_start": 9, "snippet": "z"},
    ]
    chunks = chunk_sources(sources)
    assert [c["id"] for c in chunks] == ["c1", "c2", "c3"]
    assert retrieved_pages_ordered(sources) == [4, 9]


def test_build_retrieval_diagnostics():
    resp = {
        "sources": [
            {
                "source_type": "chunk",
                "id": "a",
                "page_start": 2,
                "document_id": "d1",
                "snippet": "hi",
            },
            {
                "source_type": "chunk",
                "id": "b",
                "page_start": 5,
                "document_id": "d1",
                "snippet": "yo",
            },
            {"source_type": "entity", "id": "ENT", "snippet": "name"},
        ],
        "stats": {"sources_retrieved": 3, "retrieval_time_ms": 12},
    }
    d = build_retrieval_diagnostics(resp, evidence_pages=[5, 99])
    assert d["page_hit@1"] is False
    assert d["page_hit@3"] is True
    assert d["page_hit@5"] is True
    assert d["context_empty"] is False
    assert d["retrieved_pages"] == [2, 5]
    assert d["gold_page_in_retrieved"] == [5]
    assert d["n_chunk_sources"] == 2


def test_prefer_engine_context_empty():
    d = build_retrieval_diagnostics(
        {
            "sources": [
                {"source_type": "chunk", "id": "c", "page_start": 1, "snippet": "x"},
            ],
            "stats": {"context_empty": True, "sources_retrieved": 1},
        },
        evidence_pages=[1],
    )
    assert d["context_empty"] is True


def test_arm_stats_projected():
    d = build_retrieval_diagnostics(
        {
            "sources": [],
            "stats": {
                "context_empty": True,
                "sources_retrieved": 0,
                "arms_run": "local,naive",
                "arms_gated": True,
                "arm_local_chunks": 2,
                "arm_naive_chunks": 4,
                "arm_local_ms": 10,
                "arm_naive_ms": 20,
            },
        },
        evidence_pages=[1],
    )
    assert d["arms_run"] == "local,naive"
    assert d["arm_local_chunks"] == 2
    assert d["arm_naive_chunks"] == 4


def test_aggregate_skips_unanswerable():
    samples = [
        {
            "answer": "42",
            "retrieval": {
                "page_hit@5": True,
                "page_recall@5": 1.0,
                "context_empty": False,
                "n_chunk_sources": 2,
            },
        },
        {
            "answer": "Not answerable",
            "retrieval": {
                "page_hit@5": False,
                "page_recall@5": 0.0,
                "context_empty": True,
                "n_chunk_sources": 0,
            },
        },
        {
            "answer": "x",
            "retrieval": {
                "page_hit@5": False,
                "page_recall@5": 0.0,
                "context_empty": False,
                "n_chunk_sources": 1,
            },
        },
    ]
    agg = aggregate_page_hit_metrics(samples)
    assert agg["n_with_retrieval_diag"] == 2
    assert agg["page_hit@5"] == 0.5
    assert abs(agg["context_empty_rate"] - 0.0) < 1e-9


def test_is_not_answerable_pred_aliases():
    assert is_not_answerable_pred("Not answerable")
    assert is_not_answerable_pred("not answerable.")
    assert is_not_answerable_pred("Insufficient evidence")
    assert not is_not_answerable_pred("42")
    assert not is_not_answerable_pred("")


def test_false_refusal_and_aggregate():
    samples = [
        {
            "answer": "42",
            "pred": "Not answerable",
            "retrieval": {"page_hit@5": True},
        },
        {
            "answer": "7",
            "pred": "7",
            "retrieval": {"page_hit@5": True},
        },
        {
            "answer": "9",
            "pred": "Not answerable",
            "retrieval": {"page_hit@5": False},
        },
        {
            "answer": "Not answerable",
            "pred": "Not answerable",
            "retrieval": {"page_hit@5": False},
        },
    ]
    assert is_false_refusal(samples[0]) is True
    assert is_false_refusal(samples[1]) is False
    assert is_false_refusal(samples[3]) is False

    agg = aggregate_false_refusal_metrics(samples)
    assert agg["n_answerable"] == 3
    assert agg["n_false_refusal"] == 2
    assert abs(agg["false_refusal_rate"] - 2 / 3) < 1e-9
    assert agg["n_answerable_page_hit@5"] == 2
    assert agg["n_false_refusal_page_hit@5"] == 1
    assert abs(agg["false_refusal_given_page_hit@5"] - 0.5) < 1e-9


def test_aggregate_arm_gate_metrics():
    samples = [
        {
            "retrieval": {
                "arms_run": "naive",
                "arms_gated": True,
                "arm_naive_chunks": 10,
                "arm_local_chunks": 0,
                "arm_global_chunks": 0,
            }
        },
        {
            "retrieval": {
                "arms_run": "local,global,naive",
                "arms_gated": False,
                "arm_naive_chunks": 5,
                "arm_local_chunks": 3,
                "arm_global_chunks": 2,
            }
        },
        {"retrieval": {"page_hit@5": True}},
    ]
    agg = aggregate_arm_gate_metrics(samples)
    assert agg["n_with_arm_diag"] == 2
    assert abs(agg["arms_gated_rate"] - 0.5) < 1e-9
    assert abs(agg["naive_only_rate"] - 0.5) < 1e-9
    assert abs(agg["arm_graph_present_rate"] - 0.5) < 1e-9
    assert abs(agg["planned_naive_only_rate"] - 0.5) < 1e-9
    assert abs(agg["planned_graph_rate"] - 0.5) < 1e-9
    assert abs(agg["planned_local_rate"] - 0.5) < 1e-9


def test_planned_arms_vs_empty_local_chunks():
    """020 B2: local scheduled but 0 chunks ≠ planned naive-only."""
    samples = [
        {
            "retrieval": {
                "arms_run": "local,naive",
                "arms_gated": True,
                "arm_naive_chunks": 12,
                "arm_local_chunks": 0,
                "arm_global_chunks": 0,
            }
        },
    ]
    agg = aggregate_arm_gate_metrics(samples)
    assert abs(agg["naive_only_rate"] - 1.0) < 1e-9  # productive
    assert abs(agg["planned_naive_only_rate"] - 0.0) < 1e-9
    assert abs(agg["planned_local_rate"] - 1.0) < 1e-9
    assert abs(agg["planned_graph_rate"] - 1.0) < 1e-9


def test_summary_includes_false_refusal_and_arm_gates(tmp_path: Path):
    profile = get_profile("P0_primary")
    samples = [
        {
            "doc_id": "d.pdf",
            "doc_type": "Research report / Introduction",
            "question": "q",
            "answer": "42",
            "pred": "Not answerable",
            "answer_format": "Int",
            "evidence_pages": [1],
            "evidence_sources": ["Pure-text (Plain-text)"],
            "score": 0.0,
        }
    ]
    sc = build_scorecard(
        stage="smoke",
        profile=profile,
        samples=samples,
        pins_extra={"fixture_id": "t"},
        ops={
            "n_docs": 1,
            "n_questions": 1,
            "ingest_coverage": 1.0,
            "n_skipped_ingest_failed": 0,
            "document_scope": True,
            "retrieval": {
                "n_with_retrieval_diag": 1,
                "context_empty_rate": 0.0,
                "page_hit@1": 1.0,
                "page_hit@3": 1.0,
                "page_hit@5": 1.0,
                "page_hit@10": 1.0,
                "page_recall@5": 1.0,
                "mean_n_chunk_sources": 2,
            },
            "false_refusal": {
                "n_answerable": 1,
                "n_false_refusal": 1,
                "false_refusal_rate": 1.0,
                "n_answerable_page_hit@5": 1,
                "n_false_refusal_page_hit@5": 1,
                "false_refusal_given_page_hit@5": 1.0,
            },
            "arm_gates": {
                "n_with_arm_diag": 1,
                "arms_gated_rate": 1.0,
                "arm_graph_present_rate": 0.0,
                "naive_only_rate": 1.0,
                "arm_local_present_rate": 0.0,
                "arm_global_present_rate": 0.0,
            },
        },
        valid=True,
    )
    assert sc["ops"]["false_refusal"]["false_refusal_rate"] == 1.0
    out = tmp_path / "SUMMARY.md"
    write_summary(sc, out)
    text = out.read_text()
    assert "## Refusal diagnostics (020 A2)" in text
    assert "false_refusal_rate: 1.0000" in text
    assert "## Arm-gate diagnostics (020 B1/B2)" in text
    assert "arms_gated_rate: 1.0" in text
