"""Unit tests for vendored MMLongBench scoring semantics."""

from bench047.mmlongbench_eval_score import eval_acc_and_f1, eval_score


def test_int_exact():
    assert eval_score("3", "3", "Int") == 1.0
    assert eval_score("3", "4", "Int") == 0.0


def test_float_percentage():
    assert eval_score("18.29%", "18.29", "Float") == 1.0
    assert eval_score("18.29", "0.1829", "Float") == 1.0


def test_str_anls():
    assert eval_score("Less well-off", "less well-off", "Str") == 1.0


def test_unanswerable_f1_components():
    samples = [
        {"answer": "42", "pred": "42", "score": 1.0},
        {"answer": "Not answerable", "pred": "Not answerable", "score": 1.0},
        {"answer": "Not answerable", "pred": "something", "score": 0.0},
    ]
    acc, f1 = eval_acc_and_f1(samples)
    assert acc == 2.0 / 3.0
    assert f1 > 0.0
