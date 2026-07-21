"""Unit tests for local ROUGE Acc proxy."""

from bench001.eval_score import rouge_l_f1, score_predictions_local, token_overlap_acc


def test_rouge_identical():
    assert rouge_l_f1("hello world", "hello world") == 1.0


def test_rouge_empty():
    assert rouge_l_f1("", "hello") == 0.0


def test_token_overlap_acc():
    assert token_overlap_acc("basal cell carcinoma", "Basal cell carcinoma (BCC)") == 1.0


def test_score_predictions_local():
    preds = [
        {
            "id": "1",
            "question_type": "Fact Retrieval",
            "generated_answer": "Basal cell carcinoma",
            "ground_truth": "Basal cell carcinoma (BCC) is the most common type of skin cancer.",
        }
    ]
    m = score_predictions_local(preds)
    assert m["judge"] == "rouge_proxy"
    assert m["n"] == 1
    assert "Fact Retrieval" in m["by_type"]
