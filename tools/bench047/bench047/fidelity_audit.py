"""Offline W1 fidelity audit against ingested markdown (EQ-047-W1a)."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Optional

from .client import EdgeQuakeClient
from .fidelity import aggregate_fidelity, fidelity_for_sample
from .paths import stage_artifact_dir
from .score import load_jsonl
from .subset import parse_list_field


def run_fidelity_audit(
    stage: str = "smoke",
    *,
    base_url: Optional[str] = None,
    max_samples: Optional[int] = None,
) -> dict[str, Any]:
    art = stage_artifact_dir(stage)
    meta = json.loads((art / "meta.json").read_text())
    ingest = {r["doc_id"]: r for r in load_jsonl(art / "ingest.jsonl")}
    preds = load_jsonl(art / "predictions.jsonl")

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
        fid["page_hit@5"] = (s.get("retrieval") or {}).get("page_hit@5")
        rows.append(fid)

    report = {
        "stage": stage,
        "workspace_id": client.workspace_id,
        "aggregate": aggregate_fidelity(rows),
        "n_errors": len(errors),
        "errors": errors[:20],
        "samples": rows,
    }

    # Causal split: representation miss vs retrieval miss
    rep_miss = [r for r in rows if not r.get("answer_in_evidence_pages")]
    ret_miss_with_rep = [
        r
        for r in rows
        if r.get("answer_in_evidence_pages") and r.get("page_hit@5") is False
    ]
    report["causal"] = {
        "representation_miss_n": len(rep_miss),
        "retrieval_miss_given_rep_ok_n": len(ret_miss_with_rep),
        "note": (
            "If answer not in evidence-page markdown → W1 (ingest). "
            "If answer in markdown but page_hit@5 false → W2 (retrieve)."
        ),
    }

    out = art / "fidelity.json"
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False))
    summary = art / "FIDELITY.md"
    agg = report["aggregate"]
    lines = [
        f"# SPEC-047 {stage} — W1 representation fidelity",
        "",
        f"- n_answerable_audited: {agg.get('n', 0)}",
        f"- answer_in_evidence_rate: **{agg.get('answer_in_evidence_rate', 0):.4f}**",
        f"- answer_in_document_rate: {agg.get('answer_in_document_rate', 0):.4f}",
        f"- representation_miss_n: {report['causal']['representation_miss_n']}",
        f"- retrieval_miss_given_rep_ok_n: {report['causal']['retrieval_miss_given_rep_ok_n']}",
        "",
        "## By evidence source",
    ]
    for k, v in (agg.get("by_evidence_source") or {}).items():
        lines.append(f"- {k}: rate={v['rate']:.4f} (n={v['n']})")
    lines += ["", report["causal"]["note"], ""]
    summary.write_text("\n".join(lines))
    print(summary.read_text())
    print(f"Wrote {out}")
    return report
