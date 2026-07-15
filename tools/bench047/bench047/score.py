"""Scoring + scorecard assembly using vendored MMLongBench eval_score."""

from __future__ import annotations

import json
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from . import mmlongbench_eval_score as ev
from .paths import EVAL_SCORE_SHA
from .profiles import BANNER, BenchProfile
from .protocol import (
    PROTOCOL_VERSION,
    attribution_slices,
    exclusive_source_accuracy,
    gate_notes,
)
from .subset import parse_list_field


def score_sample(gt: str, pred: str, answer_format: str) -> float:
    return float(ev.eval_score(gt, pred, answer_format))


def build_scorecard(
    *,
    stage: str,
    profile: BenchProfile,
    samples: list[dict[str, Any]],
    pins_extra: dict[str, Any],
    ops: dict[str, Any],
    valid: bool = True,
    invalid_reason: str | None = None,
) -> dict[str, Any]:
    # prepare for upstream aggregators
    prepared = []
    for s in samples:
        row = dict(s)
        if "score" not in row:
            continue
        # ensure evidence fields are string form for show_results compatibility
        if isinstance(row.get("evidence_pages"), list):
            row["evidence_pages"] = str(row["evidence_pages"])
        if isinstance(row.get("evidence_sources"), list):
            row["evidence_sources"] = str(row["evidence_sources"])
        prepared.append(row)

    acc, f1 = ev.eval_acc_and_f1(prepared)

    def _with_lists(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
        out = []
        for r in rows:
            rr = dict(r)
            rr["evidence_pages"] = parse_list_field(str(rr.get("evidence_pages", "[]")))
            rr["evidence_sources"] = parse_list_field(str(rr.get("evidence_sources", "[]")))
            out.append(rr)
        return out

    listed = _with_lists(prepared)
    single = [s for s in listed if len(s["evidence_pages"]) == 1]
    cross = [
        s
        for s in listed
        if len(s["evidence_pages"]) != 1 and s.get("answer") != "Not answerable"
    ]
    unans = [s for s in listed if s.get("answer") == "Not answerable"]

    by_src: dict[str, list] = defaultdict(list)
    by_type: dict[str, list] = defaultdict(list)
    for s in listed:
        for src in s["evidence_sources"]:
            by_src[str(src)].append(s)
        by_type[str(s.get("doc_type", "?"))].append(s)

    return {
        "spec": "047",
        "stage": stage,
        "valid": valid,
        "invalid_reason": invalid_reason,
        "task_name": "MMLongBench-Doc/RAG-adaptation",
        "banner": BANNER,
        "pins": {
            "edgequake_version": pins_extra.get("edgequake_version", "unknown"),
            "edgequake_git_sha": pins_extra.get("edgequake_git_sha", "unknown"),
            "dataset_id": "yubo2333/MMLongBench-Doc",
            "dataset_revision": pins_extra.get("dataset_revision", "parquet-train"),
            "fixture_id": pins_extra.get("fixture_id", "smoke_doc_ids_v1"),
            "llm_provider": profile.llm_provider,
            "llm_model": profile.llm_model,
            "vision_provider": profile.vision_provider,
            "vision_model": profile.vision_model,
            "embedding_provider": profile.embedding_provider,
            "embedding_model": profile.embedding_model,
            "embedding_dim": profile.embedding_dim,
            "query_mode": profile.query_mode,
            "extractor_model": (
                "gpt-4o"
                if profile.extractor.startswith("gpt")
                else profile.llm_model
            ),
            "eval_score_sha": EVAL_SCORE_SHA,
            "system_prompt_sha": pins_extra.get("system_prompt_sha", "server-default"),
            "profile_id": profile.profile_id,
            "pdf_parser_backend": profile.pdf_parser_backend,
            "process_options": profile.process_options,
        },
        "metrics": {
            "n_docs": ops.get("n_docs", 0),
            "n_questions": ops.get("n_questions", len(prepared)),
            "n_scored": len(prepared),
            "n_skipped_ingest_failed": ops.get("n_skipped_ingest_failed", 0),
            "accuracy": float(acc),
            "f1": float(f1),
        },
        "slices": {
            "single_page_accuracy": float(ev.eval_acc_and_f1(single)[0]) if single else 0.0,
            "cross_page_accuracy": float(ev.eval_acc_and_f1(cross)[0]) if cross else 0.0,
            "unanswerable_accuracy": float(ev.eval_acc_and_f1(unans)[0]) if unans else 0.0,
            "by_evidence_source": {
                k: {"accuracy": float(ev.eval_acc_and_f1(v)[0]), "n": len(v)}
                for k, v in by_src.items()
            },
            # Honest Chart/Table-only Acc (len(evidence_sources)==1). Official
            # by_evidence_source multi-counts Chart∩Table questions into both.
            "by_evidence_source_exclusive": exclusive_source_accuracy(listed),
            "by_doc_type": {
                k: {"accuracy": float(ev.eval_acc_and_f1(v)[0]), "n": len(v)}
                for k, v in by_type.items()
            },
            "attribution": attribution_slices(listed),
        },
        "protocol": gate_notes(),
        "ops": {
            "ingest_coverage": ops.get("ingest_coverage", 0.0),
            "cost_usd_total": ops.get("cost_usd_total"),
            "p50_query_latency_ms": ops.get("p50_query_latency_ms"),
            "p95_query_latency_ms": ops.get("p95_query_latency_ms"),
            "answer_empty_rate": ops.get("answer_empty_rate"),
            "extractor_fail_rate": ops.get("extractor_fail_rate"),
            "page_hit_rate": ops.get("page_hit_rate"),
            "document_scope": ops.get("document_scope", False),
            "query_workers": ops.get("query_workers"),
            "ingest_workers": ops.get("ingest_workers"),
            "retrieval": ops.get("retrieval") or {},
            "false_refusal": ops.get("false_refusal") or {},
            "arm_gates": ops.get("arm_gates") or {},
        },
        "created_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    }


def write_summary(scorecard: dict[str, Any], path: Path) -> None:
    m = scorecard["metrics"]
    s = scorecard["slices"]
    ops = scorecard.get("ops") or {}
    ret = ops.get("retrieval") or {}
    fr = ops.get("false_refusal") or {}
    arms = ops.get("arm_gates") or {}
    lines = [
        f"# SPEC-047 {scorecard['stage']} — {scorecard['created_at_utc']}",
        "",
        f"> {scorecard['banner']}",
        "",
        "## Verdict",
        f"- valid: `{scorecard['valid']}`"
        + (f" ({scorecard['invalid_reason']})" if scorecard.get("invalid_reason") else ""),
        f"- Overall Acc: **{m['accuracy']:.4f}** (n_scored={m['n_scored']})",
        f"- Overall F1: **{m['f1']:.4f}**",
        f"- Docs: {m['n_docs']} | Questions: {m['n_questions']} | Ingest skip: {m['n_skipped_ingest_failed']}",
        f"- Ingest coverage: {ops.get('ingest_coverage', 0):.2f}",
        f"- Profile: `{scorecard['pins']['profile_id']}` mode=`{scorecard['pins']['query_mode']}`"
        + (
            f" process_options=`{scorecard['pins']['process_options']}`"
            if scorecard["pins"].get("process_options")
            else ""
        )
        + (
            f" query_workers={ops['query_workers']}"
            if ops.get("query_workers") is not None
            else ""
        )
        + (
            f" ingest_workers={ops['ingest_workers']}"
            if ops.get("ingest_workers") is not None
            else ""
        ),
        "",
        "## How to read this score",
        f"- **protocol:** `{PROTOCOL_VERSION}` — Acc/F1 = official soft-score; W1 gates use long-needle a_in_e.",
        "- **valid=true** means ops gates passed (ingest + non-empty answers). It is not “beats GPT-4o.”",
        "- **Acc** = mean official short-answer score; **F1** balances answerable vs predicted-answerable.",
        "- **LVLM GPT-4o F1≈44.9%** is a difficulty reference only (page-screenshot task ≠ RAG).",
        "- Prefer slice gaps (chart / cross-page / unanswerable) over a single headline number.",
        "- **by_evidence_source** multi-counts (official). **exclusive** = len(sources)==1 (honest Chart-only).",
        "- **Acc ↑ ≠ W1 win** — require Chart exclusive Acc ↑ and Chart a_in_e_long ≥ 0.50.",
        "- **page_hit@k** (W0): gold `evidence_pages` ∩ retrieved chunk `page_start` — retrieval law, not Acc.",
        "- **false_refusal** (020 A2): answerable gold ∧ pred≈Not answerable; slice by page_hit@5.",
        "",
        "## Retrieval diagnostics (W0)",
    ]
    if ret.get("n_with_retrieval_diag"):
        lines += [
            f"- n_answerable_with_diag: {ret['n_with_retrieval_diag']}",
            f"- document_scope: `{ops.get('document_scope', False)}`",
            f"- context_empty_rate: {ret.get('context_empty_rate', 0):.4f}",
            f"- page_hit@1: {ret.get('page_hit@1')}",
            f"- page_hit@3: {ret.get('page_hit@3')}",
            f"- page_hit@5: {ret.get('page_hit@5')}",
            f"- page_hit@10: {ret.get('page_hit@10')}",
            f"- page_recall@5: {ret.get('page_recall@5')}",
            f"- mean_n_chunk_sources: {ret.get('mean_n_chunk_sources')}",
            f"- mean_arm_local_chunks: {ret.get('mean_arm_local_chunks')}",
            f"- mean_arm_global_chunks: {ret.get('mean_arm_global_chunks')}",
            f"- mean_arm_naive_chunks: {ret.get('mean_arm_naive_chunks')}",
        ]
    else:
        lines.append("- _(no retrieval block — re-run query stage with current bench047)_")

    lines += ["", "## Refusal diagnostics (020 A2)"]
    if fr.get("n_answerable"):
        fr_rate = fr.get("false_refusal_rate")
        fr_hit = fr.get("false_refusal_given_page_hit@5")
        lines += [
            f"- n_answerable: {fr['n_answerable']}",
            f"- false_refusal_rate: {fr_rate if fr_rate is None else f'{fr_rate:.4f}'}"
            f" (n={fr.get('n_false_refusal', 0)})",
            f"- false_refusal_given_page_hit@5: "
            f"{fr_hit if fr_hit is None else f'{fr_hit:.4f}'}"
            f" (n={fr.get('n_false_refusal_page_hit@5', 0)}"
            f" / {fr.get('n_answerable_page_hit@5', 0)})",
        ]
    else:
        lines.append("- _(no refusal block — re-run query stage)_")

    lines += ["", "## Arm-gate diagnostics (020 B1/B2)"]
    if arms.get("n_with_arm_diag"):
        lines += [
            f"- n_with_arm_diag: {arms['n_with_arm_diag']}",
            f"- arms_gated_rate: {arms.get('arms_gated_rate')}",
            f"- planned_graph_rate: {arms.get('planned_graph_rate')} "
            f"(planned_naive_only={arms.get('planned_naive_only_rate')})",
            f"- arm_graph_present_rate: {arms.get('arm_graph_present_rate')} "
            f"(productive chunks; empty local ≠ gate)",
            f"- naive_only_rate: {arms.get('naive_only_rate')} "
            f"(productive; prefer planned_* for B2 honesty)",
            f"- arm_local_present_rate: {arms.get('arm_local_present_rate')}",
            f"- arm_global_present_rate: {arms.get('arm_global_present_rate')}",
        ]
    else:
        lines.append("- _(no arm-gate block — engine stats omitted)_")

    lines += [
        "",
        "## Slices",
        f"- Single-page Acc: {s['single_page_accuracy']:.4f}",
        f"- Cross-page Acc: {s['cross_page_accuracy']:.4f}",
        f"- Unanswerable Acc: {s['unanswerable_accuracy']:.4f}",
        "",
        "### By evidence source (multi-label / official)",
    ]
    for k, v in sorted(s.get("by_evidence_source", {}).items()):
        lines.append(f"- {k}: Acc={v['accuracy']:.4f} (n={v['n']})")
    lines += ["", "### By evidence source exclusive (len==1)"]
    for k, v in sorted(s.get("by_evidence_source_exclusive", {}).items()):
        lines.append(f"- {k}: Acc={v['accuracy']:.4f} (n={v['n']})")
    attr = s.get("attribution") or {}
    if attr:
        lines += ["", "### Acc attribution (single-run mass)"]
        for key in ("list_gold", "unanswerable", "other_answerable"):
            block = attr.get(key) or {}
            acc_v = block.get("accuracy")
            acc_s = f"{acc_v:.4f}" if acc_v is not None else "—"
            lines.append(
                f"- {key}: Acc={acc_s} n={block.get('n', 0)} "
                f"score_sum={block.get('score_sum', 0):.3f}"
            )
    lines += ["", "### By document type"]
    for k, v in sorted(s.get("by_doc_type", {}).items()):
        lines.append(f"- {k}: Acc={v['accuracy']:.4f} (n={v['n']})")
    lines += [
        "",
        "## Citation",
        "Ma et al., MMLongBench-Doc, arXiv:2407.01523 / NeurIPS 2024 D&B.",
        "https://github.com/mayubo2333/MMLongBench-Doc",
        "",
    ]
    path.write_text("\n".join(lines))


def append_jsonl(path: Path, row: dict[str, Any]) -> None:
    with path.open("a") as f:
        f.write(json.dumps(row, ensure_ascii=False) + "\n")


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    if not path.exists():
        return []
    rows = []
    for line in path.read_text().splitlines():
        if line.strip():
            rows.append(json.loads(line))
    return rows
