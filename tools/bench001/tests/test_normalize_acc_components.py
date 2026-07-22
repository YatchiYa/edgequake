"""Normalize Acc F1/cos from official eval raw shape."""

from bench001.eval_score import _normalize_official


def test_normalize_official_with_components():
    raw = {
        "Fact Retrieval": {
            "average_scores": {
                "answer_correctness": 0.24,
                "factuality_f1": 0.05,
                "embed_cosine": 0.9,
                "rouge_score": 0.3,
            }
        },
        "Complex Reasoning": {
            "average_scores": {
                "answer_correctness": 0.26,
                "factuality_f1": 0.07,
                "embed_cosine": 0.92,
            }
        },
    }
    m = _normalize_official(raw)
    assert m is not None
    assert abs(m["overall_acc"] - 0.25) < 1e-9
    assert abs(m["overall_f1"] - 0.06) < 1e-9
    assert abs(m["overall_cos"] - 0.91) < 1e-9
    assert "factuality_f1" in m["by_type"]["Fact Retrieval"]
