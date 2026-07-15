"""Catalog + wiring tests for SPEC-047 stronger vision (025)."""

from __future__ import annotations

from pathlib import Path

from bench047.profiles import VISION_MODEL_STRONG, get_profile
from bench047.run import _models_toml_supports_vision


def test_models_toml_marks_medium_3_5_supports_vision():
    assert _models_toml_supports_vision(VISION_MODEL_STRONG) is True
    assert _models_toml_supports_vision("mistral-medium-latest") is True
    assert _models_toml_supports_vision("mistral-small-latest") is True


def test_models_toml_rejects_unknown_vision_model():
    assert _models_toml_supports_vision("definitely-not-a-real-model-xyz") is False


def test_run_py_wires_profile_vision_to_workspace_and_upload():
    """SOLID/DIP: run_stage must pin vision from profile, not hard-code Small."""
    src = (Path(__file__).resolve().parents[1] / "bench047" / "run.py").read_text()
    assert "vision_llm_model=profile.vision_model" in src
    assert "vision_model=profile.vision_model" in src
    assert "_models_toml_supports_vision" in src


def test_client_create_workspace_payload_includes_vision_fields():
    src = (Path(__file__).resolve().parents[1] / "bench047" / "client.py").read_text()
    assert '"vision_llm_model": vision_llm_model' in src
    assert '"vision_model": vision_model' in src


def test_ensure_backend_respects_vision_env_override():
    src = (
        Path(__file__).resolve().parents[1] / "scripts" / "ensure_backend_small.sh"
    ).read_text()
    assert 'VISION_PIN="${EDGEQUAKE_VISION_MODEL:-mistral-small-latest}"' in src
    assert "EDGEQUAKE_VISION_MODEL=" in src and "${VISION_PIN}" in src


def test_vision_medium_smoke_script_pins_official_id():
    src = (
        Path(__file__).resolve().parents[1]
        / "scripts"
        / "run_chart8_vision_medium.sh"
    ).read_text()
    assert "P0_mm_ite_vision_medium" in src
    assert "mistral-medium-3-5" in src
    assert "mistral-small-latest" in src  # query LLM unchanged
    assert "EDGEQUAKE_VISION_MODEL=" in src


def test_profile_vision_medium_constants_match_battle_plan():
    p = get_profile("P0_mm_ite_vision_medium")
    assert p.vision_model == "mistral-medium-3-5"
    assert p.llm_model == "mistral-small-latest"
