#!/usr/bin/env python3
"""028 B1 — Paired EQ↔LR extract / source_id audit on warm Acc corpus.

Compares LightRAG entity↔chunk links (kv_store_entity_chunks) with EdgeQuake
AGE nodes for ``BENCH001_EQ_WORKSPACE_ID`` (or warm_workspace.json).

Does **not** mutate either store. Writes JSON + markdown under
``specs/001-benchmark/e2e/artifacts/ingest-audit/<utc>/``.

Usage:
  python3 tools/bench001/scripts/audit_eq_lr_ingest.py
  BENCH001_LR_STAGE=smoke_c100000 DATABASE_URL=... python3 ...
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
sys.path.insert(0, str(ROOT / "tools" / "bench001"))

from bench001.paths import ARTIFACTS_DIR, cache_root  # noqa: E402
from bench001.warm_workspace import resolve_warm_workspace_id  # noqa: E402


def _norm_entity(name: str) -> str:
    s = (name or "").strip().upper()
    s = re.sub(r"[^A-Z0-9]+", "_", s)
    return s.strip("_")


def _load_lr_entity_chunks(stage: str) -> dict[str, list[str]]:
    path = cache_root() / "lightrag" / stage / "kv_store_entity_chunks.json"
    if not path.is_file():
        raise SystemExit(f"LR entity_chunks missing: {path}")
    data = json.loads(path.read_text(encoding="utf-8"))
    out: dict[str, list[str]] = {}
    for key, val in data.items():
        if isinstance(val, dict):
            ids = val.get("chunk_ids") or []
        elif isinstance(val, list):
            ids = val
        else:
            ids = []
        out[str(key)] = [str(x) for x in ids]
    return out


def _load_lr_relation_chunks(stage: str) -> dict[str, list[str]]:
    path = cache_root() / "lightrag" / stage / "kv_store_relation_chunks.json"
    if not path.is_file():
        return {}
    data = json.loads(path.read_text(encoding="utf-8"))
    out: dict[str, list[str]] = {}
    for key, val in data.items():
        if isinstance(val, dict):
            ids = val.get("chunk_ids") or []
        elif isinstance(val, list):
            ids = val
        else:
            ids = []
        out[str(key)] = [str(x) for x in ids]
    return out


def _eq_database_url() -> str:
    url = (os.environ.get("DATABASE_URL") or "").strip()
    if url:
        return url
    start = Path("/tmp/edgequake-start.sh")
    if start.is_file():
        for line in start.read_text(encoding="utf-8").splitlines():
            m = re.match(r'^export\s+DATABASE_URL="([^"]*)"', line)
            if m:
                return m.group(1)
    return (
        "postgresql://edgequake:edgequake_secret@localhost:5432/edgequake"
        "?options=-c%20search_path%3Dpublic"
    )


def _load_eq_graph(workspace_id: str, graph: str) -> tuple[list[dict], int, list[dict]]:
    try:
        import psycopg2
        import psycopg2.extras
    except ImportError as e:  # noqa: BLE001
        raise SystemExit(
            "psycopg2 required for EQ audit (pip install psycopg2-binary)"
        ) from e

    url = _eq_database_url()
    # strip options query for psycopg2 if needed
    conn = psycopg2.connect(url.split("?")[0])
    try:
        with conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor) as cur:
            cur.execute(
                f"""
                SELECT ag_catalog.agtype_to_json(properties) AS props
                FROM {graph}."Node"
                WHERE ag_catalog.agtype_to_json(properties)->>'workspace_id' = %s
                """,
                (workspace_id,),
            )
            nodes = [dict(r["props"]) for r in cur.fetchall()]
            cur.execute(
                f"""
                SELECT ag_catalog.agtype_to_json(properties) AS props
                FROM {graph}."EDGE"
                WHERE ag_catalog.agtype_to_json(properties)->>'workspace_id' = %s
                """,
                (workspace_id,),
            )
            edges = [dict(r["props"]) for r in cur.fetchall()]
            edge_n = len(edges)
    finally:
        conn.close()
    return nodes, edge_n, edges


def _relation_chunk_density(edges: list[dict], lr_rel: dict[str, list[str]]) -> dict:
    """049 — multi-chunk edge lineage vs LightRAG relation_chunks."""

    def _ids(props: dict) -> list[str]:
        ids = props.get("source_chunk_ids") or props.get("source_ids") or []
        if isinstance(ids, str):
            ids = [ids] if ids.strip() else []
        if not ids:
            singular = props.get("source_chunk_id")
            if singular:
                ids = [str(singular)]
        return [str(x) for x in ids if str(x).strip()]

    eq_lens = [len(_ids(e)) for e in edges]
    lr_lens = [len(v) for v in lr_rel.values()]
    n_eq = max(len(eq_lens), 1)
    n_lr = max(len(lr_lens), 1)
    multi_eq = sum(1 for n in eq_lens if n >= 2)
    multi_lr = sum(1 for n in lr_lens if n >= 2)
    return {
        "eq_mean_chunks_per_edge": round(sum(eq_lens) / n_eq, 3) if edges else 0.0,
        "lr_mean_chunks_per_relation": round(sum(lr_lens) / n_lr, 3) if lr_rel else 0.0,
        "eq_edges_with_ge2_chunks": multi_eq,
        "eq_edges_ge2_rate": round(multi_eq / n_eq, 4) if edges else 0.0,
        "lr_relations_with_ge2_chunks": multi_lr,
        "lr_relations_ge2_rate": round(multi_lr / n_lr, 4) if lr_rel else 0.0,
        "note": "049 B6: endpoint dedupe must union chunk ids (raise eq_edges_ge2_rate).",
    }


def _count_eq_entity_vectors(workspace_id: str) -> int | None:
    """Count entity embedding rows (typed SSOT, else legacy workspace vectors)."""
    try:
        import psycopg2
    except ImportError:
        return None
    # Table naming: eq_eq_default_ws_<first8uuidhex>_vectors
    prefix = workspace_id.replace("-", "")[:8]
    table = f"eq_eq_default_ws_{prefix}_vectors"
    url = _eq_database_url().split("?")[0]
    conn = None
    try:
        conn = psycopg2.connect(url)
        with conn.cursor() as cur:
            cur.execute(
                """
                SELECT COUNT(*) FROM information_schema.tables
                WHERE table_schema='public' AND table_name='entity_embeddings'
                """
            )
            if int(cur.fetchone()[0]) > 0:
                cur.execute(
                    """
                    SELECT COUNT(*) FROM entity_embeddings
                    WHERE workspace_id = %s::uuid
                    """,
                    (workspace_id,),
                )
                typed_n = int(cur.fetchone()[0])
                if typed_n > 0:
                    return typed_n
            cur.execute(
                """
                SELECT COUNT(*) FROM information_schema.tables
                WHERE table_schema='public' AND table_name=%s
                """,
                (table,),
            )
            if int(cur.fetchone()[0]) == 0:
                return None
            cur.execute(
                f"""
                SELECT COUNT(*) FROM {table}
                WHERE metadata->>'type' = 'entity'
                """
            )
            return int(cur.fetchone()[0])
    except Exception:  # noqa: BLE001
        return None
    finally:
        if conn is not None:
            conn.close()


def _eq_entity_chunk_map(nodes: list[dict]) -> dict[str, list[str]]:
    out: dict[str, list[str]] = {}
    for n in nodes:
        name = n.get("label") or n.get("node_id") or ""
        ids = n.get("source_chunk_ids") or n.get("source_ids") or []
        if isinstance(ids, str):
            ids = [ids]
        out[str(name)] = [str(x) for x in ids]
    return out


def _classify_eq_stub_provenance(nodes: list[dict]) -> dict:
    """044 B5 — split zero-chunk nodes into UNKNOWN stubs vs named empty."""
    zero = 0
    unknown_empty = 0
    named_zero = 0
    for n in nodes:
        ids = n.get("source_chunk_ids") or n.get("source_ids") or []
        if isinstance(ids, str):
            ids = [ids] if ids.strip() else []
        if ids:
            continue
        zero += 1
        et = (n.get("entity_type") or "").strip().upper()
        desc = (n.get("description") or "").strip()
        if et == "UNKNOWN" and not desc:
            unknown_empty += 1
        else:
            named_zero += 1
    n = max(len(nodes), 1)
    return {
        "eq_zero_chunk_total": zero,
        "eq_unknown_empty_stubs": unknown_empty,
        "eq_named_zero_chunk": named_zero,
        "eq_zero_chunk_rate": round(zero / n, 4),
        "note": "B5 pass when eq_zero_chunk_rate ≤ 0.01 after placeholder provenance inherit.",
    }


def _jaccard(a: set[str], b: set[str]) -> float:
    if not a and not b:
        return 1.0
    u = a | b
    if not u:
        return 0.0
    return len(a & b) / len(u)


def run_audit(
    *,
    workspace_id: str,
    lr_stage: str,
    graph: str,
    out_dir: Path,
) -> dict:
    lr_ent = _load_lr_entity_chunks(lr_stage)
    lr_rel = _load_lr_relation_chunks(lr_stage)
    eq_nodes, eq_edge_n, eq_edges = _load_eq_graph(workspace_id, graph)
    eq_ent = _eq_entity_chunk_map(eq_nodes)

    lr_norm = {_norm_entity(k): k for k in lr_ent}
    eq_norm = {_norm_entity(k): k for k in eq_ent}
    lr_keys = set(lr_norm)
    eq_keys = set(eq_norm)
    both = lr_keys & eq_keys
    only_lr = lr_keys - eq_keys
    only_eq = eq_keys - lr_keys

    # Soft overlap: substring match for names ≥6 chars (naming/granularity gap).
    soft = 0
    for e in eq_keys:
        if len(e) < 6:
            continue
        if any(e in L or L in e for L in lr_keys if len(L) >= 6):
            soft += 1

    lr_chunk_ids: set[str] = set()
    for ids in lr_ent.values():
        lr_chunk_ids.update(ids)
    eq_chunk_ids: set[str] = set()
    for ids in eq_ent.values():
        eq_chunk_ids.update(ids)

    # Chunk ID namespaces differ (doc-hash vs uuid) — report separately.
    report = {
        "utc": datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ"),
        "workspace_id": workspace_id,
        "lr_stage": lr_stage,
        "eq_graph": graph,
        "counts": {
            "lr_entities": len(lr_ent),
            "lr_relations_with_chunks": len(lr_rel),
            "lr_entity_linked_chunks": len(lr_chunk_ids),
            "eq_nodes": len(eq_nodes),
            "eq_edges": eq_edge_n,
            "eq_entity_linked_chunks": len(eq_chunk_ids),
        },
        "entity_name_overlap": {
            "jaccard_norm": round(_jaccard(lr_keys, eq_keys), 4),
            "intersection": len(both),
            "only_lr": len(only_lr),
            "only_eq": len(only_eq),
            "lr_coverage_of_eq": round(len(both) / len(eq_keys), 4) if eq_keys else None,
            "eq_coverage_of_lr": round(len(both) / len(lr_keys), 4) if lr_keys else None,
            "eq_soft_overlap_of_eq": round(soft / len(eq_keys), 4) if eq_keys else None,
            "eq_soft_overlap_count": soft,
        },
        "source_id_note": (
            "EQ and LR use different chunk id namespaces on Acc fair pins; "
            "do not expect set equality. Compare relative linkage density and "
            "gold evidence membership in a follow-up re-ingest workspace."
        ),
        "linkage_density": {
            "lr_mean_chunks_per_entity": round(
                sum(len(v) for v in lr_ent.values()) / max(len(lr_ent), 1), 3
            ),
            "eq_mean_chunks_per_entity": round(
                sum(len(v) for v in eq_ent.values()) / max(len(eq_ent), 1), 3
            ),
            "eq_entities_with_zero_chunks": sum(1 for v in eq_ent.values() if not v),
            "lr_entities_with_zero_chunks": sum(1 for v in lr_ent.values() if not v),
        },
        # 049 B6: relation multi-chunk lineage (endpoint dedupe union).
        "relation_linkage": _relation_chunk_density(eq_edges, lr_rel),
        # 044 B5: zero-chunk debt is usually UNKNOWN stubs without source_ids.
        "stub_provenance": _classify_eq_stub_provenance(eq_nodes),
        "sample_only_lr_entities": sorted(only_lr)[:40],
        "sample_only_eq_entities": sorted(only_eq)[:40],
        "re_ingest_plan": {
            "action": "forced workspace rebuild with labeled ingest pins",
            "keep_query_pins": "P2b Acc query pins unchanged for post-ingest A* re-run",
            "never": "silent Acc ingest pin changes during query ablations",
            "next": "032 B3b workspace-scoped AGE node_id if AGE≪entity vectors",
        },
    }
    # 032: vector vs AGE density (shared-graph identity leak detector).
    vec_n = _count_eq_entity_vectors(workspace_id)
    if vec_n is not None:
        report["counts"]["eq_entity_vectors"] = vec_n
        report["identity_parity"] = {
            "eq_age_nodes": len(eq_nodes),
            "eq_entity_vectors": vec_n,
            "age_over_vectors": round(len(eq_nodes) / vec_n, 4) if vec_n else None,
            "note": "B3b pass when age_over_vectors ≈ 1.0 (±0.10); "
            "pre-fix Acc WS often ~0.09 (global node_id collision).",
        }

    out_dir.mkdir(parents=True, exist_ok=True)
    json_path = out_dir / "audit_report.json"
    json_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    md = [
        "# Ingest parity audit (028 B1)",
        "",
        f"**UTC:** {report['utc']}  ",
        f"**EQ workspace:** `{workspace_id}`  ",
        f"**LR stage:** `{lr_stage}`  ",
        f"**EQ graph:** `{graph}`  ",
        "",
        "## Counts",
        "",
        f"| Side | Entities/nodes | Edges / rels | Linked chunks |",
        f"|------|----------------|--------------|---------------|",
        f"| LR | {report['counts']['lr_entities']} | {report['counts']['lr_relations_with_chunks']} | {report['counts']['lr_entity_linked_chunks']} |",
        f"| EQ | {report['counts']['eq_nodes']} | {report['counts']['eq_edges']} | {report['counts']['eq_entity_linked_chunks']} |",
        "",
    ]
    if report.get("identity_parity"):
        ip = report["identity_parity"]
        md += [
            "## Identity parity (032 B3b)",
            "",
            f"- EQ entity vectors: **{ip['eq_entity_vectors']}**",
            f"- EQ AGE nodes (WS filter): **{ip['eq_age_nodes']}**",
            f"- AGE/vectors ratio: **{ip['age_over_vectors']}** (target ≈ 1.0)",
            f"- {ip['note']}",
            "",
        ]
    md += [
        "## Entity name overlap (normalized)",
        "",
        f"- Jaccard: **{report['entity_name_overlap']['jaccard_norm']}**",
        f"- EQ coverage of LR: **{report['entity_name_overlap']['eq_coverage_of_lr']}**",
        f"- LR coverage of EQ: **{report['entity_name_overlap']['lr_coverage_of_eq']}**",
        f"- EQ soft-overlap (substring ≥6): **{report['entity_name_overlap']['eq_soft_overlap_of_eq']}** "
        f"({report['entity_name_overlap']['eq_soft_overlap_count']}/{report['counts']['eq_nodes']})",
        f"- Only LR (sample): `{', '.join(report['sample_only_lr_entities'][:12])}`",
        "",
        "## Linkage density",
        "",
        f"- LR mean chunks/entity: {report['linkage_density']['lr_mean_chunks_per_entity']}",
        f"- EQ mean chunks/entity: {report['linkage_density']['eq_mean_chunks_per_entity']}",
        f"- EQ zero-chunk entities: {report['linkage_density']['eq_entities_with_zero_chunks']}",
        "",
    ]
    if report.get("relation_linkage"):
        rl = report["relation_linkage"]
        md += [
            "## Relation linkage (049 B6)",
            "",
            f"- EQ mean chunks/edge: **{rl['eq_mean_chunks_per_edge']}**",
            f"- LR mean chunks/relation: **{rl['lr_mean_chunks_per_relation']}**",
            f"- EQ edges ≥2 chunks: **{rl['eq_edges_with_ge2_chunks']}** "
            f"(rate **{rl['eq_edges_ge2_rate']}**)",
            f"- LR relations ≥2 chunks: **{rl['lr_relations_with_ge2_chunks']}** "
            f"(rate **{rl['lr_relations_ge2_rate']}**)",
            f"- {rl['note']}",
            "",
        ]
    if report.get("stub_provenance"):
        sp = report["stub_provenance"]
        md += [
            "## Stub provenance (044 B5)",
            "",
            f"- Zero-chunk total: **{sp['eq_zero_chunk_total']}** (rate **{sp['eq_zero_chunk_rate']}**)",
            f"- UNKNOWN empty stubs: **{sp['eq_unknown_empty_stubs']}**",
            f"- Named zero-chunk: **{sp['eq_named_zero_chunk']}**",
            f"- {sp['note']}",
            "",
        ]
    md += [
        "## Re-ingest plan",
        "",
        "- Forced new workspace + labeled ingest pins (never silent Acc pin change).",
        "- Re-run Acc query-only (A0/A1) on new workspace after B2/B3.",
        "",
        f"Artifact: `{json_path}`",
        "",
    ]
    (out_dir / "SUMMARY.md").write_text("\n".join(md), encoding="utf-8")
    return report


def main() -> int:
    ap = argparse.ArgumentParser(description="028 B1 EQ↔LR ingest audit")
    ap.add_argument(
        "--workspace-id",
        default=os.environ.get("BENCH001_EQ_WORKSPACE_ID")
        or resolve_warm_workspace_id()
        or "",
    )
    ap.add_argument(
        "--lr-stage",
        default=os.environ.get("BENCH001_LR_STAGE")
        or os.environ.get("BENCH001_STAGE")
        or "smoke",
    )
    ap.add_argument(
        "--graph",
        default=os.environ.get("EDGEQUAKE_AGE_GRAPH") or "eq_eq_default_graph",
    )
    ap.add_argument(
        "--out",
        default="",
        help="Output directory (default artifacts/ingest-audit/<utc>)",
    )
    args = ap.parse_args()
    if not args.workspace_id:
        raise SystemExit("workspace id required (BENCH001_EQ_WORKSPACE_ID / warm pointer)")

    utc = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    out = Path(args.out) if args.out else ARTIFACTS_DIR / "ingest-audit" / utc
    report = run_audit(
        workspace_id=args.workspace_id,
        lr_stage=args.lr_stage,
        graph=args.graph,
        out_dir=out,
    )
    print(json.dumps(report["counts"], indent=2))
    print(json.dumps(report["entity_name_overlap"], indent=2))
    if report.get("identity_parity"):
        print(json.dumps(report["identity_parity"], indent=2))
    print(f"→ {out / 'SUMMARY.md'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
