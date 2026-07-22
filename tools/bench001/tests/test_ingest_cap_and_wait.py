"""Fast Acc ingest cap + fail-fast wait_document."""

from __future__ import annotations

import json

import httpx
import pytest

from bench001.client import EdgeQuakeClient
from bench001.ingest_cap import (
    apply_ingest_cap,
    eq_workspace_name_for_cap,
    ingest_max_chars,
    lr_stage_for_cap,
)


def test_apply_ingest_cap_truncates(monkeypatch):
    monkeypatch.setenv("BENCH001_INGEST_MAX_CHARS", "100")
    texts, meta = apply_ingest_cap(["a" * 500 + " end"])
    assert meta["ingest_capped"] is True
    assert meta["ingest_max_chars"] == 100
    assert len(texts[0]) <= 101
    assert texts[0].endswith("\n")


def test_apply_ingest_cap_unlimited(monkeypatch):
    monkeypatch.setenv("BENCH001_INGEST_MAX_CHARS", "0")
    assert ingest_max_chars() is None
    texts, meta = apply_ingest_cap(["hello" * 20])
    assert meta["ingest_capped"] is False
    assert texts[0] == "hello" * 20


def test_stage_and_workspace_isolation(monkeypatch):
    monkeypatch.setenv("BENCH001_INGEST_MAX_CHARS", "100000")
    assert lr_stage_for_cap("smoke") == "smoke_c100000"
    assert eq_workspace_name_for_cap("bench001-smoke") == "bench001-smoke-c100000"
    monkeypatch.setenv("BENCH001_INGEST_MAX_CHARS", "0")
    assert lr_stage_for_cap("smoke") == "smoke"


def test_wait_document_fails_fast_on_failed_status(httpx_mock):
    client = EdgeQuakeClient("http://eq.test")
    doc_id = "doc-1"
    task_id = "task-1"
    httpx_mock.add_response(
        url=f"http://eq.test/api/v1/tasks/{task_id}",
        json={"status": "processing"},
    )
    httpx_mock.add_response(
        url=f"http://eq.test/api/v1/documents/{doc_id}",
        json={
            "id": doc_id,
            "status": "failed",
            "display_status": "failed",
            "current_stage": "failed",
            "ui_phase": "terminal",
            "stage_message": "Graph merge duplicate key",
            "stage_progress": 1.0,
        },
    )
    with pytest.raises(RuntimeError, match="failed"):
        client.wait_document(doc_id, task_id=task_id, timeout_s=5.0, poll_s=0.01)


def test_wait_document_rejects_indexed_with_storage_errors(httpx_mock):
    """032 B3b: indexed + storage_error_count must fail closed (saga rollback)."""
    client = EdgeQuakeClient("http://eq.test")
    doc_id = "doc-enospace"
    httpx_mock.add_response(
        url=f"http://eq.test/api/v1/documents/{doc_id}",
        json={
            "id": doc_id,
            "status": "indexed",
            "display_status": "indexed",
            "current_stage": "done",
            "ui_phase": "terminal",
            "stage_progress": 1.0,
            "storage_error_count": 1,
            "warning_message": "Knowledge graph persist failed: No space left on device",
            "chunk_count": 188,
        },
    )
    with pytest.raises(RuntimeError, match="storage_error_count"):
        client.wait_document(doc_id, timeout_s=5.0, poll_s=0.01)


def test_wait_document_heartbeat_callback(httpx_mock):
    client = EdgeQuakeClient("http://eq.test")
    doc_id = "doc-2"
    # First poll: in-flight storing
    httpx_mock.add_response(
        url=f"http://eq.test/api/v1/documents/{doc_id}",
        json={
            "id": doc_id,
            "status": "indexing",
            "display_status": "storing",
            "current_stage": "storing",
            "ui_phase": "running",
            "stage_progress": 0.4,
            "stage_message": "Merging relationships 40%",
            "chunk_count": 20,
        },
    )
    # Second poll: completed
    httpx_mock.add_response(
        url=f"http://eq.test/api/v1/documents/{doc_id}",
        json={
            "id": doc_id,
            "status": "completed",
            "display_status": "completed",
            "current_stage": "done",
            "ui_phase": "terminal",
            "stage_progress": 1.0,
            "chunk_count": 20,
        },
    )
    ticks: list[dict] = []

    # Force immediate heartbeat by backdating last_log via tiny timeout path:
    # call with poll_s small; first loop logs because last_log=0.
    out = client.wait_document(
        doc_id,
        timeout_s=30.0,
        poll_s=0.01,
        progress_cb=ticks.append,
    )
    assert out["status"] == "completed"
    assert ticks, "expected at least one progress callback"
    assert ticks[0].get("pct") == pytest.approx(0.4)
