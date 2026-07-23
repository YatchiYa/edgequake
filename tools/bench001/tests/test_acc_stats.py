"""Unit tests for Acc bootstrap CI and component helpers."""

from bench001.acc_stats import (
    bootstrap_mean_ci,
    components_present,
    extract_per_sample_metric,
    paired_delta_ci,
)


def test_bootstrap_mean_ci_basic():
    ci = bootstrap_mean_ci([0.2, 0.3, 0.25, 0.28], n_boot=500, seed=1)
    assert ci is not None
    assert ci["ci_low"] <= ci["mean"] <= ci["ci_high"]
    assert ci["n"] == 4.0


def test_paired_delta_ci():
    eq = [0.3, 0.4, 0.35, 0.5]
    lr = [0.25, 0.35, 0.3, 0.45]
    ci = paired_delta_ci(eq, lr, n_boot=500, seed=2)
    assert ci is not None
    assert ci["mean"] > 0


def test_components_present():
    assert components_present({"overall_f1": 0.1, "overall_cos": 0.9})
    assert not components_present({"overall_acc": 0.24})
    assert not components_present(None)


def test_extract_per_sample_metric():
    metrics = {
        "raw": {
            "Fact Retrieval": {
                "detailed": [
                    {"id": "a", "metrics": {"answer_correctness": 0.5, "factuality_f1": 0.2}},
                    {"id": "b", "metrics": {"answer_correctness": 0.6, "factuality_f1": 0.3}},
                ]
            }
        }
    }
    accs = extract_per_sample_metric(metrics, "answer_correctness")
    f1s = extract_per_sample_metric(metrics, "factuality_f1")
    assert accs == [0.5, 0.6]
    assert f1s == [0.2, 0.3]


def test_delta_stats_pairs_on_shared_ids():
    """One missing EQ row must not drop the Acc bootstrap CI."""
    from bench001.acc_stats import delta_stats_block

    eq = {
        "raw": {
            "Fact Retrieval": {
                "detailed": [
                    {"id": "a", "metrics": {"answer_correctness": 0.5}},
                    {"id": "b", "metrics": {"answer_correctness": 0.6}},
                ]
            }
        }
    }
    lr = {
        "raw": {
            "Fact Retrieval": {
                "detailed": [
                    {"id": "a", "metrics": {"answer_correctness": 0.4}},
                    {"id": "b", "metrics": {"answer_correctness": 0.55}},
                    {"id": "c", "metrics": {"answer_correctness": 0.9}},
                ]
            }
        }
    }
    block = delta_stats_block(eq, lr)
    ci = block["overall_acc_delta_ci"]
    assert ci["n_paired"] == 2.0
    assert ci["n_eq"] == 2.0
    assert ci["n_lr"] == 3.0
    assert ci["ci_low"] <= ci["mean"] <= ci["ci_high"]
