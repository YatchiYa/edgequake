"""Unit tests for SPEC-047 / 026 W4-extract short-answer normalize."""

from __future__ import annotations

from bench047.extract import EXTRACT_PROMPT, normalize_short_answer


def test_extract_prompt_pins_english_and_json_lists():
    assert "English" in EXTRACT_PROMPT
    assert "JSON array" in EXTRACT_PROMPT
    assert "Not answerable" in EXTRACT_PROMPT


def test_normalize_preserves_not_answerable():
    assert normalize_short_answer("Not answerable") == "Not answerable"
    assert normalize_short_answer("n/a") == "Not answerable"


def test_normalize_strips_fluff_prefix():
    assert normalize_short_answer("The answer is 42") == "42"
    assert normalize_short_answer("Final answer: 18.29%") == "18.29%"


def test_normalize_bullets_to_json_array():
    raw = "- Alice\n- Bob\n- Carol"
    assert normalize_short_answer(raw) == '["Alice", "Bob", "Carol"]'


def test_normalize_numbered_list_to_json_array():
    raw = "1. alpha\n2. beta"
    assert normalize_short_answer(raw) == '["alpha", "beta"]'


def test_normalize_comma_list_of_short_tokens():
    assert normalize_short_answer("Alice, Bob, Carol") == '["Alice", "Bob", "Carol"]'


def test_normalize_leaves_prose_sentence():
    prose = "The company grew revenue across three regions in Q4."
    assert normalize_short_answer(prose) == prose


def test_normalize_leaves_bare_number():
    assert normalize_short_answer("83672770") == "83672770"


def test_normalize_unwraps_singleton_json_array():
    """W4 Acc hygiene: ChartEx MMMU regression was pred=["MMMU"] vs gold=MMMU."""
    assert normalize_short_answer('["MMMU"]') == "MMMU"
    assert normalize_short_answer("[42]") == "42"


def test_normalize_keeps_multi_item_json_array():
    raw = '["Perceptual Error", "Lack of Knowledge", "Reasoning Error"]'
    assert normalize_short_answer(raw) == raw


def test_extract_answer_detailed_keeps_pred_raw(monkeypatch):
    from bench047 import extract as E

    monkeypatch.setattr(E, "_mistral_extract", lambda q, a, m: "- A\n- B")
    out = E.extract_answer_detailed("q", "long analysis", extractor="mistral")
    assert out["pred_raw"] == "- A\n- B"
    assert out["pred"] == '["A", "B"]'
    assert E.extract_answer("q", "long analysis", extractor="mistral") == '["A", "B"]'

