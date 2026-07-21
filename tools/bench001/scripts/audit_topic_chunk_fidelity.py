#!/usr/bin/env python3
"""041 — Topic-admit chunk id/content fidelity (First Principles).

After 038–040 protect ladder REJECT: packing cannot help when post-CE Mix has
zero topic ids. This audit answers only:

  entity (exact-name from q) → source_chunk_ids → DB body → question bigrams?

Laws (necessary conditions — not Acc gates):
  RESOLVE   each source_chunk_id fetches a non-empty body (chunks or vectors)
  CONTENT   ≥1 resolved body contains a question content bigram
  IN_MIX    ≥1 topic chunk body bigram appears in admitted Mix C
  (CE_GAP)  CONTENT ∧ ¬IN_MIX ⇒ survivors lost between graph link and C

Forbidden: densify-all, domain needle bags, protect-knob Acc fishing.

Usage:
  BENCH001_EQ_WORKSPACE_ID=2a7bcb2f-... \\
  python3 tools/bench001/scripts/audit_topic_chunk_fidelity.py \\
    --question-id Medical-0002d2de \\
    --predictions-eq specs/.../predictions_eq.json
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
    _eq_database_url,
    _eq_entity_chunk_map,
    _load_eq_graph,
    _load_lr_entity_chunks,
    _norm_entity,
)
from audit_summarize_chunk_links import (  # noqa: E402
    _mix_parts,
    _question_content_bigrams,
)
from bench001.paths import ARTIFACTS_DIR  # noqa: E402
from bench001.warm_workspace import resolve_warm_workspace_id  # noqa: E402


def _content_bigrams(question: str) -> list[str]:
    return list(_question_content_bigrams(question))


def _bare_norm(name: str) -> str:
    bare = name.rsplit("::", 1)[-1]
    return _norm_entity(bare)


def _singularize_norm(n: str) -> str | None:
    """Match edgequake-query topic_entity_admit::singularize_norm."""
    if len(n) > 4 and n.endswith("S") and not n.endswith("SS"):
        return n[:-1]
    return None


def _entity_norms_from_bigrams(bigrams: list[str]) -> list[str]:
    """038 candidate norms: bigram → TOKEN + singularized form."""
    out: list[str] = []
    seen: set[str] = set()
    for b in bigrams:
        n = _norm_entity(b.replace(" ", "_"))
        for cand in ( _singularize_norm(n), n ):
            if not cand or len(cand) < 4 or cand in seen:
                continue
            seen.add(cand)
            out.append(cand)
    return out


def _fetch_chunk_bodies(chunk_ids: list[str], workspace_id: str) -> dict[str, str]:
    """Resolve id → content via eq_eq_default_kv (content_ref), then public.chunks."""
    del workspace_id  # reserved for future workspace-scoped KV
    import psycopg2
    import psycopg2.extras

    out: dict[str, str] = {}
    if not chunk_ids:
        return out
    url = _eq_database_url().split("?")[0]
    conn = psycopg2.connect(url)
    try:
        with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
            # Acc fair ingest: chunk body lives in default KV under the chunk id key.
            cur.execute(
                """
                SELECT key AS id, value->>'content' AS content
                FROM eq_eq_default_kv
                WHERE key = ANY(%s)
                """,
                (chunk_ids,),
            )
            for r in cur.fetchall():
                if r.get("content"):
                    out[r["id"]] = r["content"]

            missing = [c for c in chunk_ids if c not in out]
            if missing:
                uuids = [
                    c for c in missing if re.fullmatch(r"[0-9a-fA-F-]{36}", c)
                ]
                if uuids:
                    cur.execute(
                        """
                        SELECT id::text AS id, content
                        FROM chunks
                        WHERE id = ANY(%s::uuid[])
                        """,
                        (uuids,),
                    )
                    for r in cur.fetchall():
                        if r.get("content"):
                            out[r["id"]] = r["content"]
    finally:
        conn.close()
    return out


def _bigram_hits(text: str, bigrams: list[str]) -> list[str]:
    low = text.lower()
    return [b for b in bigrams if b.lower() in low]


def run_audit(
    *,
    workspace_id: str,
    question_id: str,
    predictions_eq: Path,
    predictions_lr: Path | None,
    graph: str,
    lr_stage: str,
    out_dir: Path,
) -> dict:
    preds = json.loads(predictions_eq.read_text(encoding="utf-8"))
    hit = next((p for p in preds if question_id in str(p.get("id", ""))), None)
    if not hit:
        raise SystemExit(f"question {question_id} not in {predictions_eq}")

    question = hit.get("question") or ""
    bigrams = _content_bigrams(question)
    # Exact-name entity candidates from content bigrams (038: + singularize)
    entity_norms = _entity_norms_from_bigrams(bigrams)

    eq_nodes, _ = _load_eq_graph(workspace_id, graph)
    eq_ent = _eq_entity_chunk_map(eq_nodes)
    eq_by_norm: dict[str, tuple[str, list[str]]] = {}
    for name, ids in eq_ent.items():
        n = _bare_norm(name)
        prev = eq_by_norm.get(n)
        if prev is None or len(ids) > len(prev[1]):
            eq_by_norm[n] = (name, ids)

    lr_ent = {}
    if lr_stage:
        try:
            lr_ent = _load_lr_entity_chunks(lr_stage)
        except SystemExit:
            lr_ent = {}
    lr_by_norm = {_norm_entity(k): (k, v) for k, v in lr_ent.items()}

    parts, mix_text = _mix_parts(hit.get("context"))
    mix_hits = _bigram_hits(mix_text, bigrams)

    entities_report = []
    all_topic_ids: list[str] = []
    for en in entity_norms:
        eq_row = eq_by_norm.get(en)
        lr_row = lr_by_norm.get(en)
        eq_ids = list(eq_row[1]) if eq_row else []
        lr_ids = list(lr_row[1]) if lr_row else []
        all_topic_ids.extend(eq_ids)
        bodies = _fetch_chunk_bodies(eq_ids, workspace_id)
        resolved = sum(1 for i in eq_ids if i in bodies and bodies[i].strip())
        content_hit_ids = []
        samples = []
        for cid in eq_ids:
            body = bodies.get(cid, "")
            hits = _bigram_hits(body, bigrams) if body else []
            if hits:
                content_hit_ids.append(cid)
            samples.append(
                {
                    "id": cid,
                    "resolved": bool(body.strip()),
                    "chars": len(body),
                    "bigram_hits": hits,
                    "head": body[:160].replace("\n", " ") if body else "",
                }
            )
        entities_report.append(
            {
                "norm": en,
                "eq_name": eq_row[0] if eq_row else None,
                "lr_name": lr_row[0] if lr_row else None,
                "eq_n_chunks": len(eq_ids),
                "lr_n_chunks": len(lr_ids),
                "resolved": resolved,
                "unresolved": len(eq_ids) - resolved,
                "content_hit_chunks": len(content_hit_ids),
                "content_hit_ids": content_hit_ids,
                "samples": samples,
            }
        )

    # Dedup topic ids preserve order
    seen = set()
    topic_ids = []
    for i in all_topic_ids:
        if i not in seen:
            seen.add(i)
            topic_ids.append(i)

    bodies_all = _fetch_chunk_bodies(topic_ids, workspace_id)
    content_ok = any(
        _bigram_hits(bodies_all.get(i, ""), bigrams) for i in topic_ids
    )
    resolve_ok = bool(topic_ids) and all(
        bodies_all.get(i, "").strip() for i in topic_ids
    )
    resolve_any = any(bodies_all.get(i, "").strip() for i in topic_ids)
    in_mix = bool(mix_hits)

    if not topic_ids:
        law = "NO_ENTITY"
        note = "No exact-name entity from question bigrams in EQ graph."
    elif not resolve_any:
        law = "RESOLVE"
        note = "source_chunk_ids do not fetch bodies (id namespace / missing rows)."
    elif not content_ok:
        law = "CONTENT"
        note = (
            "Linked chunk bodies resolve but contain 0 question content bigrams "
            "(provenance pollution / wrong neighborhood on the entity)."
        )
    elif not in_mix:
        law = "CE_GAP"
        note = (
            "Linked bodies contain question bigrams, but Mix C does not "
            "(SELECT after link — CE/fuse/trunc survivors)."
        )
    else:
        law = "GEN_OR_EVAL"
        note = "Mix C already has question bigrams; miss is generation/eval."

    report = {
        "utc": datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ"),
        "workspace_id": workspace_id,
        "question_id": question_id,
        "question": question,
        "query_intent": hit.get("query_intent"),
        "content_bigrams": bigrams,
        "entity_norms": entity_norms,
        "mix": {
            "parts": len(parts),
            "chars": len(mix_text),
            "bigram_hits": mix_hits,
        },
        "laws": {
            "RESOLVE_all": resolve_ok,
            "RESOLVE_any": resolve_any,
            "CONTENT": content_ok,
            "IN_MIX": in_mix,
        },
        "verdict_law": law,
        "verdict_note": note,
        "entities": entities_report,
        "predictions_eq": str(predictions_eq),
        "predictions_lr": str(predictions_lr) if predictions_lr else None,
    }

    # Optional LR Mix probe
    if predictions_lr and predictions_lr.is_file():
        lr_preds = json.loads(predictions_lr.read_text(encoding="utf-8"))
        lr_hit = next(
            (p for p in lr_preds if question_id in str(p.get("id", ""))), None
        )
        if lr_hit:
            _, lr_mix = _mix_parts(lr_hit.get("context"))
            report["lr_mix"] = {
                "chars": len(lr_mix),
                "bigram_hits": _bigram_hits(lr_mix, bigrams),
            }

    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "audit_report.json").write_text(
        json.dumps(report, indent=2) + "\n", encoding="utf-8"
    )
    md = [
        "# 041 Topic chunk fidelity audit",
        "",
        f"**UTC:** {report['utc']}  ",
        f"**Q:** `{question_id}` — {question}  ",
        f"**WS:** `{workspace_id}`  ",
        "",
        "## Verdict",
        "",
        f"**Law:** `{law}`  ",
        f"{note}",
        "",
        "## Observables",
        "",
        f"- Content bigrams: `{', '.join(bigrams)}`",
        f"- Entity norms: `{', '.join(entity_norms)}`",
        f"- Mix chars/parts: {report['mix']['chars']} / {report['mix']['parts']}",
        f"- Mix bigram hits: `{', '.join(mix_hits) or '∅'}`",
        f"- RESOLVE_any={resolve_any} CONTENT={content_ok} IN_MIX={in_mix}",
        "",
        "## Entities",
        "",
    ]
    for e in entities_report:
        md.append(
            f"### `{e['norm']}` (EQ `{e['eq_name']}` · LR `{e['lr_name']}`)"
        )
        md.append("")
        md.append(
            f"- chunks EQ/LR: {e['eq_n_chunks']} / {e['lr_n_chunks']} · "
            f"resolved {e['resolved']}/{e['eq_n_chunks']} · "
            f"content-hit {e['content_hit_chunks']}"
        )
        for s in e["samples"][:6]:
            md.append(
                f"  - `{s['id'][:48]}` resolved={s['resolved']} "
                f"hits={s['bigram_hits'] or '∅'} · {s['head'][:100]!r}"
            )
        md.append("")
    if report.get("lr_mix"):
        md += [
            "## LR Mix (same Q)",
            "",
            f"- chars: {report['lr_mix']['chars']}",
            f"- bigram hits: `{', '.join(report['lr_mix']['bigram_hits']) or '∅'}`",
            "",
        ]
    md += [
        "## Next (one confound)",
        "",
        "- If `RESOLVE`: fix chunk id namespace between AGE `source_chunk_ids` and storage.",
        "- If `CONTENT`: fix entity↔chunk provenance (wrong links) — not CE protect.",
        "- If `CE_GAP`: one SELECT fix so CONTENT survivors enter Mix C.",
        "- Forbidden: densify-all, stacking TOPIC_* protect without fidelity law.",
        "",
    ]
    (out_dir / "SUMMARY.md").write_text("\n".join(md), encoding="utf-8")
    return report


def main() -> int:
    ap = argparse.ArgumentParser(description="041 topic chunk fidelity audit")
    ap.add_argument(
        "--workspace-id",
        default=os.environ.get("BENCH001_EQ_WORKSPACE_ID")
        or resolve_warm_workspace_id()
        or "",
    )
    ap.add_argument("--question-id", default="Medical-0002d2de")
    ap.add_argument(
        "--predictions-eq",
        default="",
        help="EQ predictions JSON (default: latest a1fp Acc peer)",
    )
    ap.add_argument("--predictions-lr", default="")
    ap.add_argument(
        "--graph",
        default=os.environ.get("EDGEQUAKE_AGE_GRAPH") or "eq_eq_default_graph",
    )
    ap.add_argument(
        "--lr-stage",
        default=os.environ.get("BENCH001_LR_STAGE") or "smoke",
    )
    ap.add_argument("--out", default="")
    args = ap.parse_args()
    if not args.workspace_id:
        raise SystemExit("workspace id required")

    pred_eq = Path(args.predictions_eq) if args.predictions_eq else Path(
        "specs/001-benchmark/e2e/artifacts/history/smoke-20260720T095809Z/"
        "predictions_eq.json"
    )
    pred_lr = (
        Path(args.predictions_lr)
        if args.predictions_lr
        else pred_eq.parent / "predictions_lr.json"
    )
    utc = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out = (
        Path(args.out)
        if args.out
        else ARTIFACTS_DIR / "ingest-audit" / f"topic-fidelity-{utc}"
    )
    report = run_audit(
        workspace_id=args.workspace_id,
        question_id=args.question_id,
        predictions_eq=pred_eq,
        predictions_lr=pred_lr if pred_lr.is_file() else None,
        graph=args.graph,
        lr_stage=args.lr_stage,
        out_dir=out,
    )
    print(f"verdict_law={report['verdict_law']}")
    print(json.dumps(report["laws"], indent=2))
    print(f"→ {out / 'SUMMARY.md'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
