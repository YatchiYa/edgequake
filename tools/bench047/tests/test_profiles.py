"""Profile / process_options wiring for SPEC-047 Phase A."""

from __future__ import annotations

from bench047.profiles import (
    PROFILES,
    QUERY_LLM_MODEL,
    VISION_MODEL_LOCKED_SMALL,
    VISION_MODEL_STRONG,
    get_profile,
)
from bench047.protocol import BEST_SCORE_STACK, DEFAULT_BENCH_PROFILE


def test_best_score_stack_locked_to_p0_mm_ite():
    """Acc #5 chart-8 stack SSOT — Phase B+ must use this profile by default."""
    assert DEFAULT_BENCH_PROFILE == "P0_mm_ite"
    assert BEST_SCORE_STACK["profile_id"] == "P0_mm_ite"
    assert BEST_SCORE_STACK["process_options"] == "ite"
    assert "W3-arith-v2" in " ".join(BEST_SCORE_STACK["levers"])
    assert BEST_SCORE_STACK["chart8_smoke_acc"] >= 0.56

    p = get_profile("P0_mm_ite")
    assert p.process_options == "ite"
    assert p.requires_vlm_process is True
    assert p.profile_id == "P0_mm_ite"
    assert p.llm_model == QUERY_LLM_MODEL
    assert p.vision_model == VISION_MODEL_LOCKED_SMALL
    assert p.uses_stronger_vision() is False
    assert p.is_split_llm_vision() is False


def test_p0_mm_ite_vision_medium_is_split_w1_ablation():
    """025: one causal change — Medium vision, Small query LLM, ite intact."""
    p = get_profile("P0_mm_ite_vision_medium")
    assert p.process_options == "ite"
    assert p.requires_vlm_process is True
    assert p.llm_model == QUERY_LLM_MODEL == "mistral-small-latest"
    assert p.vision_model == VISION_MODEL_STRONG == "mistral-medium-3-5"
    assert p.uses_stronger_vision() is True
    assert p.is_split_llm_vision() is True
    assert p.query_mode == "hybrid"
    assert p.embedding_model == "mistral-embed"
    assert p.embedding_dim == 1024


def test_locked_p0_mm_ite_unchanged_by_stronger_vision_profile():
    locked = get_profile("P0_mm_ite")
    strong = get_profile("P0_mm_ite_vision_medium")
    assert locked.vision_model == VISION_MODEL_LOCKED_SMALL
    assert strong.vision_model != locked.vision_model
    assert strong.llm_model == locked.llm_model


def test_p0_primary_has_no_process_options():
    p = get_profile("P0_primary")
    assert p.process_options is None
    assert p.requires_vlm_process is False


def test_all_profiles_registered():
    assert "P0_mm_ite" in PROFILES
    assert "P0_mm_ite_vision_medium" in PROFILES
    assert "P0_primary" in PROFILES
    assert "P1_mix_rrf" in PROFILES


def test_p1_mix_rrf_ablation_profile():
    p = get_profile("P1_mix_rrf")
    assert p.query_mode == "mix"
    assert p.process_options == "ite"
    assert p.requires_vlm_process is True
