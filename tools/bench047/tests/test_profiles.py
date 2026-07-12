"""Profile / process_options wiring for SPEC-047 Phase A."""

from __future__ import annotations

from bench047.profiles import PROFILES, get_profile


def test_p0_mm_ite_enables_ite_and_vlm_gate():
    p = get_profile("P0_mm_ite")
    assert p.process_options == "ite"
    assert p.requires_vlm_process is True
    assert p.profile_id == "P0_mm_ite"


def test_p0_primary_has_no_process_options():
    p = get_profile("P0_primary")
    assert p.process_options is None
    assert p.requires_vlm_process is False


def test_all_profiles_registered():
    assert "P0_mm_ite" in PROFILES
    assert "P0_primary" in PROFILES
    assert "P1_mix_rrf" in PROFILES


def test_p1_mix_rrf_ablation_profile():
    p = get_profile("P1_mix_rrf")
    assert p.query_mode == "mix"
    assert p.process_options == "ite"
    assert p.requires_vlm_process is True
