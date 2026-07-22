"""Warm EQ workspace resolve / persist for make bench-warm."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from bench001.warm_workspace import (
    persist_warm_workspace,
    resolve_warm_workspace_id,
)


def test_persist_and_resolve_from_pointer(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    artifacts = tmp_path / "artifacts"
    artifacts.mkdir()
    monkeypatch.setattr("bench001.warm_workspace.ARTIFACTS_DIR", artifacts)
    monkeypatch.setattr("bench001.warm_workspace.WARM_POINTER", artifacts / "warm_workspace.json")
    monkeypatch.delenv("BENCH001_EQ_WORKSPACE_ID", raising=False)

    stage = artifacts / "smoke"
    wid = "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
    persist_warm_workspace(wid, stage_dir=stage, full_corpus=True)

    assert resolve_warm_workspace_id(prefer_env=False) == wid
    blob = json.loads((stage / "eq_workspace.json").read_text(encoding="utf-8"))
    assert blob["workspace_id"] == wid
    assert blob["full_corpus"] is True


def test_env_wins_over_pointer(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    artifacts = tmp_path / "artifacts"
    artifacts.mkdir()
    monkeypatch.setattr("bench001.warm_workspace.ARTIFACTS_DIR", artifacts)
    monkeypatch.setattr("bench001.warm_workspace.WARM_POINTER", artifacts / "warm_workspace.json")
    persist_warm_workspace("11111111-1111-1111-1111-111111111111", full_corpus=True)
    monkeypatch.setenv("BENCH001_EQ_WORKSPACE_ID", "22222222-2222-2222-2222-222222222222")
    assert resolve_warm_workspace_id(prefer_env=True) == "22222222-2222-2222-2222-222222222222"
    assert resolve_warm_workspace_id(prefer_env=False) == "11111111-1111-1111-1111-111111111111"


def test_rejects_capped_pointer(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    artifacts = tmp_path / "artifacts"
    hist = artifacts / "history"
    hist.mkdir(parents=True)
    pointer = artifacts / "warm_workspace.json"
    monkeypatch.setattr("bench001.warm_workspace.ARTIFACTS_DIR", artifacts)
    monkeypatch.setattr("bench001.warm_workspace.WARM_POINTER", pointer)
    monkeypatch.setattr("bench001.warm_workspace.history_root", lambda: hist)
    monkeypatch.delenv("BENCH001_EQ_WORKSPACE_ID", raising=False)
    pointer.write_text(
        json.dumps({"workspace_id": "cccccccc-cccc-cccc-cccc-cccccccccccc", "full_corpus": False}),
        encoding="utf-8",
    )
    assert resolve_warm_workspace_id(prefer_env=False) is None


def test_ablation_note_markdown_bold(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    artifacts = tmp_path / "artifacts"
    hist = artifacts / "history" / "smoke-20260719T155350Z"
    hist.mkdir(parents=True)
    (hist / "ABLATION_NOTE.md").write_text(
        "**Warm workspace:** `8b359190-0733-4949-994c-f39eca074d79` (query-only)\n",
        encoding="utf-8",
    )
    monkeypatch.setattr("bench001.warm_workspace.ARTIFACTS_DIR", artifacts)
    monkeypatch.setattr("bench001.warm_workspace.WARM_POINTER", artifacts / "warm_workspace.json")
    monkeypatch.setattr("bench001.warm_workspace.history_root", lambda: artifacts / "history")
    monkeypatch.delenv("BENCH001_EQ_WORKSPACE_ID", raising=False)
    assert resolve_warm_workspace_id(prefer_env=False) == "8b359190-0733-4949-994c-f39eca074d79"


def test_repo_seed_pointer_resolves():
    """Seeded warm_workspace.json must resolve without env (B3b identity+packing Acc warm)."""
    import os

    os.environ.pop("BENCH001_EQ_WORKSPACE_ID", None)
    wid = resolve_warm_workspace_id(prefer_env=False)
    # Acc warm pointer tracks the last *valid* full-corpus WS (B2 until B3b promotes).
    assert wid == "2a7bcb2f-b156-4c49-9229-67f5bcde22a4"
