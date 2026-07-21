#!/usr/bin/env python3
"""037 Horizon B — Summarize chunk-link audit (First Principles).

Causal chain (001 / LightRAG operate.py):
  q → entity|relation hit → source_id / source_chunk_ids → Mix C → Â → Summarize ER

This audit measures only observables on that chain. It does **not** apply
promote thresholds (+N, ≥p%) or topic-needle bags that invent a verdict.

Layers
  L0  Global hygiene: zero-chunk entity counts (EQ AGE vs LR kv_store_entity_chunks)
  L1  Admitted Mix: parts/chars EQ vs LR (same question id)
  L2  Gold coverage of admitted Mix: share of Mix parts that hit ≥1 gold evidence phrase
  L3  Exact-name entity↔chunk pairs: for each LR entity whose normalized name equals
      an EQ entity, report n_chunks (no substring fishing)

Verdict law (necessary conditions — not scores):
  LINK:   topic exact-pair entities have chunks on LR and empty on EQ
  SELECT: topic exact-pairs have chunks on both, but EQ Mix gold-hit ≪ LR Mix gold-hit
  GEN:    EQ Mix gold-hit ≈ LR but Summarize ER still lags (out of scope here)

Read-only. Disk-safe. No re-ingest.

Usage:
  BENCH001_EQ_WORKSPACE_ID=2a7bcb2f-... \\
  python3 tools/bench001/scripts/audit_summarize_chunk_links.py \\
    --predictions-eq .../predictions_eq.json \\
    --predictions-lr .../predictions_lr.json
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
_SCRIPTS = Path(__file__).resolve().parent
sys.path.insert(0, str(_SCRIPTS))
sys.path.insert(0, str(ROOT / "tools" / "bench001"))

from audit_eq_lr_ingest import (  # noqa: E402
    _eq_entity_chunk_map,
    _load_eq_graph,
    _load_lr_entity_chunks,
    _norm_entity,
)
from bench001.paths import ARTIFACTS_DIR  # noqa: E402
from bench001.warm_workspace import resolve_warm_workspace_id  # noqa: E402


def _flat_evidence(ev) -> list[str]:
    if not ev:
        return []
    if isinstance(ev, str):
        return [ev]
    out: list[str] = []
    for e in ev:
        if isinstance(e, list):
            out.extend(str(x) for x in e)
        else:
            out.append(str(e))
    return out


def _mix_parts(ctx) -> list[str]:
    if isinstance(ctx, list) and ctx:
        ctx_s = str(ctx[0])
    else:
        ctx_s = str(ctx or "")
    return [p for p in ctx_s.split("\n-----\n") if p.strip()], ctx_s


_Q_STOP = {
    "how",
    "are",
    "what",
    "which",
    "the",
    "and",
    "for",
    "with",
    "in",
    "of",
    "to",
    "a",
    "an",
    "is",
    "their",
    "this",
    "that",
    "from",
    "into",
    "used",
    "most",
    "main",
    "considered",
    "determining",
    "factors",
    "does",
    "do",
    "can",
    "when",
    "where",
    "who",
    "why",
}


def _question_content_bigrams(question: str) -> list[str]:
    """Contiguous content bigrams taken verbatim from the question text.

    GraphRAG-Bench `evidence` strings are often paraphrases — they are **not**
    reliable corpus probes. The question's own content bigrams are.
    """
    words = re.findall(r"[A-Za-z][A-Za-z0-9\-]+", question or "")
    content = [w for w in words if w.casefold() not in _Q_STOP]
    grams: list[str] = []
    for i in range(len(content) - 1):
        grams.append(f"{content[i]} {content[i+1]}".casefold())
    # de-dupe preserve order
    seen: set[str] = set()
    out: list[str] = []
    for g in grams:
        if g not in seen:
            seen.add(g)
            out.append(g)
    return out


def _mix_topic_coverage(parts: list[str], bigrams: list[str]) -> dict:
    """Count which question content bigrams appear as substrings in Mix."""
    blob = "\n".join(parts).casefold()
    hits = [g for g in bigrams if g in blob]
    missed = [g for g in bigrams if g not in blob]
    return {
        "n_parts": len(parts),
        "n_bigrams": len(bigrams),
        "n_bigrams_hit": len(hits),
        "hit_bigrams": hits,
        "missed_bigrams": missed,
    }


def _bare_eq_map(eq_ent: dict[str, list[str]]) -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    for name, chunks in eq_ent.items():
        bare = name.split("::", 1)[-1] if "::" in name else name
        out[bare] = list(dict.fromkeys(out.get(bare, []) + list(chunks)))
    return out


def _index_by_norm(ent_map: dict[str, list[str]]) -> dict[str, tuple[str, list[str]]]:
    """normalized_name → (display_name, chunks). First wins; merge chunks on collision."""
    idx: dict[str, tuple[str, list[str]]] = {}
    for name, chunks in ent_map.items():
        nn = _norm_entity(name)
        if not nn:
            continue
        if nn in idx:
            prev_name, prev = idx[nn]
            merged = list(dict.fromkeys(prev + list(chunks)))
            idx[nn] = (prev_name, merged)
        else:
            idx[nn] = (name, list(chunks))
    return idx


def _question_topic_norms(question: str, evidence: list[str]) -> list[str]:
    """Multi-word topic names derived from the question (exact-norm candidates).

    Example: "How are bone cancers staged..." → BONE_CANCERS, BONE_CANCER (singularize).
    No domain needle list — only tokens from the question text.
    """
    q = question or ""
    # noun-ish bigrams/trigrams from question
    words = re.findall(r"[A-Za-z][A-Za-z0-9\-]+", q)
    stop = {
        "how",
        "are",
        "what",
        "which",
        "the",
        "and",
        "for",
        "with",
        "in",
        "of",
        "to",
        "a",
        "an",
        "is",
        "their",
        "this",
        "that",
        "from",
        "into",
        "used",
        "most",
        "main",
        "considered",
        "determining",
        "factors",
    }
    content = [w for w in words if w.casefold() not in stop]
    norms: list[str] = []
    for n in (2, 3, 1):
        for i in range(0, max(0, len(content) - n + 1)):
            gram = "_".join(content[i : i + n])
            norms.append(_norm_entity(gram))
            # naive singular: cancers → cancer
            if gram.casefold().endswith("s") and len(gram) > 4:
                norms.append(_norm_entity(gram[:-1]))
    # evidence head terms (first 4 words) as secondary exact candidates
    for e in evidence[:4]:
        ew = re.findall(r"[A-Za-z][A-Za-z0-9\-]+", str(e))[:4]
        if len(ew) >= 2:
            norms.append(_norm_entity("_".join(ew[:2])))
            norms.append(_norm_entity("_".join(ew[:3])))
    # unique preserve order
    seen: set[str] = set()
    out: list[str] = []
    for n in norms:
        if n and n not in seen and len(n) >= 4:
            seen.add(n)
            out.append(n)
    return out


def _exact_pair_rows(
    topic_norms: list[str],
    eq_idx: dict[str, tuple[str, list[str]]],
    lr_idx: dict[str, tuple[str, list[str]]],
) -> list[dict]:
    rows: list[dict] = []
    for nn in topic_norms:
        eq = eq_idx.get(nn)
        lr = lr_idx.get(nn)
        if not eq and not lr:
            continue
        rows.append(
            {
                "norm": nn,
                "eq_name": eq[0] if eq else None,
                "lr_name": lr[0] if lr else None,
                "eq_n_chunks": len(eq[1]) if eq else None,
                "lr_n_chunks": len(lr[1]) if lr else None,
                "link_gap": (
                    "EQ_EMPTY_LR_LINKED"
                    if (eq is not None and len(eq[1]) == 0 and lr is not None and len(lr[1]) > 0)
                    else "LR_EMPTY_EQ_LINKED"
                    if (lr is not None and len(lr[1]) == 0 and eq is not None and len(eq[1]) > 0)
                    else "BOTH_EMPTY"
                    if (
                        (eq is not None and len(eq[1]) == 0)
                        and (lr is not None and len(lr[1]) == 0)
                    )
                    else "BOTH_LINKED"
                    if (eq is not None and lr is not None and len(eq[1]) > 0 and len(lr[1]) > 0)
                    else "EQ_ONLY"
                    if eq is not None and lr is None
                    else "LR_ONLY"
                ),
            }
        )
    return rows


def _classify_binding(row: dict) -> dict:
    """Necessary-condition classification — no numeric promote thresholds."""
    pairs = row.get("exact_pairs") or []
    both = [p for p in pairs if p["link_gap"] == "BOTH_LINKED"]
    eq_empty = [p for p in pairs if p["link_gap"] == "EQ_EMPTY_LR_LINKED"]
    lr_only = [p for p in pairs if p["link_gap"] == "LR_ONLY"]

    eq_t = row["eq_topic"]
    lr_t = row["lr_topic"]

    facts = {
        "eq_mix_parts": row["eq_mix_parts"],
        "lr_mix_parts": row["lr_mix_parts"],
        "eq_question_bigrams_hit": eq_t["n_bigrams_hit"],
        "lr_question_bigrams_hit": lr_t["n_bigrams_hit"],
        "eq_missed_bigrams": eq_t["missed_bigrams"],
        "lr_missed_bigrams": lr_t["missed_bigrams"],
        "n_exact_pairs_both_linked": len(both),
        "n_exact_pairs_eq_empty_lr_linked": len(eq_empty),
        "n_exact_pairs_lr_only": len(lr_only),
        "eq_empty_names": [p["norm"] for p in eq_empty],
        "both_linked_sample": [
            {
                "norm": p["norm"],
                "eq_n": p["eq_n_chunks"],
                "lr_n": p["lr_n_chunks"],
            }
            for p in both[:12]
        ],
    }

    # Law application (boolean comparisons of counts only — no % gates):
    if eq_empty:
        law = "LINK"
        why = (
            "Question-derived exact-name entity has source chunks on LR and empty "
            "source_chunk_ids on EQ — that entity cannot contribute chunks to Mix."
        )
    elif lr_only and not both:
        law = "LINK_OR_NAME"
        why = (
            "LR has question-topic entities EQ lacks under exact normalized name — "
            "extract/naming coverage gap (ingest)."
        )
    elif both and eq_t["n_bigrams_hit"] < lr_t["n_bigrams_hit"]:
        law = "SELECT"
        why = (
            "Topic entities are linked on both sides (not LINK starvation), yet EQ "
            "Mix contains fewer of the question's own content bigrams than LR — "
            "Mix admission selected the wrong neighborhood."
        )
    elif both and row["eq_mix_parts"] < row["lr_mix_parts"] and eq_t["n_bigrams_hit"] == 0:
        law = "SELECT"
        why = (
            "Topic entities linked both sides; EQ Mix hits none of the question "
            "content bigrams and admits fewer Mix parts than LR."
        )
    elif both and eq_t["n_bigrams_hit"] >= lr_t["n_bigrams_hit"]:
        law = "GEN_OR_EVAL"
        why = (
            "EQ Mix is not below LR on question-bigram containment; if Summarize ER "
            "still lags, look at generation/eval — not entity→chunk link density."
        )
    else:
        law = "INCONCLUSIVE"
        why = (
            "Not enough exact-name pairs from the question to separate LINK from "
            "SELECT. Report observables; do not invent density thresholds."
        )

    return {"law": law, "why": why, "facts": facts}


def _load_rows(path: Path) -> list[dict]:
    rows = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(rows, dict):
        rows = rows.get("predictions") or rows.get("results") or list(rows.values())
    return list(rows)


def run(
    *,
    predictions_eq: Path,
    predictions_lr: Path | None,
    workspace_id: str,
    lr_stage: str,
    graph: str,
    out_dir: Path,
) -> dict:
    eq_rows = _load_rows(predictions_eq)
    lr_by_id: dict[str, dict] = {}
    if predictions_lr and predictions_lr.is_file():
        for r in _load_rows(predictions_lr):
            lr_by_id[str(r.get("id") or "")] = r

    summarize = [r for r in eq_rows if r.get("question_type") == "Contextual Summarize"]

    lr_ent = _load_lr_entity_chunks(lr_stage)
    eq_nodes, eq_edges = _load_eq_graph(workspace_id, graph)
    eq_by_norm = _bare_eq_map(_eq_entity_chunk_map(eq_nodes))
    eq_idx = _index_by_norm(eq_by_norm)
    lr_idx = _index_by_norm(lr_ent)

    eq_zero = sum(1 for v in eq_by_norm.values() if not v)
    lr_zero = sum(1 for v in lr_ent.values() if not v)
    eq_lens = [len(v) for v in eq_by_norm.values()]
    lr_lens = [len(v) for v in lr_ent.values()]

    per_q: list[dict] = []
    for r in summarize:
        qid = str(r.get("id") or "")
        question = r.get("question") or ""
        evidence = _flat_evidence(r.get("evidence"))
        eq_parts, eq_ctx = _mix_parts(r.get("context"))
        lr_r = lr_by_id.get(qid) or {}
        lr_parts, lr_ctx = _mix_parts(lr_r.get("context"))
        bigrams = _question_content_bigrams(question)

        topic_norms = _question_topic_norms(question, evidence)
        pairs = _exact_pair_rows(topic_norms, eq_idx, lr_idx)
        row = {
            "id": qid,
            "question": question,
            "query_intent": r.get("query_intent"),
            "n_evidence": len(evidence),
            "eq_mix_parts": len(eq_parts),
            "eq_mix_chars": len(eq_ctx),
            "lr_mix_parts": len(lr_parts),
            "lr_mix_chars": len(lr_ctx),
            "question_bigrams": bigrams,
            "eq_topic": _mix_topic_coverage(eq_parts, bigrams),
            "lr_topic": _mix_topic_coverage(lr_parts, bigrams),
            "topic_norms": topic_norms[:24],
            "exact_pairs": pairs,
            "eq_part_heads": [" ".join(p.split())[:160] for p in eq_parts[:8]],
            "lr_part_heads": [" ".join(p.split())[:160] for p in lr_parts[:8]],
        }
        row["verdict"] = _classify_binding(row)
        per_q.append(row)

    # Binding = fewest EQ question-bigram hits, then fewest EQ mix parts
    per_q.sort(
        key=lambda x: (
            x["eq_topic"]["n_bigrams_hit"],
            x["eq_mix_parts"],
            -x["lr_topic"]["n_bigrams_hit"],
        )
    )

    report = {
        "utc": datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ"),
        "workspace_id": workspace_id,
        "lr_stage": lr_stage,
        "eq_graph": graph,
        "predictions_eq": str(predictions_eq),
        "predictions_lr": str(predictions_lr) if predictions_lr else None,
        "first_principles": {
            "chain": "q → entity hit → source_chunk_ids → Mix C → Summarize ER",
            "laws": {
                "LINK": "EQ empty source_ids on exact-name topic entity that LR links",
                "SELECT": (
                    "both linked; EQ Mix hits fewer question content bigrams than LR"
                ),
                "GEN_OR_EVAL": "EQ Mix question-bigram hits not below LR",
            },
            "probes": {
                "topic_in_C": "verbatim question content bigrams (not evidence paraphrases)",
                "entity_links": "exact normalized name pairs only",
            },
            "forbidden": "No +N / ≥p% / domain-needle / token-overlap promote heuristics",
        },
        "global": {
            "eq_nodes": len(eq_nodes),
            "eq_edges": eq_edges,
            "eq_entities_bare": len(eq_by_norm),
            "lr_entities": len(lr_ent),
            "eq_zero_chunk": eq_zero,
            "lr_zero_chunk": lr_zero,
            "eq_mean_chunks": round(sum(eq_lens) / len(eq_lens), 3) if eq_lens else 0.0,
            "lr_mean_chunks": round(sum(lr_lens) / len(lr_lens), 3) if lr_lens else 0.0,
            "note": (
                "Zero-chunk counts are hygiene observables. They are not a promote "
                "gate; they only matter if LINK law fires on question-topic entities."
            ),
        },
        "summarize_questions": per_q,
        "binding": per_q[0] if per_q else None,
    }

    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "audit_report.json").write_text(
        json.dumps(report, indent=2) + "\n", encoding="utf-8"
    )

    md = [
        "# Summarize chunk-link audit (037 Horizon B) — First Principles",
        "",
        f"**UTC:** {report['utc']}  ",
        f"**EQ workspace:** `{workspace_id}`  ",
        f"**LR stage:** `{lr_stage}`  ",
        f"**EQ preds:** `{predictions_eq}`  ",
        f"**LR preds:** `{predictions_lr}`  ",
        "",
        "## Laws (necessary conditions)",
        "",
        "| Law | Meaning |",
        "|-----|---------|",
        "| LINK | Exact-name topic entity: LR has source chunks, EQ has none |",
        "| SELECT | Topic entities linked both sides; EQ Mix hits fewer question content bigrams |",
        "| GEN_OR_EVAL | EQ Mix question-bigram hits not below LR |",
        "",
        "Probes: question content bigrams (verbatim) + exact-name entity pairs.  ",
        "Forbidden: `+N` / `%` / domain-needle / token-overlap promote heuristics.  ",
        "Note: GraphRAG-Bench evidence strings are paraphrases — not used as corpus probes.",
        "",
        "## Global hygiene (observables only)",
        "",
        f"- EQ entities: **{len(eq_by_norm)}** · mean chunks **{report['global']['eq_mean_chunks']}** · zero-chunk **{eq_zero}**",
        f"- LR entities: **{len(lr_ent)}** · mean chunks **{report['global']['lr_mean_chunks']}** · zero-chunk **{lr_zero}**",
        f"- EQ AGE nodes/edges: **{len(eq_nodes)}** / **{eq_edges}**",
        "",
        "## Per Summarize question",
        "",
        "| ID | Law | EQ parts | LR parts | EQ q-bigrams | LR q-bigrams | LINK empty |",
        "|----|-----|---------:|---------:|-------------:|-------------:|-----------:|",
    ]
    for q in per_q:
        v = q["verdict"]
        md.append(
            f"| `{q['id'][:18]}` | {v['law']} | {q['eq_mix_parts']} | {q['lr_mix_parts']} | "
            f"{q['eq_topic']['n_bigrams_hit']}/{q['eq_topic']['n_bigrams']} | "
            f"{q['lr_topic']['n_bigrams_hit']}/{q['lr_topic']['n_bigrams']} | "
            f"{v['facts']['n_exact_pairs_eq_empty_lr_linked']} |"
        )

    if report["binding"]:
        b = report["binding"]
        v = b["verdict"]
        md += [
            "",
            "## Binding question (fewest EQ question-bigram hits)",
            "",
            f"**{b['id']}** — {b['question']}",
            "",
            f"**Law:** `{v['law']}`  ",
            f"**Why:** {v['why']}",
            "",
            f"**Question bigrams:** `{b['question_bigrams']}`",
            "",
            "### Facts",
            "",
            "```json",
            json.dumps(v["facts"], indent=2),
            "```",
            "",
            "### Exact-name pairs (question-derived)",
            "",
            "| norm | EQ name | EQ n | LR name | LR n | gap |",
            "|------|---------|-----:|---------|-----:|-----|",
        ]
        for p in b["exact_pairs"]:
            md.append(
                f"| `{p['norm']}` | {p['eq_name'] or '—'} | {p['eq_n_chunks'] if p['eq_n_chunks'] is not None else '—'} | "
                f"{p['lr_name'] or '—'} | {p['lr_n_chunks'] if p['lr_n_chunks'] is not None else '—'} | {p['link_gap']} |"
            )
        md += [
            "",
            "### EQ Mix part heads",
            "",
        ]
        for i, h in enumerate(b["eq_part_heads"], 1):
            md.append(f"{i}. {h}")
        md += ["", "### LR Mix part heads", ""]
        for i, h in enumerate(b["lr_part_heads"], 1):
            md.append(f"{i}. {h}")
        md += [
            "",
            "## Next confound (from law)",
            "",
        ]
        if v["law"] == "LINK":
            md.append(
                "- Preserve `source_chunk_ids` on extract/merge for EQ_EMPTY_LR_LINKED "
                "names (reingest when disk allows). One confound."
            )
        elif v["law"] == "SELECT":
            md.append(
                "- Do **not** densify-all links. Next confound: Mix entity ranking / "
                "related-chunk pick so linked question-topic entities admit into C "
                "instead of generic neighbors (off-topic Mix)."
            )
        elif v["law"] == "LINK_OR_NAME":
            md.append(
                "- Extract/naming parity for LR_ONLY topic entities (Horizon B ingest)."
            )
        else:
            md.append("- See law why; do not invent a density threshold.")

    (out_dir / "SUMMARY.md").write_text("\n".join(md) + "\n", encoding="utf-8")
    # rewrite JSON with full global (already complete)
    (out_dir / "audit_report.json").write_text(
        json.dumps(report, indent=2) + "\n", encoding="utf-8"
    )
    return report


def main() -> int:
    ap = argparse.ArgumentParser(
        description="037 Summarize chunk-link audit (First Principles)"
    )
    ap.add_argument("--predictions-eq", "--predictions", type=Path, required=True)
    ap.add_argument(
        "--predictions-lr",
        type=Path,
        default=None,
        help="Paired LR predictions (default: sibling predictions_lr.json)",
    )
    ap.add_argument(
        "--workspace-id",
        default=os.environ.get("BENCH001_EQ_WORKSPACE_ID")
        or resolve_warm_workspace_id()
        or "",
    )
    ap.add_argument("--lr-stage", default=os.environ.get("BENCH001_LR_STAGE") or "smoke")
    ap.add_argument(
        "--graph",
        default=os.environ.get("EDGEQUAKE_AGE_GRAPH") or "eq_eq_default_graph",
    )
    ap.add_argument("--out", default="")
    args = ap.parse_args()
    if not args.workspace_id:
        raise SystemExit("workspace id required")
    if not args.predictions_eq.is_file():
        raise SystemExit(f"predictions missing: {args.predictions_eq}")
    lr_path = args.predictions_lr
    if lr_path is None:
        cand = args.predictions_eq.with_name("predictions_lr.json")
        lr_path = cand if cand.is_file() else None

    utc = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out = (
        Path(args.out)
        if args.out
        else ARTIFACTS_DIR / "ingest-audit" / f"summarize-{utc}"
    )
    report = run(
        predictions_eq=args.predictions_eq,
        predictions_lr=lr_path,
        workspace_id=args.workspace_id,
        lr_stage=args.lr_stage,
        graph=args.graph,
        out_dir=out,
    )
    b = report.get("binding") or {}
    v = b.get("verdict") or {}
    print(
        json.dumps(
            {
                "binding_id": b.get("id"),
                "law": v.get("law"),
                "eq_parts": b.get("eq_mix_parts"),
                "lr_parts": b.get("lr_mix_parts"),
                "eq_q_bigrams": (b.get("eq_topic") or {}).get("n_bigrams_hit"),
                "lr_q_bigrams": (b.get("lr_topic") or {}).get("n_bigrams_hit"),
                "facts": (v.get("facts") or {}),
            },
            indent=2,
        )
    )
    print(f"→ {out / 'SUMMARY.md'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
