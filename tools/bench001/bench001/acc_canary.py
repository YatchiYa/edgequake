"""Acc instrument canary gate (SPEC-001 P15) — judge-only validity check."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .paths import FIXTURES_DIR, stage_artifact_dir


CANARY_FIXTURE = "acc_canary_v1.json"


def canary_fixture_path() -> Path:
    return FIXTURES_DIR / CANARY_FIXTURE


def load_canary() -> dict[str, Any]:
    path = canary_fixture_path()
    data = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(data.get("items"), list) or len(data["items"]) < 4:
        raise ValueError(f"invalid canary fixture: {path}")
    return data


def canary_predictions(data: dict[str, Any] | None = None) -> list[dict[str, Any]]:
    data = data or load_canary()
    preds: list[dict[str, Any]] = []
    for item in data["items"]:
        preds.append(
            {
                "id": item["id"],
                "question": item["question"],
                "question_type": item.get("question_type") or "Fact Retrieval",
                "generated_answer": item["generated_answer"],
                "ground_truth": item["ground_truth"],
                "gold_answer": item["ground_truth"],
                "context": ["canary: no retrieval context required"],
                "evidence": [],
                "source": "canary",
                "canary_kind": item["kind"],
            }
        )
    return preds


def _sample_scores_from_metrics(metrics: dict[str, Any]) -> dict[str, dict[str, float]]:
    """Map sample id → {answer_correctness, factuality_f1, embed_cosine}."""
    out: dict[str, dict[str, float]] = {}
    raw = metrics.get("raw") or {}
    for _qtype, block in raw.items():
        if not isinstance(block, dict):
            continue
        for row in block.get("detailed") or []:
            if not isinstance(row, dict):
                continue
            sid = str(row.get("id") or "")
            m = row.get("metrics") or {}
            if not sid or "answer_correctness" not in m:
                continue
            out[sid] = {
                "answer_correctness": float(m["answer_correctness"]),
                "factuality_f1": float(m.get("factuality_f1", 0.0)),
                "embed_cosine": float(m.get("embed_cosine", 0.0)),
            }
    return out


def evaluate_canary_thresholds(
    metrics: dict[str, Any],
    data: dict[str, Any],
) -> dict[str, Any]:
    """Apply paraphrase/wrong-fact thresholds; return pass/fail report."""
    thr = data.get("thresholds") or {}
    para_min_acc = float(thr.get("paraphrase_min_acc", 0.7))
    para_min_f1 = float(thr.get("paraphrase_min_f1", 0.5))
    wrong_max_acc = float(thr.get("wrong_fact_max_acc", 0.4))
    wrong_max_f1 = float(thr.get("wrong_fact_max_f1", 0.2))

    by_id = {item["id"]: item for item in data["items"]}
    scores = _sample_scores_from_metrics(metrics)
    failures: list[str] = []
    details: list[dict[str, Any]] = []

    if not scores:
        return {
            "passed": False,
            "failures": ["no_per_sample_acc_components"],
            "details": [],
            "n_scored": 0,
        }

    for sid, item in by_id.items():
        sc = scores.get(sid)
        if sc is None:
            failures.append(f"missing_score:{sid}")
            continue
        kind = item["kind"]
        acc = sc["answer_correctness"]
        f1 = sc["factuality_f1"]
        ok = True
        reason = ""
        if kind == "paraphrase":
            if acc < para_min_acc or f1 < para_min_f1:
                ok = False
                reason = f"paraphrase Acc={acc:.3f}<{para_min_acc} or F1={f1:.3f}<{para_min_f1}"
        elif kind == "wrong_fact":
            if acc > wrong_max_acc or f1 > wrong_max_f1:
                ok = False
                reason = f"wrong_fact Acc={acc:.3f}>{wrong_max_acc} or F1={f1:.3f}>{wrong_max_f1}"
        else:
            ok = False
            reason = f"unknown_kind:{kind}"
        details.append({"id": sid, "kind": kind, **sc, "ok": ok, "reason": reason})
        if not ok:
            failures.append(f"{sid}:{reason}")

    return {
        "passed": not failures,
        "failures": failures,
        "details": details,
        "n_scored": len(scores),
        "thresholds": {
            "paraphrase_min_acc": para_min_acc,
            "paraphrase_min_f1": para_min_f1,
            "wrong_fact_max_acc": wrong_max_acc,
            "wrong_fact_max_f1": wrong_max_f1,
        },
    }


def run_acc_canary(*, eval_concurrency: int | None = None) -> dict[str, Any]:
    """Score canary predictions with active judge pins; write artifacts."""
    from .eval_score import try_official_generation_eval
    from .profiles import active_pins, eval_concurrency as eval_conc

    data = load_canary()
    preds = canary_predictions(data)
    art = stage_artifact_dir("acc-canary")
    out_path = art / "eval_canary.json"
    conc = eval_conc(eval_concurrency)
    metrics = try_official_generation_eval(
        preds, output_file=out_path, concurrency=conc
    )
    if metrics is None:
        report = {
            "passed": False,
            "failures": ["generation_eval_failed"],
            "details": [],
            "n_scored": 0,
            "pins": active_pins().lineage(),
        }
    else:
        report = evaluate_canary_thresholds(metrics, data)
        report["overall_acc"] = metrics.get("overall_acc")
        report["overall_f1"] = metrics.get("overall_f1")
        report["overall_cos"] = metrics.get("overall_cos")
        report["pins"] = active_pins().lineage()
        report["judge_model"] = active_pins().judge_model

    (art / "canary_report.json").write_text(
        json.dumps(report, indent=2), encoding="utf-8"
    )
    (art / "predictions_canary.json").write_text(
        json.dumps(preds, indent=2), encoding="utf-8"
    )
    lines = [
        "# SPEC-001 Acc canary",
        "",
        f"- **passed:** `{report['passed']}`",
        f"- **judge:** `{report.get('judge_model')}`",
        f"- **n_scored:** {report.get('n_scored')}",
        f"- **overall_acc / f1 / cos:** "
        f"{report.get('overall_acc')} / {report.get('overall_f1')} / {report.get('overall_cos')}",
        "",
    ]
    if report.get("failures"):
        lines.append("## Failures")
        lines.append("")
        for f in report["failures"]:
            lines.append(f"- {f}")
        lines.append("")
    (art / "SUMMARY.md").write_text("\n".join(lines), encoding="utf-8")
    return report
