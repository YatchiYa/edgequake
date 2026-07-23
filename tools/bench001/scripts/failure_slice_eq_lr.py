#!/usr/bin/env python3
"""080/081 — EQ vs LR failure forensics on a dual-SUT archive (no new packing).

Reads predictions_*.json + eval_*.json from a medical-mid/full history archive and
writes ranked failure modes: AccΔ by type, empty answers, gold/evidence∩context SNR,
and 081 F1 membership-vs-generation split on Fact LR-wins.

Example:
  PYTHONPATH=tools/bench001 python3 tools/bench001/scripts/failure_slice_eq_lr.py \\
    --archive specs/001-benchmark/e2e/artifacts/history/medical-full-20260722T171906Z \\
    --out specs/001-benchmark/e2e/artifacts/forensics/f1-e2-full
"""

from __future__ import annotations

import argparse
import json
import re
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


_TOKEN_RE = re.compile(r"[a-z0-9]{3,}", re.I)


def _tok(text: str) -> set[str]:
    return set(_TOKEN_RE.findall((text or "").lower()))


def _jaccard(a: set[str], b: set[str]) -> float:
    if not a and not b:
        return 1.0
    if not a or not b:
        return 0.0
    return len(a & b) / len(a | b)


def _load_json(path: Path) -> Any:
    with path.open() as f:
        return json.load(f)


def _pred_index(preds: list[dict]) -> dict[str, dict]:
    return {str(p.get("id")): p for p in preds if p.get("id")}


def _detailed_rows(eval_doc: dict) -> list[dict]:
    raw = eval_doc.get("raw") or {}
    rows: list[dict] = []
    if not isinstance(raw, dict):
        return rows
    for qtype, block in raw.items():
        if not isinstance(block, dict):
            continue
        detailed = block.get("detailed") or []
        if not isinstance(detailed, list):
            continue
        for item in detailed:
            if not isinstance(item, dict):
                continue
            m = item.get("metrics") or {}
            rows.append(
                {
                    "id": str(item.get("id") or ""),
                    "question_type": qtype,
                    "question": item.get("question") or "",
                    "acc": float(m.get("answer_correctness") or 0.0),
                    "f1": float(m.get("factuality_f1") or 0.0),
                    "cos": float(m.get("embed_cosine") or 0.0),
                }
            )
    return rows


def _context_text(pred: dict) -> str:
    ctx = pred.get("context")
    if isinstance(ctx, list):
        parts = []
        for c in ctx:
            if isinstance(c, str):
                parts.append(c)
            elif isinstance(c, dict):
                parts.append(str(c.get("content") or c.get("text") or ""))
        return "\n".join(parts)
    if isinstance(ctx, str):
        return ctx
    return ""


def _evidence_text(pred: dict) -> str:
    ev = pred.get("evidence")
    if isinstance(ev, list):
        return "\n".join(str(x) for x in ev)
    if isinstance(ev, str):
        return ev
    return str(pred.get("ground_truth") or pred.get("gold_answer") or "")


def analyze(archive: Path) -> dict[str, Any]:
    eq_pred = _load_json(archive / "predictions_eq.json")
    lr_pred = _load_json(archive / "predictions_lr.json")
    eq_eval = _load_json(archive / "eval_eq.json")
    lr_eval = _load_json(archive / "eval_lr.json")
    scorecard = _load_json(archive / "scorecard.json")

    eq_by_id = _pred_index(eq_pred)
    lr_by_id = _pred_index(lr_pred)
    eq_rows = {r["id"]: r for r in _detailed_rows(eq_eval) if r["id"]}
    lr_rows = {r["id"]: r for r in _detailed_rows(lr_eval) if r["id"]}

    empty_eq = [
        pid
        for pid, p in eq_by_id.items()
        if not str(p.get("generated_answer") or "").strip()
    ]
    empty_lr = [
        pid
        for pid, p in lr_by_id.items()
        if not str(p.get("generated_answer") or "").strip()
    ]

    by_type_acc_gap: dict[str, list[float]] = defaultdict(list)
    fact_lr_wins: list[dict[str, Any]] = []
    snr_eq: list[float] = []
    snr_lr: list[float] = []
    snr_eq_lt_lr = 0

    for qid, er in eq_rows.items():
        lr = lr_rows.get(qid)
        if not lr:
            continue
        gap = er["acc"] - lr["acc"]
        by_type_acc_gap[er["question_type"]].append(gap)
        pe = eq_by_id.get(qid) or {}
        pl = lr_by_id.get(qid) or {}
        gold = _tok(_evidence_text(pe) or _evidence_text(pl) or er.get("question", ""))
        je = _jaccard(gold, _tok(_context_text(pe)))
        jl = _jaccard(gold, _tok(_context_text(pl)))
        snr_eq.append(je)
        snr_lr.append(jl)
        if je + 1e-9 < jl:
            snr_eq_lt_lr += 1
        if er["question_type"] == "Fact Retrieval" and gap <= -0.05:
            # 081 F1: membership miss = gold tokens largely absent from EQ Acc context;
            # generation miss = gold present in context but Acc still lags LR.
            gold_n = len(gold)
            overlap = len(gold & _tok(_context_text(pe))) if gold else 0
            coverage = (overlap / gold_n) if gold_n else 0.0
            if gold_n == 0 or coverage < 0.15:
                miss_class = "membership"
            else:
                miss_class = "generation"
            fact_lr_wins.append(
                {
                    "id": qid,
                    "eq_acc": round(er["acc"], 4),
                    "lr_acc": round(lr["acc"], 4),
                    "delta": round(gap, 4),
                    "eq_snr": round(je, 4),
                    "lr_snr": round(jl, 4),
                    "gold_token_coverage": round(coverage, 4),
                    "miss_class": miss_class,
                    "question": (er.get("question") or "")[:160],
                }
            )

    fact_lr_wins.sort(key=lambda x: x["delta"])
    membership_n = sum(1 for w in fact_lr_wins if w.get("miss_class") == "membership")
    generation_n = sum(1 for w in fact_lr_wins if w.get("miss_class") == "generation")
    type_summary = {
        t: {
            "n": len(gaps),
            "mean_acc_delta_eq_minus_lr": round(sum(gaps) / len(gaps), 4),
            "eq_ahead": sum(1 for g in gaps if g > 0.02),
            "lr_ahead": sum(1 for g in gaps if g < -0.02),
            "near_tie": sum(1 for g in gaps if abs(g) <= 0.02),
        }
        for t, gaps in sorted(by_type_acc_gap.items())
    }

    sc_eq = (scorecard.get("metrics") or {}).get("eq") or {}
    sc_lr = (scorecard.get("metrics") or {}).get("lr") or {}
    sc_ret_eq = (sc_eq.get("retrieval") or {})
    sc_ret_lr = (sc_lr.get("retrieval") or {})
    fact_er_eq = ((sc_ret_eq.get("by_type") or {}).get("Fact Retrieval") or {}).get(
        "evidence_recall"
    )
    fact_er_lr = ((sc_ret_lr.get("by_type") or {}).get("Fact Retrieval") or {}).get(
        "evidence_recall"
    )

    modes = []
    if empty_eq:
        modes.append(
            {
                "mode": "R5_empty_answers",
                "count": len(empty_eq),
                "note": "EQ empty generated_answer with (likely) non-empty context",
                "ids_sample": empty_eq[:20],
            }
        )
    if type_summary.get("Fact Retrieval", {}).get("lr_ahead", 0) > 0:
        modes.append(
            {
                "mode": "Fact_Acc_LR_ahead",
                "count": type_summary["Fact Retrieval"]["lr_ahead"],
                "mean_delta": type_summary["Fact Retrieval"]["mean_acc_delta_eq_minus_lr"],
                "note": "Primary Acc type gap — often R6 list split / evidence miss",
            }
        )
    if snr_eq and snr_lr and (sum(snr_eq) / len(snr_eq)) + 0.02 < (sum(snr_lr) / len(snr_lr)):
        modes.append(
            {
                "mode": "SNR_context_vs_gold_lower",
                "eq_mean_jaccard": round(sum(snr_eq) / len(snr_eq), 4),
                "lr_mean_jaccard": round(sum(snr_lr) / len(snr_lr), 4),
                "eq_lt_lr_count": snr_eq_lt_lr,
                "note": "Proxy for noisier / less complete context → prefer D1 unify / D2 type weights",
            }
        )
    if fact_er_eq is not None and fact_er_lr is not None and fact_er_eq + 0.03 < fact_er_lr:
        modes.append(
            {
                "mode": "Fact_ER_gap",
                "eq_fact_er": round(float(fact_er_eq), 4),
                "lr_fact_er": round(float(fact_er_lr), 4),
                "note": "Supports D1 R6 Acc/L2 unify before packing retries",
            }
        )

    # 081: packing D1–D3 STOP — recommend B10 naming vs generation groundedness.
    recommended_next = "F3_B10_naming"
    if modes and modes[0]["mode"] == "R5_empty_answers" and len(empty_eq) >= 5:
        recommended_next = "D5_empty_then_F3"
    elif generation_n > membership_n and generation_n >= 3:
        recommended_next = "F4_generation_after_F3"
    elif membership_n > generation_n and membership_n >= 3:
        recommended_next = "F3_B10_naming"
    elif any(m["mode"] == "Fact_ER_gap" for m in modes):
        recommended_next = "F3_B10_or_B6_fact_er_label"

    fact_miss_split = {
        "fact_lr_wins_n": len(fact_lr_wins),
        "membership_n": membership_n,
        "generation_n": generation_n,
        "membership_share": round(membership_n / len(fact_lr_wins), 4)
        if fact_lr_wins
        else None,
        "generation_share": round(generation_n / len(fact_lr_wins), 4)
        if fact_lr_wins
        else None,
        "note": (
            "081 F1: membership = gold token coverage <0.15 in EQ Acc context; "
            "generation = coverage ≥0.15 but Acc still lags LR"
        ),
    }

    return {
        "archive": str(archive),
        "n_eq_preds": len(eq_by_id),
        "n_lr_preds": len(lr_by_id),
        "n_paired_eval": len(set(eq_rows) & set(lr_rows)),
        "scorecard": {
            "eq_acc": sc_eq.get("overall_acc"),
            "lr_acc": sc_lr.get("overall_acc"),
            "eq_ctx": sc_ret_eq.get("overall_context_relevancy"),
            "lr_ctx": sc_ret_lr.get("overall_context_relevancy"),
            "eq_er": sc_ret_eq.get("overall_evidence_recall"),
            "lr_er": sc_ret_lr.get("overall_evidence_recall"),
            "eq_fact_er": fact_er_eq,
            "lr_fact_er": fact_er_lr,
            "eq_empty_answer_rate": (scorecard.get("ops") or {}).get("eq_empty_answer_rate"),
            "lr_empty_answer_rate": (scorecard.get("ops") or {}).get("lr_empty_answer_rate"),
        },
        "by_type_acc": type_summary,
        "empty_answers": {
            "eq_count": len(empty_eq),
            "lr_count": len(empty_lr),
            "eq_ids_sample": empty_eq[:30],
        },
        "snr_proxy": {
            "eq_mean_jaccard_gold_vs_context": round(sum(snr_eq) / len(snr_eq), 4)
            if snr_eq
            else None,
            "lr_mean_jaccard_gold_vs_context": round(sum(snr_lr) / len(snr_lr), 4)
            if snr_lr
            else None,
            "eq_lt_lr_count": snr_eq_lt_lr,
        },
        "fact_lr_wins_top": fact_lr_wins[:40],
        "fact_miss_split": fact_miss_split,
        "ranked_failure_modes": modes,
        "recommended_next_phase": recommended_next,
        "honesty": "Labeled forensics only — not Acc Beat; Acc publish/latest stays P0 mid",
    }


def write_report(out: Path, report: dict[str, Any]) -> None:
    out.mkdir(parents=True, exist_ok=True)
    (out / "failure_slice.json").write_text(json.dumps(report, indent=2) + "\n")
    lines = [
        "# 081 / 080 failure slice",
        "",
        f"**Archive:** `{report['archive']}`",
        f"**Paired eval n:** {report['n_paired_eval']}",
        f"**Recommended next:** `{report['recommended_next_phase']}`",
        "",
        "## Scorecard snapshot",
        "",
        "```json",
        json.dumps(report["scorecard"], indent=2),
        "```",
        "",
        "## 081 F1 membership vs generation (Fact LR-wins)",
        "",
        "```json",
        json.dumps(report.get("fact_miss_split") or {}, indent=2),
        "```",
        "",
        "## Ranked failure modes",
        "",
    ]
    for i, m in enumerate(report["ranked_failure_modes"], 1):
        lines.append(f"{i}. **{m['mode']}** — {m.get('note', '')}")
        lines.append(f"   `{json.dumps({k: v for k, v in m.items() if k != 'note'})}`")
        lines.append("")
    lines += [
        "## Acc Δ by type (EQ − LR)",
        "",
        "```json",
        json.dumps(report["by_type_acc"], indent=2),
        "```",
        "",
        "## Empty answers",
        "",
        f"- EQ empty: {report['empty_answers']['eq_count']}",
        f"- LR empty: {report['empty_answers']['lr_count']}",
        "",
        "## SNR proxy (gold/evidence tokens ∩ context)",
        "",
        "```json",
        json.dumps(report["snr_proxy"], indent=2),
        "```",
        "",
        f"Top Fact LR-wins written in `failure_slice.json` ({len(report['fact_lr_wins_top'])} rows).",
        "",
    ]
    (out / "FAILURE_SLICE.md").write_text("\n".join(lines))


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--archive",
        type=Path,
        required=True,
        help="History archive dir with predictions_*/eval_*/scorecard.json",
    )
    ap.add_argument("--out", type=Path, required=True, help="Output forensics directory")
    args = ap.parse_args()
    archive = args.archive.resolve()
    if not (archive / "scorecard.json").is_file():
        raise SystemExit(f"missing scorecard.json under {archive}")
    report = analyze(archive)
    write_report(args.out.resolve(), report)
    print(f"wrote {args.out}/FAILURE_SLICE.md")
    print(f"recommended_next_phase={report['recommended_next_phase']}")
    print(f"modes={[m['mode'] for m in report['ranked_failure_modes']]}")


if __name__ == "__main__":
    main()
