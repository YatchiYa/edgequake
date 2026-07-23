"""Acc statistical helpers: bootstrap CI on dual-SUT deltas (SPEC-001 P15)."""

from __future__ import annotations

import random
from typing import Any


def _mean(xs: list[float]) -> float:
    return sum(xs) / len(xs) if xs else 0.0


def bootstrap_mean_ci(
    values: list[float],
    *,
    n_boot: int = 2000,
    alpha: float = 0.05,
    seed: int = 42,
) -> dict[str, float] | None:
    """Percentile bootstrap CI for the mean of ``values``."""
    if len(values) < 2:
        return None
    rng = random.Random(seed)
    n = len(values)
    means: list[float] = []
    for _ in range(n_boot):
        sample = [values[rng.randrange(n)] for _ in range(n)]
        means.append(_mean(sample))
    means.sort()
    lo_i = int(alpha / 2 * n_boot)
    hi_i = min(n_boot - 1, int((1 - alpha / 2) * n_boot))
    return {
        "mean": _mean(values),
        "ci_low": means[lo_i],
        "ci_high": means[hi_i],
        "n": float(n),
        "n_boot": float(n_boot),
        "alpha": alpha,
    }


def paired_delta_ci(
    eq_scores: list[float],
    lr_scores: list[float],
    *,
    n_boot: int = 2000,
    alpha: float = 0.05,
    seed: int = 42,
) -> dict[str, float] | None:
    """Bootstrap CI on mean(EQ − LR) for paired per-sample scores."""
    if len(eq_scores) != len(lr_scores) or len(eq_scores) < 2:
        return None
    deltas = [a - b for a, b in zip(eq_scores, lr_scores)]
    return bootstrap_mean_ci(deltas, n_boot=n_boot, alpha=alpha, seed=seed)


def extract_per_sample_metric(
    metrics: dict[str, Any] | None,
    key: str = "answer_correctness",
) -> list[float]:
    """Pull per-sample scores from official detailed eval (metrics['raw'])."""
    return list(extract_per_sample_metric_by_id(metrics, key).values())


def extract_per_sample_metric_by_id(
    metrics: dict[str, Any] | None,
    key: str = "answer_correctness",
) -> dict[str, float]:
    """Pull per-sample scores keyed by question id from official detailed eval."""
    if not metrics:
        return {}
    raw = metrics.get("raw") or {}
    out: dict[str, float] = {}
    for _qtype, block in raw.items():
        if not isinstance(block, dict):
            continue
        detailed = block.get("detailed") or []
        for row in detailed:
            if not isinstance(row, dict):
                continue
            qid = row.get("id")
            if not qid:
                continue
            m = row.get("metrics") or {}
            if key not in m:
                continue
            try:
                out[str(qid)] = float(m[key])
            except (TypeError, ValueError):
                continue
    return out


def components_present(metrics: dict[str, Any] | None) -> bool:
    """True when Acc decomposition (F1 + cos) is present at aggregate level."""
    if not metrics:
        return False
    f1 = metrics.get("overall_f1")
    cos = metrics.get("overall_cos")
    return f1 is not None and cos is not None


def delta_stats_block(
    eq_metrics: dict[str, Any] | None,
    lr_metrics: dict[str, Any] | None,
) -> dict[str, Any]:
    """Build delta CI block for Acc and F1 when per-sample detailed scores exist.

    Pairs on shared question ids so a single judge 429 / missing row does not
    drop the entire bootstrap CI (common on medical-mid n=200).
    """
    out: dict[str, Any] = {}
    for key, label in (
        ("answer_correctness", "overall_acc"),
        ("factuality_f1", "overall_f1"),
        ("embed_cosine", "overall_cos"),
    ):
        eq_by = extract_per_sample_metric_by_id(eq_metrics, key)
        lr_by = extract_per_sample_metric_by_id(lr_metrics, key)
        shared = sorted(set(eq_by) & set(lr_by))
        if len(shared) < 2:
            continue
        eq_s = [eq_by[i] for i in shared]
        lr_s = [lr_by[i] for i in shared]
        ci = paired_delta_ci(eq_s, lr_s)
        if ci is not None:
            ci = dict(ci)
            ci["n_paired"] = float(len(shared))
            ci["n_eq"] = float(len(eq_by))
            ci["n_lr"] = float(len(lr_by))
            out[f"{label}_delta_ci"] = ci
    return out