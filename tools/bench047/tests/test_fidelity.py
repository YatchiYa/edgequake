"""Unit tests for W1 representation fidelity (answer-in-page)."""

from bench047.fidelity import (
    aggregate_fidelity,
    answer_in_text,
    fidelity_for_sample,
    split_markdown_by_page,
)


def test_split_page_markers():
    md = "pre\n<!-- edgequake-page:1 -->\nHello\n<!-- edgequake-page:2 -->\nWorld 42\n"
    pages = split_markdown_by_page(md)
    assert 1 in pages and 2 in pages
    assert "Hello" in pages[1]
    assert "42" in pages[2]


def test_answer_in_text_numeric():
    assert answer_in_text("18.29%", "growth was 18.29 percent")
    assert answer_in_text("42", "the answer is 42 today")
    assert not answer_in_text("99", "the answer is 42")


def test_fidelity_for_sample_chart_miss():
    md = (
        "<!-- edgequake-page:1 -->\nintro\n"
        "<!-- edgequake-page:3 -->\nchart shows revenue\n"
    )
    d = fidelity_for_sample(
        answer="18.29%",
        evidence_pages=[3],
        markdown=md,
        evidence_sources=["Chart"],
    )
    assert d["answer_in_evidence_pages"] is False
    assert d["answer_in_document"] is False


def test_fidelity_for_sample_hit():
    md = "<!-- edgequake-page:5 -->\nKPI reached 18.29% YoY\n"
    d = fidelity_for_sample(
        answer="18.29%",
        evidence_pages=[5],
        markdown=md,
        evidence_sources=["Chart"],
    )
    assert d["answer_in_evidence_pages"] is True
    assert d["pages_with_answer"] == [5]


def test_aggregate_by_source():
    rows = [
        {"answer_in_evidence_pages": True, "answer_in_document": True, "evidence_sources": ["Chart"]},
        {"answer_in_evidence_pages": False, "answer_in_document": True, "evidence_sources": ["Chart"]},
        {"answer_in_evidence_pages": True, "answer_in_document": True, "evidence_sources": ["Table"]},
    ]
    agg = aggregate_fidelity(rows)
    assert agg["n"] == 3
    assert abs(agg["answer_in_evidence_rate"] - 2 / 3) < 1e-9
    assert agg["by_evidence_source"]["Chart"]["rate"] == 0.5
