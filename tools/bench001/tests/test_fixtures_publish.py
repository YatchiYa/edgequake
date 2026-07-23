"""Fixture stratification + medical-publish ladder tests."""

from __future__ import annotations

from collections import Counter

import pytest

from bench001.fixtures import (
    freeze_publish_verify,
    freeze_smoke_verify,
    read_ids,
    select_questions,
    verify_fixtures,
)
from bench001.paths import (
    MEDICAL_ONLY_FIXTURES,
    PUBLISH_FIXTURE,
    QUESTION_TYPES,
    SMOKE_FIXTURE,
    fixture_path,
    questions_path,
)


def test_publish_fixture_file_exists_and_counts():
    assert fixture_path(PUBLISH_FIXTURE).exists()
    ids = read_ids(PUBLISH_FIXTURE)
    assert len(ids) == 200
    assert len(set(ids)) == 200
    smoke = set(read_ids(SMOKE_FIXTURE))
    assert smoke.issubset(set(ids))
    assert PUBLISH_FIXTURE in MEDICAL_ONLY_FIXTURES


@pytest.mark.skipif(
    not questions_path("medical").exists(),
    reason="GraphRAG-Bench medical questions not cached",
)
def test_publish_stratification_50_per_type():
    qs = select_questions(PUBLISH_FIXTURE)
    counts = Counter(q["question_type"] for q in qs)
    assert len(qs) == 200
    for t in QUESTION_TYPES:
        assert counts[t] == 50, counts


@pytest.mark.skipif(
    not questions_path("medical").exists(),
    reason="GraphRAG-Bench medical questions not cached",
)
def test_freeze_smoke_and_publish_verify():
    freeze_smoke_verify()
    freeze_publish_verify()


def test_verify_fixtures_includes_publish():
    fx = verify_fixtures()
    assert fx.get("publish_exists") is True
    assert fx.get("publish_n") == 200
    assert fx.get("smoke_n") == 40
