"""Unit tests for Acc env hygiene (FAKE key scrub + start.sh reload)."""

from __future__ import annotations

from pathlib import Path

from bench001.acc_env import (
    apply_acc_publication_pins,
    apply_acc_speed_defaults,
    assert_publication_ingest,
    backend_pin_mismatches,
    ensure_acc_api_keys,
    is_placeholder_api_key,
    load_mistral_key_from_start_sh,
    scrub_placeholder_api_keys,
)


def test_placeholder_detection():
    assert is_placeholder_api_key(None)
    assert is_placeholder_api_key("")
    assert is_placeholder_api_key("FAKESECRET")
    assert is_placeholder_api_key("fake-key")
    assert not is_placeholder_api_key("4X1dx9s66LHvRI3W8LNY77A1dT94cxqr")


def test_scrub_clears_fake(monkeypatch):
    env = {
        "LLM_API_KEY": "FAKESECRET",
        "MISTRAL_API_KEY": "real-mistral-key-xxxxxxxxxxxx",
        "OPENAI_API_KEY": "FAKEOPENAI",
    }
    cleared = scrub_placeholder_api_keys(env=env)
    assert "LLM_API_KEY" in cleared
    assert "OPENAI_API_KEY" in cleared
    assert "MISTRAL_API_KEY" not in cleared
    assert env["MISTRAL_API_KEY"].startswith("real-")


def test_load_key_from_start_sh(tmp_path: Path):
    sh = tmp_path / "edgequake-start.sh"
    sh.write_text(
        '#!/bin/bash\nexport MISTRAL_API_KEY="real-from-start-sh-zzzzzzzz"\n'
        'export LLM_API_KEY="FAKESECRET"\nexec /bin/true\n',
        encoding="utf-8",
    )
    assert load_mistral_key_from_start_sh(paths=(sh,)) == "real-from-start-sh-zzzzzzzz"


def test_ensure_acc_api_keys_reloads_from_start_sh(monkeypatch, tmp_path: Path):
    monkeypatch.setenv("LLM_API_KEY", "FAKESECRET")
    monkeypatch.delenv("MISTRAL_API_KEY", raising=False)
    monkeypatch.delenv("OPENAI_API_KEY", raising=False)
    sh = tmp_path / "start.sh"
    sh.write_text(
        'export MISTRAL_API_KEY="reloaded-mistral-key-yyyyyyyy"\n',
        encoding="utf-8",
    )
    monkeypatch.setattr(
        "bench001.acc_env.START_SH_CANDIDATES",
        (sh,),
    )
    assert ensure_acc_api_keys(verbose=False) is True
    import os

    assert os.environ["MISTRAL_API_KEY"] == "reloaded-mistral-key-yyyyyyyy"
    assert os.environ["LLM_API_KEY"] == "reloaded-mistral-key-yyyyyyyy"
    assert not is_placeholder_api_key(os.environ["MISTRAL_API_KEY"])


def test_apply_acc_speed_defaults_clears_round_robin(monkeypatch):
    monkeypatch.setenv("EDGEQUAKE_MIX_FUSION", "round_robin")
    monkeypatch.delenv("BENCH001_ALLOW_ROUND_ROBIN", raising=False)
    apply_acc_speed_defaults()
    import os

    assert os.environ["EDGEQUAKE_MIX_FUSION"] == "rrf"


def test_apply_acc_publication_pins_full_corpus(monkeypatch):
    monkeypatch.setenv("BENCH001_INGEST_MAX_CHARS", "100000")
    monkeypatch.setenv("EDGEQUAKE_VISION_PROVIDER", "ollama")
    monkeypatch.setenv("EDGEQUAKE_VISION_MODEL", "gemma4:latest")
    monkeypatch.setenv("BENCH001_EQ_WORKSPACE_ID", "ws-c100000-old")
    # Contaminated shell from T011703Z-era soft path must be overwritten when unset… 
    # (preserve_if_set keeps explicit overrides; clear so publication wins)
    monkeypatch.delenv("EDGEQUAKE_PATH_PRUNE", raising=False)
    monkeypatch.delenv("EDGEQUAKE_PATH_PRUNE_FRACTION", raising=False)
    apply_acc_publication_pins(full_corpus=True, verbose=False)
    import os

    assert os.environ["BENCH001_INGEST_MAX_CHARS"] == "0"
    assert os.environ["BENCH001_PUBLICATION"] == "1"
    assert os.environ["EDGEQUAKE_VISION_PROVIDER"] == "mistral"
    assert os.environ["EDGEQUAKE_VISION_MODEL"] == "mistral-small-latest"
    assert os.environ["EDGEQUAKE_EMBEDDING_MODEL"] == "mistral-embed"
    assert os.environ["EDGEQUAKE_CHUNK_SIZE"] == "1200"
    assert os.environ["EDGEQUAKE_PATH_PRUNE"] == "0"
    assert os.environ["EDGEQUAKE_PATH_PRUNE_FRACTION"] == "0"
    assert os.environ["EDGEQUAKE_POPULAR_NODE_FALLBACK"] == "0"
    assert os.environ["EDGEQUAKE_QUERY_ARM_CONCURRENCY"] == "16"
    assert "BENCH001_EQ_WORKSPACE_ID" not in os.environ


def test_path_prune_fraction_pin_respects_path_off(monkeypatch):
    from bench001.fair_pins import _path_prune_fraction_pin

    monkeypatch.setenv("EDGEQUAKE_PATH_PRUNE", "0")
    monkeypatch.setenv("EDGEQUAKE_PATH_PRUNE_FRACTION", "0.4")
    assert _path_prune_fraction_pin() == 0.0
    monkeypatch.setenv("EDGEQUAKE_PATH_PRUNE", "1")
    assert _path_prune_fraction_pin() == 0.4


def test_backend_pin_mismatches(monkeypatch):
    bad = backend_pin_mismatches(
        {
            "providers": {
                "llm": {"name": "ollama", "model": "gemma3"},
                "embedding": {"name": "mistral", "model": "mistral-embed"},
            }
        }
    )
    assert any("llm_provider" in m for m in bad)
    ok = backend_pin_mismatches(
        {
            "providers": {
                "llm": {"name": "mistral", "model": "mistral-small-latest"},
                "embedding": {"name": "mistral", "model": "mistral-embed"},
            }
        }
    )
    assert ok == []
    monkeypatch.delenv("BENCH001_ALLOW_ROUND_ROBIN", raising=False)
    rr_bad = backend_pin_mismatches(
        {
            "providers": {
                "llm": {"name": "mistral", "model": "mistral-small-latest"},
                "embedding": {"name": "mistral", "model": "mistral-embed"},
            },
            "operational": {"query_engine": {"mix_fusion": "round_robin"}},
        }
    )
    assert any("mix_fusion" in m for m in rr_bad)
    monkeypatch.setenv("BENCH001_ALLOW_ROUND_ROBIN", "1")
    rr_ok = backend_pin_mismatches(
        {
            "providers": {
                "llm": {"name": "mistral", "model": "mistral-small-latest"},
                "embedding": {"name": "mistral", "model": "mistral-embed"},
            },
            "operational": {"query_engine": {"mix_fusion": "round_robin"}},
        }
    )
    assert rr_ok == []


def test_assert_publication_ingest_fail_closed(monkeypatch):
    monkeypatch.setenv("BENCH001_PUBLICATION", "1")
    try:
        assert_publication_ingest({"ingest_capped": True, "ingest_max_chars": 100000})
        raise AssertionError("expected RuntimeError")
    except RuntimeError as exc:
        assert "forbids capped" in str(exc)
