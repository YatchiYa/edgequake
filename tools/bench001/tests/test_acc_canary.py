"""Unit tests for Acc canary fixture + thresholds."""

from bench001.acc_canary import (
    canary_fixture_path,
    canary_predictions,
    evaluate_canary_thresholds,
    load_canary,
)


def test_canary_fixture_loads():
    data = load_canary()
    assert data["fixture_id"] == "acc_canary_v1"
    assert canary_fixture_path().exists()
    kinds = {i["kind"] for i in data["items"]}
    assert kinds == {"paraphrase", "wrong_fact"}
    preds = canary_predictions(data)
    assert len(preds) == len(data["items"])
    assert all(p.get("generated_answer") for p in preds)


def test_canary_thresholds_pass_ideal():
    data = load_canary()
    detailed = []
    for item in data["items"]:
        if item["kind"] == "paraphrase":
            m = {"answer_correctness": 0.85, "factuality_f1": 0.8, "embed_cosine": 0.95}
        else:
            m = {"answer_correctness": 0.2, "factuality_f1": 0.05, "embed_cosine": 0.5}
        detailed.append({"id": item["id"], "metrics": m})
    metrics = {
        "raw": {"Fact Retrieval": {"detailed": detailed}},
        "overall_f1": 0.4,
        "overall_cos": 0.7,
    }
    report = evaluate_canary_thresholds(metrics, data)
    assert report["passed"] is True
    assert report["n_scored"] == len(data["items"])


def test_canary_thresholds_fail_inverted():
    data = load_canary()
    detailed = []
    for item in data["items"]:
        # Invert: paraphrases score low, wrong facts score high.
        if item["kind"] == "paraphrase":
            m = {"answer_correctness": 0.2, "factuality_f1": 0.1, "embed_cosine": 0.5}
        else:
            m = {"answer_correctness": 0.9, "factuality_f1": 0.8, "embed_cosine": 0.9}
        detailed.append({"id": item["id"], "metrics": m})
    metrics = {"raw": {"Fact Retrieval": {"detailed": detailed}}}
    report = evaluate_canary_thresholds(metrics, data)
    assert report["passed"] is False
    assert report["failures"]
