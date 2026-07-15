"""bench047 CLI."""

from __future__ import annotations

import argparse
import json
import os
import sys

from . import __version__
from .download import download_pdfs, download_qa
from .paths import ARTIFACTS_DIR
from .profiles import PROFILES, get_profile
from .protocol import DEFAULT_BENCH_PROFILE
from .run import doctor, run_stage
from .subset import freeze_core, freeze_smoke, read_doc_ids


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="bench047", description="SPEC-047 EdgeQuake RAG eval")
    parser.add_argument("--version", action="version", version=__version__)
    sub = parser.add_subparsers(dest="cmd", required=True)

    sub.add_parser("download-qa", help="Download Q&A parquet")
    p_pdf = sub.add_parser("download-pdfs", help="Download PDFs (smoke fixture or --all)")
    p_pdf.add_argument("--all", action="store_true")
    p_pdf.add_argument("--fixture", default="smoke_doc_ids_v1.txt")

    sub.add_parser("freeze-smoke", help="Freeze stratified 10-doc smoke list")
    p_core = sub.add_parser("freeze-core", help="Freeze ~40-doc core list")
    p_core.add_argument("-n", type=int, default=40)

    p_doc = sub.add_parser("doctor", help="Check API + keys + provider profile")
    p_doc.add_argument("--profile", default=DEFAULT_BENCH_PROFILE)
    p_doc.add_argument("--api", default=None)

    def add_run_flags(p: argparse.ArgumentParser) -> None:
        p.add_argument("--profile", default=DEFAULT_BENCH_PROFILE, choices=sorted(PROFILES))
        p.add_argument("--api", default=None)
        p.add_argument("--resume", action="store_true", default=True)
        p.add_argument("--no-resume", action="store_true")
        p.add_argument("--i-accept-cost", action="store_true")
        p.add_argument("--max-docs", type=int, default=None)
        p.add_argument("--max-questions", type=int, default=None)
        p.add_argument("--ingest-only", action="store_true")
        p.add_argument("--query-only", action="store_true")
        p.add_argument(
            "--document-scope",
            action="store_true",
            help="W2: pass DocumentFilter.document_ids for the question's PDF (not gold pages)",
        )
        p.add_argument(
            "--workers",
            type=int,
            default=int(os.environ.get("BENCH047_WORKERS", "1")),
            help="Parallel query workers (default: BENCH047_WORKERS or 1)",
        )
        p.add_argument(
            "--ingest-workers",
            type=int,
            default=int(os.environ.get("BENCH047_INGEST_WORKERS", "10")),
            help="Parallel document ingest workers per workspace "
            "(default: BENCH047_INGEST_WORKERS or 10)",
        )

    for name in ("smoke", "core", "full"):
        p = sub.add_parser(name, help=f"Run {name} stage")
        add_run_flags(p)

    p_rep = sub.add_parser("report", help="Print SUMMARY / compare scorecards")
    p_rep.add_argument("stage_or_path", help="smoke|core|full or path to artifacts dir")
    p_rep.add_argument("--compare", default=None, help="other artifacts dir")

    p_fid = sub.add_parser(
        "fidelity",
        help="W1: audit whether gold answers appear in ingested evidence-page markdown",
    )
    p_fid.add_argument("stage", nargs="?", default="smoke", choices=("smoke", "core", "full"))
    p_fid.add_argument("--api", default=None)
    p_fid.add_argument(
        "--max-samples",
        type=int,
        default=None,
        help="DEBUG only: truncate audit (requires --allow-partial; not gateable)",
    )
    p_fid.add_argument(
        "--allow-partial",
        action="store_true",
        help="Permit --max-samples; marks fidelity report gateable=false",
    )

    p_watch = sub.add_parser(
        "watch-ingest",
        help="Live ingest telemetry (doc stages + P6 unique-before-embed log markers)",
    )
    p_watch.add_argument("--api", default="http://127.0.0.1:8090")
    p_watch.add_argument("--workspace", required=True)
    p_watch.add_argument("--log", default="/tmp/edgequake-backend.log")
    p_watch.add_argument("--interval", type=float, default=15.0)

    args = parser.parse_args(argv)

    if args.cmd == "download-qa":
        download_qa()
        return 0
    if args.cmd == "download-pdfs":
        if args.all:
            download_pdfs()
        else:
            download_pdfs(read_doc_ids(args.fixture))
        return 0
    if args.cmd == "freeze-smoke":
        freeze_smoke()
        return 0
    if args.cmd == "freeze-core":
        freeze_core(n=args.n)
        return 0
    if args.cmd == "doctor":
        return doctor(base_url=args.api, profile=get_profile(args.profile))
    if args.cmd in {"smoke", "core", "full"}:
        resume = not args.no_resume
        return run_stage(
            args.cmd,
            profile_name=args.profile,
            base_url=args.api,
            resume=resume,
            accept_cost=args.i_accept_cost or args.cmd == "smoke",
            max_docs=args.max_docs,
            max_questions=args.max_questions,
            ingest_only=args.ingest_only,
            query_only=args.query_only,
            document_scope=args.document_scope,
            workers=max(1, args.workers),
            ingest_workers=max(1, args.ingest_workers),
        )
    if args.cmd == "report":
        path = args.stage_or_path
        if path in {"smoke", "core", "full"}:
            art = ARTIFACTS_DIR / path
        else:
            from pathlib import Path

            art = Path(path)
        summary = art / "SUMMARY.md"
        score = art / "scorecard.json"
        if summary.exists():
            print(summary.read_text())
        if score.exists() and args.compare:
            from pathlib import Path as _P

            from .protocol import paired_acc_delta
            from .score import load_jsonl

            a = json.loads(score.read_text())
            b_path = _P(args.compare)
            b = json.loads((b_path / "scorecard.json").read_text())
            print("\n## Compare")
            print(f"F1: {a['metrics']['f1']:.4f} vs {b['metrics']['f1']:.4f} "
                  f"(Δ={a['metrics']['f1']-b['metrics']['f1']:+.4f})")
            print(f"Acc: {a['metrics']['accuracy']:.4f} vs {b['metrics']['accuracy']:.4f} "
                  f"(Δ={a['metrics']['accuracy']-b['metrics']['accuracy']:+.4f})")
            ra = (a.get("ops") or {}).get("retrieval") or {}
            rb = (b.get("ops") or {}).get("retrieval") or {}
            for key in ("page_hit@1", "page_hit@3", "page_hit@5", "page_hit@10"):
                if key in ra or key in rb:
                    va = ra.get(key)
                    vb = rb.get(key)
                    if va is not None and vb is not None:
                        print(f"{key}: {va:.4f} vs {vb:.4f} (Δ={va-vb:+.4f})")
            # Exclusive Chart Acc
            ae = ((a.get("slices") or {}).get("by_evidence_source_exclusive") or {}).get("Chart")
            be = ((b.get("slices") or {}).get("by_evidence_source_exclusive") or {}).get("Chart")
            am = ((a.get("slices") or {}).get("by_evidence_source") or {}).get("Chart")
            bm = ((b.get("slices") or {}).get("by_evidence_source") or {}).get("Chart")
            if ae and be:
                print(
                    f"Chart exclusive Acc: {ae['accuracy']:.4f} (n={ae['n']}) vs "
                    f"{be['accuracy']:.4f} (n={be['n']}) "
                    f"(Δ={ae['accuracy']-be['accuracy']:+.4f})"
                )
            if am and bm:
                print(
                    f"Chart multi-label Acc: {am['accuracy']:.4f} (n={am['n']}) vs "
                    f"{bm['accuracy']:.4f} (n={bm['n']}) "
                    f"(Δ={am['accuracy']-bm['accuracy']:+.4f})"
                )
            # Paired attribution (this run = a, baseline = b)
            pred_a = art / "predictions.jsonl"
            pred_b = b_path / "predictions.jsonl"
            if pred_a.exists() and pred_b.exists():
                delta = paired_acc_delta(load_jsonl(pred_b), load_jsonl(pred_a))
                print("\n### Paired Acc Δ attribution (a vs compare=baseline)")
                print(
                    f"n_paired={delta['n_paired']} improved={delta['n_improved']} "
                    f"worsened={delta['n_worsened']} ΔAcc={delta['delta_acc']:+.4f}"
                )
                for k, v in (delta.get("acc_points") or {}).items():
                    print(f"  {k}: {v:+.4f} Acc points")
                print(f"  note: {delta.get('note')}")
            # Fidelity long gate if present
            for label, p in (("this", art), ("compare", b_path)):
                fid = p / "fidelity.json"
                if not fid.exists():
                    continue
                fr = json.loads(fid.read_text())
                agg = fr.get("aggregate") or {}
                gates = agg.get("gates") or {}
                print(
                    f"\n### Fidelity ({label}): gateable={fr.get('gateable')} "
                    f"n={agg.get('n')} long={agg.get('answer_in_evidence_rate_long')} "
                    f"Chart_long={gates.get('chart_a_in_e_long')} "
                    f"Table_long={gates.get('table_a_in_e_long')}"
                )
        return 0
    if args.cmd == "fidelity":
        from .fidelity_audit import run_fidelity_audit

        run_fidelity_audit(
            args.stage,
            base_url=args.api,
            max_samples=args.max_samples,
            allow_partial=args.allow_partial,
        )
        return 0
    if args.cmd == "watch-ingest":
        from .watch_ingest import watch
        from pathlib import Path

        watch(
            api=args.api,
            workspace_id=args.workspace,
            log_path=Path(args.log),
            interval_s=args.interval,
        )
        return 0
    return 1


if __name__ == "__main__":
    sys.exit(main())
