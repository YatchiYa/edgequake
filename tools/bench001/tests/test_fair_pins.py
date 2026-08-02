"""Publishable fairness pins + retrieval JSON normalization."""

from __future__ import annotations

import json
from pathlib import Path

from bench001.eval_score import _to_retrieval_eval_json
from bench001.fair_pins import (
    FAIR_CHUNK_TOKEN_SIZE,
    PUBLISH_PROFILE_ID,
    adaptive_chunking_enabled,
    chunk_overlap_token_size,
    chunk_token_size,
    eq_query_overrides,
    lr_query_param_overrides,
    mix_arm_gate_enabled,
    publish_fairness_enabled,
    publish_pin_fields,
    resolve_publish_profile_id,
    retrieve_topk,
)
from bench001.profiles import PROFILE_ID_DEFAULT, resolve_pins

PUBLISH_PROFILE_ID_LRLIKE = "P0_mistral_mix_lrlike_arms_v2"


def test_publish_fairness_defaults(monkeypatch):
    monkeypatch.delenv("BENCH001_PUBLISH_FAIRNESS", raising=False)
    monkeypatch.delenv("BENCH001_RETRIEVE_TOPK", raising=False)
    monkeypatch.delenv("EDGEQUAKE_MIX_ARM_GATE", raising=False)
    assert publish_fairness_enabled() is True
    assert retrieve_topk() == 30
    assert mix_arm_gate_enabled() is False  # LR-like always-on arms under publish fairness
    lr = lr_query_param_overrides()
    assert lr["top_k"] == 30
    assert lr["chunk_top_k"] == 30
    assert lr["enable_rerank"] is False
    eq = eq_query_overrides()
    assert eq["max_results"] == 30
    assert eq["rerank_top_k"] == 30
    # SPEC-086 Acc law default: post-fuse rerank OFF
    assert eq["enable_rerank"] is False
    monkeypatch.setenv("BENCH001_EQ_ENABLE_RERANK", "1")
    assert eq_query_overrides()["enable_rerank"] is True
    monkeypatch.setenv("BENCH001_EQ_ENABLE_RERANK", "0")
    pins = publish_pin_fields()
    assert pins["mix_arm_gate"] is False
    assert pins["eq_enable_rerank"] is False


def test_profile_bumps_to_v2_lrlike(monkeypatch):
    monkeypatch.delenv("BENCH001_PROFILE_ID", raising=False)
    monkeypatch.delenv("EDGEQUAKE_MIX_ARM_GATE", raising=False)
    monkeypatch.setenv("BENCH001_PUBLISH_FAIRNESS", "1")
    pins = resolve_pins()
    assert pins.profile_id == PUBLISH_PROFILE_ID_LRLIKE
    assert resolve_publish_profile_id(PROFILE_ID_DEFAULT) == PUBLISH_PROFILE_ID_LRLIKE


def test_profile_keeps_v2_when_arm_gate_on(monkeypatch):
    monkeypatch.delenv("BENCH001_PROFILE_ID", raising=False)
    monkeypatch.setenv("BENCH001_PUBLISH_FAIRNESS", "1")
    monkeypatch.setenv("EDGEQUAKE_MIX_ARM_GATE", "true")
    assert mix_arm_gate_enabled() is True
    assert resolve_publish_profile_id(PROFILE_ID_DEFAULT) == PUBLISH_PROFILE_ID


def test_legacy_fairness_off(monkeypatch):
    monkeypatch.setenv("BENCH001_PUBLISH_FAIRNESS", "0")
    monkeypatch.delenv("BENCH001_RETRIEVE_TOPK", raising=False)
    monkeypatch.delenv("EDGEQUAKE_MIX_ARM_GATE", raising=False)
    assert publish_fairness_enabled() is False
    assert retrieve_topk() == 5
    assert resolve_publish_profile_id(PROFILE_ID_DEFAULT) == PROFILE_ID_DEFAULT
    assert mix_arm_gate_enabled() is True  # production default when fairness off


def test_fair_chunk_pins_under_publish_fairness(monkeypatch):
    monkeypatch.delenv("EDGEQUAKE_ADAPTIVE_CHUNKING", raising=False)
    monkeypatch.delenv("EDGEQUAKE_CHUNK_SIZE", raising=False)
    monkeypatch.delenv("EDGEQUAKE_CHUNK_OVERLAP", raising=False)
    monkeypatch.setenv("BENCH001_PUBLISH_FAIRNESS", "1")
    assert adaptive_chunking_enabled() is False
    assert chunk_token_size() == FAIR_CHUNK_TOKEN_SIZE
    assert chunk_overlap_token_size() == 100
    pins = publish_pin_fields()
    assert pins["adaptive_chunking"] is False
    assert pins["chunk_token_size"] == 1200
    assert pins["chunk_overlap_token_size"] == 100


def test_adaptive_chunking_env_override(monkeypatch):
    monkeypatch.setenv("BENCH001_PUBLISH_FAIRNESS", "1")
    monkeypatch.setenv("EDGEQUAKE_ADAPTIVE_CHUNKING", "1")
    monkeypatch.setenv("EDGEQUAKE_CHUNK_SIZE", "800")
    assert adaptive_chunking_enabled() is True
    assert chunk_token_size() == 800


def test_retrieval_eval_json_includes_evidence(tmp_path: Path):
    preds = [
        {
            "id": "Medical-1",
            "question": "q?",
            "question_type": "Fact Retrieval",
            "context": ["chunk a", "chunk b"],
            "evidence": ["gold span"],
        }
    ]
    out = tmp_path / "r.json"
    _to_retrieval_eval_json(preds, out)
    rows = json.loads(out.read_text())
    assert rows[0]["evidence"] == ["gold span"]
    assert rows[0]["context"] == ["chunk a", "chunk b"]
