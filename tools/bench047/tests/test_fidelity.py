"""Unit tests for W1 representation fidelity (answer-in-page) + protocol gates."""

from bench047.fidelity import (
    aggregate_fidelity,
    answer_in_text,
    fidelity_for_sample,
    parse_crop_coverage_comment,
    split_markdown_by_page,
)
from bench047.protocol import (
    MIN_NEEDLE_LEN_GATE,
    PROTOCOL_VERSION,
    paired_acc_delta,
)


def test_split_page_markers():
    md = "pre\n<!-- edgequake-page:1 -->\nHello\n<!-- edgequake-page:2 -->\nWorld 42\n"
    pages = split_markdown_by_page(md)
    assert 1 in pages and 2 in pages
    assert "Hello" in pages[1]
    assert "42" in pages[2]


def test_parse_crop_coverage_comment():
    md = (
        "<!-- edgequake-page:1 -->\nbody\n"
        "<!-- edgequake-crop-coverage: total_pages=8 pages_with_fig=3 pages_with_table=2 "
        "residual_candidates=6 residual_alongside_fig=3 residual_skipped_due_to_fig_or_table=2 "
        "residual_after_ink_filter=2 residual_crops_written=2 -->\n"
    )
    cov = parse_crop_coverage_comment(md)
    assert cov is not None
    assert cov["total_pages"] == 8
    assert cov["residual_alongside_fig"] == 3
    assert cov["residual_skipped_due_to_fig_or_table"] == 2
    assert cov["residual_crops_written"] == 2
    assert parse_crop_coverage_comment("no telemetry") is None


def test_answer_in_text_numeric():
    assert answer_in_text("18.29%", "growth was 18.29 percent")
    assert answer_in_text("42", "the answer is 42 today")
    assert not answer_in_text("99", "the answer is 42")


def test_answer_in_text_list_all_members():
    """List gold hits iff every member is on the page (MMLongBench List physics)."""
    page = (
        "Views about Trump’s ability to make good decisions about economic policy "
        "and make wise decisions about immigration policy are mixed."
    )
    gold = (
        "['Make good decisions about economic policy ', "
        "'Make wise decisions about immigration policy ']"
    )
    assert answer_in_text(gold, page)
    assert not answer_in_text(gold, "only economic policy mentioned")


def test_answer_in_text_quoted_scalar():
    """Quoted string gold matches unquoted page text (W1-measure quote strip)."""
    assert answer_in_text('"MMMU"', "We introduce the MMMU benchmark")
    assert answer_in_text('"MMMU"', 'dataset label is "MMMU" here')


def test_answer_in_text_year_span_expand():
    """Abbreviated year spans expand so list gold years hit (032)."""
    page = "Discriminatory Taxation … Year 1981-82 … Year 2001-02 (EST)"
    gold = "['1981', '1982', '2001', '2002']"
    assert answer_in_text(gold, page)
    assert answer_in_text("1982", "Year 1981-82")
    assert not answer_in_text("1983", "Year 1981-82")


def test_fidelity_list_gold_long_hit():
    md = (
        "<!-- edgequake-page:5 -->\n"
        "Make good decisions about economic policy (53%). "
        "Make wise decisions about immigration policy (43%).\n"
    )
    gold = (
        "['Make good decisions about economic policy ', "
        "'Make wise decisions about immigration policy ']"
    )
    d = fidelity_for_sample(
        answer=gold,
        evidence_pages=[5],
        markdown=md,
        evidence_sources=["Chart"],
    )
    assert d["answer_in_evidence_pages"] is True
    assert d["answer_in_evidence_pages_long"] is True
    assert d["long_eligible"] is True


def test_fidelity_quoted_mmmu_on_page():
    md = "<!-- edgequake-page:4 -->\n# The MMMU Benchmark\nWe introduce MMMU.\n"
    d = fidelity_for_sample(
        answer='"MMMU"',
        evidence_pages=[4],
        markdown=md,
        evidence_sources=["Chart"],
    )
    assert d["answer_in_evidence_pages"] is True
    assert d["answer_in_evidence_pages_long"] is True


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
    assert d["long_eligible"] is True  # "1829" after normalize is long enough
    assert d["answer_in_evidence_pages_long"] is False


def test_fidelity_for_sample_hit():
    md = "<!-- edgequake-page:5 -->\nKPI reached 18.29% YoY\n"
    d = fidelity_for_sample(
        answer="18.29%",
        evidence_pages=[5],
        markdown=md,
        evidence_sources=["Chart"],
    )
    assert d["answer_in_evidence_pages"] is True
    assert d["answer_in_evidence_pages_long"] is True
    assert d["pages_with_answer"] == [5]


def test_short_needle_fp_suspect():
    """Single-digit gold on every page → raw hit but FP-suspect; excluded from long."""
    pages = "\n".join(
        f"<!-- edgequake-page:{i} -->\nvalue is 6 here\n" for i in range(1, 6)
    )
    d = fidelity_for_sample(
        answer="6",
        evidence_pages=[1],
        markdown=pages,
        evidence_sources=["Chart"],
    )
    assert d["answer_in_evidence_pages"] is True
    assert d["short_needle"] is True
    assert d["needle_len"] < MIN_NEEDLE_LEN_GATE
    assert d["long_eligible"] is False
    assert d["answer_in_evidence_pages_long"] is None
    assert d["short_needle_fp_suspect"] is True
    assert d["page_spread"] >= 0.3


def test_aggregate_by_source_and_long_gate():
    rows = [
        {
            "answer_in_evidence_pages": True,
            "answer_in_evidence_pages_long": True,
            "answer_in_document": True,
            "evidence_sources": ["Chart"],
            "long_eligible": True,
            "short_needle": False,
            "short_needle_fp_suspect": False,
            "needle_len": 5,
        },
        {
            "answer_in_evidence_pages": True,
            "answer_in_evidence_pages_long": None,
            "answer_in_document": True,
            "evidence_sources": ["Chart"],
            "long_eligible": False,
            "short_needle": True,
            "short_needle_fp_suspect": True,
            "needle_len": 1,
        },
        {
            "answer_in_evidence_pages": False,
            "answer_in_evidence_pages_long": False,
            "answer_in_document": True,
            "evidence_sources": ["Table"],
            "long_eligible": True,
            "short_needle": False,
            "short_needle_fp_suspect": False,
            "needle_len": 4,
        },
    ]
    agg = aggregate_fidelity(rows)
    assert agg["n"] == 3
    assert agg["protocol_version"] == PROTOCOL_VERSION
    assert abs(agg["answer_in_evidence_rate"] - 2 / 3) < 1e-9
    assert agg["by_evidence_source"]["Chart"]["rate"] == 1.0
    # Long: only eligible Chart hit + Table miss → Chart long rate 1.0 n=1
    assert agg["n_long_eligible"] == 2
    assert abs(agg["answer_in_evidence_rate_long"] - 0.5) < 1e-9
    assert agg["by_evidence_source_long"]["Chart"]["rate"] == 1.0
    assert agg["by_evidence_source_long"]["Chart"]["n"] == 1
    assert agg["n_short_needle_fp_suspect"] == 1
    assert agg["gates"]["chart_a_in_e_long"]["n"] == 1
    assert agg["gates"]["chart_a_in_e_long"]["pass"] is True  # 1.0 >= 0.50


def test_paired_acc_delta_attribution():
    base = [
        {"doc_id": "a", "question": "q1", "answer": '["x","y"]', "score": 0.0},
        {"doc_id": "a", "question": "q2", "answer": "Not answerable", "score": 0.0},
        {"doc_id": "a", "question": "q3", "answer": "42", "score": 0.0},
    ]
    now = [
        {"doc_id": "a", "question": "q1", "answer": '["x","y"]', "score": 1.0},
        {"doc_id": "a", "question": "q2", "answer": "Not answerable", "score": 1.0},
        {"doc_id": "a", "question": "q3", "answer": "42", "score": 0.0},
    ]
    d = paired_acc_delta(base, now)
    assert d["n_paired"] == 3
    assert d["n_improved"] == 2
    assert abs(d["acc_points"]["list_gold"] - 1.0 / 3) < 1e-9
    assert abs(d["acc_points"]["unanswerable"] - 1.0 / 3) < 1e-9
    assert abs(d["acc_points"]["other_answerable"]) < 1e-9
