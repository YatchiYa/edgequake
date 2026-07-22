"""Unit tests for SPEC-001 query concurrency pin."""

from __future__ import annotations

import os

from bench001.profiles import (
    DEFAULT_EVAL_CONCURRENCY,
    DEFAULT_QUERY_CONCURRENCY,
    MAX_EVAL_CONCURRENCY,
    MAX_QUERY_CONCURRENCY,
    eval_concurrency,
    query_concurrency,
)


def test_query_concurrency_default(monkeypatch):
    monkeypatch.delenv("BENCH001_QUERY_CONCURRENCY", raising=False)
    assert query_concurrency() == DEFAULT_QUERY_CONCURRENCY


def test_query_concurrency_env(monkeypatch):
    monkeypatch.setenv("BENCH001_QUERY_CONCURRENCY", "12")
    assert query_concurrency() == 12


def test_query_concurrency_override_beats_env(monkeypatch):
    monkeypatch.setenv("BENCH001_QUERY_CONCURRENCY", "12")
    assert query_concurrency(4) == 4


def test_query_concurrency_clamps(monkeypatch):
    monkeypatch.setenv("BENCH001_QUERY_CONCURRENCY", "0")
    assert query_concurrency() == 1
    monkeypatch.setenv("BENCH001_QUERY_CONCURRENCY", "999")
    assert query_concurrency() == MAX_QUERY_CONCURRENCY
    assert query_concurrency(-3) == 1


def test_query_concurrency_bad_env(monkeypatch):
    monkeypatch.setenv("BENCH001_QUERY_CONCURRENCY", "nope")
    assert query_concurrency() == DEFAULT_QUERY_CONCURRENCY


def test_eval_concurrency_default(monkeypatch):
    monkeypatch.delenv("BENCH001_EVAL_CONCURRENCY", raising=False)
    assert eval_concurrency() == DEFAULT_EVAL_CONCURRENCY
    assert DEFAULT_EVAL_CONCURRENCY >= 16
    assert MAX_EVAL_CONCURRENCY >= 64


def test_eval_concurrency_env_and_clamp(monkeypatch):
    monkeypatch.setenv("BENCH001_EVAL_CONCURRENCY", "24")
    assert eval_concurrency() == 24
    assert eval_concurrency(99) == MAX_EVAL_CONCURRENCY
