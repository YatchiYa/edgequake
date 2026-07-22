"""Provider / judge pin resolution + lineage."""

from __future__ import annotations

from bench001.profiles import (
    DEFAULT_EMBEDDING_MODEL,
    DEFAULT_JUDGE_EMBEDDING_MODEL,
    DEFAULT_LLM_MODEL,
    PAPER_JUDGE_EMBEDDING_MODEL,
    ProviderPins,
    pin_block,
    resolve_pins,
    set_active_pins,
)
from bench001.scorecard import build_scorecard, write_summary


def test_defaults_are_mistral_small_and_embed(monkeypatch):
    for k in (
        "BENCH001_LLM_PROVIDER",
        "BENCH001_LLM_MODEL",
        "EDGEQUAKE_LLM_PROVIDER",
        "EDGEQUAKE_LLM_MODEL",
        "BENCH001_EMBEDDING_MODEL",
        "MISTRAL_EMBEDDING_MODEL",
        "BENCH001_JUDGE_MODEL",
        "BENCH001_LLM_BASE_URL",
        "OPENAI_BASE_URL",
        "BENCH001_PROFILE_ID",
        "BENCH001_PUBLISH_FAIRNESS",
        "EDGEQUAKE_MIX_ARM_GATE",
    ):
        monkeypatch.delenv(k, raising=False)
    pins = resolve_pins()
    assert pins.llm_provider == "mistral"
    assert pins.llm_model == DEFAULT_LLM_MODEL == "mistral-small-latest"
    assert pins.vision_provider == "mistral"
    assert pins.vision_model == "mistral-small-latest"
    assert pins.embedding_provider == "mistral"
    assert pins.embedding_model == DEFAULT_EMBEDDING_MODEL == "mistral-embed"
    assert pins.embedding_dim == 1024
    assert pins.judge_model == "mistral-small-latest"
    assert pins.judge_provider == "mistral"
    assert "mistral.ai" in pins.llm_base_url
    assert pins.judge_embedding_model == DEFAULT_JUDGE_EMBEDDING_MODEL == "mistral-embed"
    assert PAPER_JUDGE_EMBEDDING_MODEL.startswith("BAAI/")
        # Publishable fairness: matched top-k + L2 + LR-like always-on Mix arms.
    assert pins.profile_id == "P0_mistral_mix_lrlike_arms_v2"


def test_cli_overrides_beat_env(monkeypatch):
    monkeypatch.setenv("BENCH001_LLM_MODEL", "from-env")
    pins = resolve_pins(llm_model="gpt-4o-mini", llm_provider="openai")
    assert pins.llm_model == "gpt-4o-mini"
    assert pins.llm_provider == "openai"
    assert pins.profile_id.startswith("P0_custom_")


def test_judge_independent_of_sut(monkeypatch):
    monkeypatch.delenv("BENCH001_JUDGE_MODEL", raising=False)
    monkeypatch.delenv("BENCH001_JUDGE_BASE_URL", raising=False)
    pins = resolve_pins(
        llm_model="mistral-small-latest",
        judge_model="gpt-4o-mini",
        judge_provider="openai",
    )
    assert pins.llm_model == "mistral-small-latest"
    assert pins.judge_model == "gpt-4o-mini"
    assert pins.judge_provider == "openai"
    assert "openai.com" in pins.judge_base_url
    assert "mistral.ai" in pins.llm_base_url
    lin = pins.lineage()
    assert lin["sut_llm"] == "mistral/mistral-small-latest"
    assert lin["judge_llm"] == "openai/gpt-4o-mini"


def test_scorecard_records_lineage(tmp_path, monkeypatch):
    for k in ("BENCH001_LLM_MODEL", "EDGEQUAKE_LLM_MODEL", "BENCH001_JUDGE_MODEL"):
        monkeypatch.delenv(k, raising=False)
    pins = resolve_pins()
    set_active_pins(pins)
    sc = build_scorecard(
        stage="smoke-dry-run",
        fixture_id="smoke_question_ids_v1",
        eq_metrics={"overall_acc": 0.1, "by_type": {}},
        lr_metrics={"overall_acc": 0.2, "by_type": {}},
        eq_preds=[{"generated_answer": "a", "context": ["c"]}],
        lr_preds=[{"generated_answer": "b", "context": ["c"]}],
        valid=False,
        invalid_reason="dry_run",
        judge="rouge_proxy",
        provider_pins=pins,
    )
    assert sc["pins"]["llm_model"] == "mistral-small-latest"
    assert sc["pins"]["embedding_model"] == "mistral-embed"
    assert sc["pins"]["judge_model"] == "mistral-small-latest"
    assert sc["pins"]["lineage"]["sut_embed"].startswith("mistral/mistral-embed")
    out = tmp_path / "SUMMARY.md"
    write_summary(sc, out)
    text = out.read_text()
    assert "## Model lineage" in text
    assert "mistral-small-latest" in text
    assert "mistral-embed" in text


def test_pin_block_includes_judge_fields():
    pins = ProviderPins.defaults()
    block = pin_block(
        fixture_id="smoke_question_ids_v1",
        judge="generation_eval",
        git_sha="deadbeef",
        dataset_id="GraphRAG-Bench/GraphRAG-Bench",
        dataset_revision="abc",
        pins=pins,
    )
    assert block["judge"] == "generation_eval"
    assert block["judge_model"] == "mistral-small-latest"
    assert block["judge_embedding_model"] == DEFAULT_JUDGE_EMBEDDING_MODEL
    assert "lineage" in block
