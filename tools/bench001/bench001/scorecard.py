"""Build scorecard.json + SUMMARY.md for SPEC-001."""

from __future__ import annotations

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .paths import DATASET_ID, DATASET_REVISION, REPO_ROOT
from .profiles import ProviderPins, active_pins, pin_block
from .progress import empty_context_rate


def _banner(pins: ProviderPins) -> str:
    sut = f"{pins.llm_provider}/{pins.llm_model}"
    emb = f"{pins.embedding_provider}/{pins.embedding_model}"
    return (
        f"EQ mix vs LR mix on GraphRAG-Bench ({sut} + {emb}) — "
        "publishable dual-SUT under matched top-k + L2 retrieval metrics. "
        "Not UltraDomain win-rates; Acc is not paper Table-2 comparable unless "
        "P0_paper ablation pins are used."
    )


def git_sha() -> str:
    try:
        return (
            subprocess.check_output(
                ["git", "rev-parse", "--short", "HEAD"],
                cwd=REPO_ROOT,
                text=True,
            ).strip()
        )
    except Exception:  # noqa: BLE001
        return "unknown"


def empty_rate(preds: list[dict[str, Any]]) -> float:
    if not preds:
        return 1.0
    empty = sum(1 for p in preds if not (p.get("generated_answer") or "").strip())
    return empty / len(preds)


def latency_percentile(preds: list[dict[str, Any]], p: float) -> int:
    vals = sorted(int(p.get("latency_ms") or 0) for p in preds)
    if not vals:
        return 0
    idx = min(len(vals) - 1, max(0, int(round((p / 100.0) * (len(vals) - 1)))))
    return vals[idx]


def stage_percentile(preds: list[dict[str, Any]], field: str, p: float) -> int | None:
    """021 F3: p50/p95 for retrieve/rerank/generate when present on predictions."""
    vals = []
    for pred in preds:
        raw = pred.get(field)
        if raw is None:
            continue
        try:
            vals.append(int(raw))
        except (TypeError, ValueError):
            continue
    if not vals:
        return None
    vals.sort()
    idx = min(len(vals) - 1, max(0, int(round((p / 100.0) * (len(vals) - 1)))))
    return vals[idx]


def build_scorecard(
    *,
    stage: str,
    fixture_id: str,
    eq_metrics: dict[str, Any] | None,
    lr_metrics: dict[str, Any] | None,
    eq_preds: list[dict[str, Any]],
    lr_preds: list[dict[str, Any]],
    valid: bool,
    invalid_reason: str | None,
    judge: str,
    ingest_wall_s: float = 0.0,
    profile_id: str | None = None,
    retrieve_topk: int = 5,
    provider_pins: ProviderPins | None = None,
) -> dict[str, Any]:
    from .fair_pins import retrieve_topk as fair_topk

    eq_acc = float((eq_metrics or {}).get("overall_acc") or 0.0)
    lr_acc = float((lr_metrics or {}).get("overall_acc") or 0.0)
    pp = provider_pins or active_pins()
    pins = pin_block(
        fixture_id=fixture_id,
        judge=judge,
        git_sha=git_sha(),
        dataset_id=DATASET_ID,
        dataset_revision=DATASET_REVISION,
        pins=pp,
    )
    pins["retrieve_topk"] = int(retrieve_topk) if retrieve_topk else fair_topk()

    def _sut_block(m: dict[str, Any] | None) -> dict[str, Any]:
        block: dict[str, Any] = {
            "overall_acc": float((m or {}).get("overall_acc") or 0.0),
            "by_type": (m or {}).get("by_type") or {},
        }
        if (m or {}).get("overall_f1") is not None:
            block["overall_f1"] = float(m["overall_f1"])  # type: ignore[index]
        if (m or {}).get("overall_cos") is not None:
            block["overall_cos"] = float(m["overall_cos"])  # type: ignore[index]
        ret = (m or {}).get("retrieval") or {}
        if ret:
            block["retrieval"] = {
                "overall_context_relevancy": ret.get("overall_context_relevancy"),
                "overall_evidence_recall": ret.get("overall_evidence_recall"),
                "by_type": ret.get("by_type") or {},
            }
        return block

    eq_block = _sut_block(eq_metrics)
    lr_block = _sut_block(lr_metrics)
    delta: dict[str, Any] = {"overall_acc": eq_acc - lr_acc}
    if eq_block.get("overall_f1") is not None and lr_block.get("overall_f1") is not None:
        delta["overall_f1"] = float(eq_block["overall_f1"]) - float(lr_block["overall_f1"])
    if eq_block.get("overall_cos") is not None and lr_block.get("overall_cos") is not None:
        delta["overall_cos"] = float(eq_block["overall_cos"]) - float(lr_block["overall_cos"])
    eq_rec = (eq_block.get("retrieval") or {}).get("overall_evidence_recall")
    lr_rec = (lr_block.get("retrieval") or {}).get("overall_evidence_recall")
    if eq_rec is not None and lr_rec is not None:
        delta["overall_evidence_recall"] = float(eq_rec) - float(lr_rec)
    eq_rel = (eq_block.get("retrieval") or {}).get("overall_context_relevancy")
    lr_rel = (lr_block.get("retrieval") or {}).get("overall_context_relevancy")
    if eq_rel is not None and lr_rel is not None:
        delta["overall_context_relevancy"] = float(eq_rel) - float(lr_rel)

    from .acc_stats import components_present, delta_stats_block

    delta.update(delta_stats_block(eq_metrics, lr_metrics))
    ops: dict[str, Any] = {
        "n_questions": max(len(eq_preds), len(lr_preds)),
        "eq_empty_answer_rate": empty_rate(eq_preds),
        "lr_empty_answer_rate": empty_rate(lr_preds),
        "eq_empty_context_rate": empty_context_rate(eq_preds),
        "lr_empty_context_rate": empty_context_rate(lr_preds),
        "eq_query_p50_ms": latency_percentile(eq_preds, 50),
        "eq_query_p95_ms": latency_percentile(eq_preds, 95),
        "lr_query_p50_ms": latency_percentile(lr_preds, 50),
        "lr_query_p95_ms": latency_percentile(lr_preds, 95),
        "ingest_wall_s": ingest_wall_s,
        "acc_components_present": components_present(eq_metrics)
        and components_present(lr_metrics),
    }
    # 021 F3 — stage split when EQ API returns QueryStats on predictions.
    for field, op_key in (
        ("retrieval_time_ms", "eq_retrieve_p50_ms"),
        ("rerank_time_ms", "eq_rerank_p50_ms"),
        ("generation_time_ms", "eq_generate_p50_ms"),
        ("embedding_time_ms", "eq_embed_p50_ms"),
        ("keyword_time_ms", "eq_keyword_p50_ms"),
    ):
        p50 = stage_percentile(eq_preds, field, 50)
        if p50 is not None:
            ops[op_key] = p50
    eq_p50 = ops["eq_query_p50_ms"]
    lr_p50 = ops["lr_query_p50_ms"]
    if lr_p50 > 0:
        ops["eq_over_lr_p50_ratio"] = round(eq_p50 / lr_p50, 3)
        ops["latency_slo_1_5x"] = eq_p50 <= int(1.5 * lr_p50)

    return {
        "spec": "001",
        "stage": stage,
        "valid": valid,
        "invalid_reason": invalid_reason,
        "task_name": "GraphRAG-Bench/EQ-vs-LR",
        "banner": _banner(pp),
        "profile_id": profile_id or pp.profile_id,
        "pins": pins,
        "metrics": {
            "eq": eq_block,
            "lr": lr_block,
            "delta_eq_minus_lr": delta,
        },
        "ops": ops,
        "created_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    }


def _fmt(v: Any) -> str:
    if v is None:
        return "—"
    try:
        return f"{float(v):.4f}"
    except (TypeError, ValueError):
        return "—"


def write_summary(scorecard: dict[str, Any], path: Path) -> None:
    m = scorecard["metrics"]
    lines = [
        f"# SPEC-001 {scorecard['stage']} SUMMARY",
        "",
        f"> {scorecard['banner']}",
        "",
        f"- **valid:** `{scorecard['valid']}`"
        + (f" ({scorecard['invalid_reason']})" if scorecard.get("invalid_reason") else ""),
        f"- **profile:** `{scorecard['profile_id']}`",
        f"- **judge:** `{scorecard['pins']['judge']}`",
        f"- **fixture:** `{scorecard['pins']['fixture_id']}` (n={scorecard['ops']['n_questions']})",
        f"- **dataset revision:** `{scorecard['pins']['dataset_revision']}`",
        "",
        "## Model lineage",
        "",
    ]
    lineage = (scorecard.get("pins") or {}).get("lineage") or {}
    if lineage:
        for k, v in lineage.items():
            lines.append(f"- **{k}:** `{v}`")
    else:
        p = scorecard.get("pins") or {}
        lines.extend(
            [
                f"- **sut_llm:** `{p.get('llm_provider')}/{p.get('llm_model')}`",
                f"- **sut_vision:** `{p.get('vision_provider')}/{p.get('vision_model')}`",
                f"- **sut_embed:** `{p.get('embedding_provider')}/{p.get('embedding_model')}`",
                f"- **judge_llm:** `{p.get('judge_provider', p.get('llm_provider'))}/{p.get('judge_model', p.get('llm_model'))}`",
            ]
        )
    eq_ret = (m["eq"].get("retrieval") or {})
    lr_ret = (m["lr"].get("retrieval") or {})
    eq_f1 = m["eq"].get("overall_f1")
    lr_f1 = m["lr"].get("overall_f1")
    eq_cos = m["eq"].get("overall_cos")
    lr_cos = m["lr"].get("overall_cos")
    d = m["delta_eq_minus_lr"]
    lines.extend(
        [
            "",
            "## Overall Acc (L0) — Acc = 0.75·F1 + 0.25·cos",
            "",
            "| SUT | Acc | F1 | cos |",
            "|-----|-----|----|-----|",
            f"| EdgeQuake mix | {m['eq']['overall_acc']:.4f} | {_fmt(eq_f1)} | {_fmt(eq_cos)} |",
            f"| LightRAG mix | {m['lr']['overall_acc']:.4f} | {_fmt(lr_f1)} | {_fmt(lr_cos)} |",
            f"| Δ (EQ − LR) | {d['overall_acc']:+.4f} | {_fmt(d.get('overall_f1'))} | {_fmt(d.get('overall_cos'))} |",
            "",
        ]
    )
    acc_ci = d.get("overall_acc_delta_ci") or {}
    if acc_ci:
        lines.extend(
            [
                f"- **Δ Acc 95% CI (bootstrap):** "
                f"[{acc_ci.get('ci_low', 0):+.4f}, {acc_ci.get('ci_high', 0):+.4f}] "
                f"(n={int(acc_ci.get('n', 0))})",
                "",
            ]
        )
    if eq_ret or lr_ret:
        eq_er = eq_ret.get("overall_evidence_recall")
        lr_er = lr_ret.get("overall_evidence_recall")
        eq_cr = eq_ret.get("overall_context_relevancy")
        lr_cr = lr_ret.get("overall_context_relevancy")
        lines.extend(
            [
                "## Retrieval (L2)",
                "",
                "| SUT | evidence_recall | context_relevancy |",
                "|-----|-----------------|-------------------|",
                f"| EdgeQuake | {_fmt(eq_er)} | {_fmt(eq_cr)} |",
                f"| LightRAG | {_fmt(lr_er)} | {_fmt(lr_cr)} |",
                "",
            ]
        )
    lines.extend(
        [
            "## By question_type (EQ Acc)",
            "",
        ]
    )
    for qtype, block in (m["eq"].get("by_type") or {}).items():
        acc = block.get("answer_correctness", block.get("rouge_score", 0.0))
        lines.append(f"- **{qtype}:** {acc:.4f}")
    lines.extend(
        [
            "",
            "## By question_type (LR Acc)",
            "",
        ]
    )
    for qtype, block in (m["lr"].get("by_type") or {}).items():
        acc = block.get("answer_correctness", block.get("rouge_score", 0.0))
        lines.append(f"- **{qtype}:** {acc:.4f}")
    ops = scorecard["ops"]
    lines.extend(
        [
            "",
            "## Ops",
            "",
            f"- EQ empty-answer rate: {ops['eq_empty_answer_rate']:.3f}",
            f"- LR empty-answer rate: {ops['lr_empty_answer_rate']:.3f}",
            f"- EQ empty-context rate: {ops.get('eq_empty_context_rate', 0.0):.3f}",
            f"- LR empty-context rate: {ops.get('lr_empty_context_rate', 0.0):.3f}",
            f"- EQ query p50/p95 ms: {ops['eq_query_p50_ms']} / {ops['eq_query_p95_ms']}",
            f"- LR query p50/p95 ms: {ops['lr_query_p50_ms']} / {ops['lr_query_p95_ms']}",
            f"- ingest wall s: {ops['ingest_wall_s']:.1f}",
        ]
    )
    if ops.get("eq_over_lr_p50_ratio") is not None:
        slo = ops.get("latency_slo_1_5x")
        lines.append(
            f"- EQ/LR p50 ratio: {ops['eq_over_lr_p50_ratio']} "
            f"(SLO ≤1.5×: {'PASS' if slo else 'FAIL/WAIVE'})"
        )
    stage_bits = []
    for label, key in (
        ("keyword", "eq_keyword_p50_ms"),
        ("embed", "eq_embed_p50_ms"),
        ("retrieve", "eq_retrieve_p50_ms"),
        ("rerank", "eq_rerank_p50_ms"),
        ("generate", "eq_generate_p50_ms"),
    ):
        if ops.get(key) is not None:
            stage_bits.append(f"{label}={ops[key]}")
    if stage_bits:
        lines.append(f"- EQ stage p50 ms: {', '.join(stage_bits)}")
    lines.extend(
        [
            "",
            "## Pins",
            "",
            "```json",
            json.dumps(scorecard["pins"], indent=2),
            "```",
            "",
            "## Progression",
            "",
            "- Ladder ledger: `specs/001-benchmark/e2e/artifacts/PROGRESS.md`",
            "- This run archives under `specs/001-benchmark/e2e/artifacts/history/`",
            "",
        ]
    )
    path.write_text("\n".join(lines), encoding="utf-8")
