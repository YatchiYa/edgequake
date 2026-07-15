"""Offline W1 fidelity audit against ingested markdown (EQ-047-W1a).

Protocol: gate runs MUST audit all answerable questions (no max_samples).
`--max-samples` is debug-only and marks the report non-gateable.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Optional

from .client import EdgeQuakeClient
from .fidelity import aggregate_fidelity, fidelity_for_sample
from .paths import stage_artifact_dir
from .protocol import DEFAULT_BENCH_PROFILE, PROTOCOL_VERSION, gate_notes
from .score import load_jsonl


def run_fidelity_audit(
    stage: str = "smoke",
    *,
    base_url: Optional[str] = None,
    max_samples: Optional[int] = None,
    allow_partial: bool = False,
) -> dict[str, Any]:
    art = stage_artifact_dir(stage)
    meta = json.loads((art / "meta.json").read_text())
    ingest = {r["doc_id"]: r for r in load_jsonl(art / "ingest.jsonl")}
    preds = load_jsonl(art / "predictions.jsonl")

    n_answerable = sum(1 for s in preds if str(s.get("answer") or "") != "Not answerable")
    gateable = True
    warnings: list[str] = []
    if max_samples is not None:
        if not allow_partial:
            print(
                "ERROR: --max-samples is debug-only. Gate runs require full answerable "
                "audit. Pass --allow-partial to force a truncated debug audit "
                "(report will be marked gateable=false).",
                file=sys.stderr,
            )
            raise SystemExit(2)
        warnings.append(
            f"partial_audit max_samples={max_samples} of {n_answerable} answerable — NOT gateable"
        )
        gateable = False
        print(f"WARN: {warnings[-1]}", file=sys.stderr)

    client = EdgeQuakeClient(base_url=base_url or meta.get("api_base"))
    client.workspace_id = meta.get("workspace_id")

    md_cache: dict[str, str] = {}
    rows: list[dict[str, Any]] = []
    errors: list[dict[str, str]] = []

    for s in preds:
        if str(s.get("answer") or "") == "Not answerable":
            continue
        if max_samples is not None and len(rows) >= max_samples:
            break
        doc_id = s["doc_id"]
        ingest_row = ingest.get(doc_id) or {}
        eq_doc = ingest_row.get("document_id") or s.get("edgequake_document_id")
        if not eq_doc:
            errors.append({"doc_id": doc_id, "error": "missing_document_id"})
            continue
        if eq_doc not in md_cache:
            try:
                md_cache[eq_doc] = client.get_markdown(eq_doc)
            except Exception as e:
                errors.append({"doc_id": doc_id, "error": str(e)})
                continue
        fid = fidelity_for_sample(
            answer=s.get("answer") or "",
            evidence_pages=s.get("evidence_pages"),
            markdown=md_cache[eq_doc],
            evidence_sources=s.get("evidence_sources"),
        )
        fid["doc_id"] = doc_id
        fid["question"] = s.get("question")
        fid["pred"] = s.get("pred")
        fid["pred_raw"] = s.get("pred_raw")
        fid["page_hit@5"] = (s.get("retrieval") or {}).get("page_hit@5")
        rows.append(fid)

    if gateable and n_answerable and len(rows) < n_answerable:
        # Errors reduced coverage — still not a clean gate compare
        warnings.append(
            f"audited {len(rows)}/{n_answerable} answerable (errors={len(errors)}) — "
            "cross-run compare only if peer has same n"
        )
        if errors:
            gateable = False

    agg = aggregate_fidelity(rows)
    report = {
        "stage": stage,
        "workspace_id": client.workspace_id,
        "protocol_version": PROTOCOL_VERSION,
        "gateable": gateable,
        "n_answerable_total": n_answerable,
        "n_answerable_audited": len(rows),
        "warnings": warnings,
        "protocol": gate_notes(),
        "aggregate": agg,
        "n_errors": len(errors),
        "errors": errors[:20],
        "samples": rows,
    }

    # Causal split: representation miss vs retrieval miss (raw + long)
    rep_miss = [r for r in rows if not r.get("answer_in_evidence_pages")]
    rep_miss_long = [
        r
        for r in rows
        if r.get("long_eligible") and not r.get("answer_in_evidence_pages_long")
    ]
    ret_miss_with_rep = [
        r
        for r in rows
        if r.get("answer_in_evidence_pages") and r.get("page_hit@5") is False
    ]
    report["causal"] = {
        "representation_miss_n": len(rep_miss),
        "representation_miss_long_n": len(rep_miss_long),
        "retrieval_miss_given_rep_ok_n": len(ret_miss_with_rep),
        "short_needle_fp_suspect_n": sum(
            1 for r in rows if r.get("short_needle_fp_suspect")
        ),
        "note": (
            "If answer not in evidence-page markdown → W1 (ingest). "
            "If answer in markdown but page_hit@5 false → W2 (retrieve). "
            "Gate Wave 1 on long-needle rates, not raw short-needle rates."
        ),
    }

    out = art / "fidelity.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False))
    summary = art / "FIDELITY.md"
    gates = agg.get("gates") or {}
    chart_g = gates.get("chart_a_in_e_long") or {}
    table_g = gates.get("table_a_in_e_long") or {}
    long_rate = agg.get("answer_in_evidence_rate_long")
    lines = [
        f"# SPEC-047 {stage} — W1 representation fidelity",
        "",
        f"- protocol: `{PROTOCOL_VERSION}`",
        f"- gateable: `{gateable}`"
        + (f" ({'; '.join(warnings)})" if warnings else ""),
        f"- n_answerable_audited: {agg.get('n', 0)} / total_answerable={n_answerable}",
        f"- answer_in_evidence_rate (raw): {agg.get('answer_in_evidence_rate', 0):.4f}",
        f"- answer_in_evidence_rate_long (GATE): "
        f"**{long_rate if long_rate is None else f'{long_rate:.4f}'}** "
        f"(n_long={agg.get('n_long_eligible', 0)}, min_needle≥{agg.get('min_needle_len_gate')})",
        f"- answer_in_document_rate: {agg.get('answer_in_document_rate', 0):.4f}",
        f"- short_needle_fp_suspect: {report['causal']['short_needle_fp_suspect_n']} "
        f"/ short_needle={agg.get('n_short_needle', 0)}",
        f"- representation_miss_n (raw): {report['causal']['representation_miss_n']}",
        f"- representation_miss_long_n: {report['causal']['representation_miss_long_n']}",
        f"- retrieval_miss_given_rep_ok_n: {report['causal']['retrieval_miss_given_rep_ok_n']}",
        "",
        "## Wave 1 gates (long-needle)",
        f"- Chart a_in_e_long: "
        f"{'PASS' if chart_g.get('pass') else 'FAIL'} "
        f"rate={chart_g.get('rate')} n={chart_g.get('n')} "
        f"threshold≥{chart_g.get('threshold')}",
        f"- Table a_in_e_long: "
        f"{'PASS' if table_g.get('pass') else 'FAIL'} "
        f"rate={table_g.get('rate')} n={table_g.get('n')} "
        f"threshold≥{table_g.get('threshold')}",
        "",
        "## By evidence source (raw / multi-label)",
    ]
    for k, v in (agg.get("by_evidence_source") or {}).items():
        lines.append(f"- {k}: rate={v['rate']:.4f} (n={v['n']})")
    lines += ["", "## By evidence source (long-needle / GATE)"]
    for k, v in (agg.get("by_evidence_source_long") or {}).items():
        lines.append(f"- {k}: rate={v['rate']:.4f} (n={v['n']})")
    lines += ["", "## By evidence source exclusive (len==1, raw)"]
    for k, v in (agg.get("by_evidence_source_exclusive") or {}).items():
        lines.append(f"- {k}: rate={v['rate']:.4f} (n={v['n']})")
    lines += [
        "",
        report["causal"]["note"],
        "",
        "Do not compare raw a_in_e across runs with different n_answerable_audited.",
        "",
    ]
    summary.write_text("\n".join(lines))
    print(summary.read_text())
    print(f"Wrote {out}")
    return report
