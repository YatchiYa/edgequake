"""bench001 CLI."""

from __future__ import annotations

import argparse
import sys

from . import __version__
from .run import doctor, freeze_smoke, report, rescore_stage, run_acc_canary_cmd, run_stage
from .progress import print_live


def _add_provider_flags(p: argparse.ArgumentParser) -> None:
    """SUT + judge provider/model knobs (defaults: Mistral Small + mistral-embed)."""
    g = p.add_argument_group("provider pins (default: mistral-small-latest + mistral-embed)")
    g.add_argument("--llm-provider", default=None, help="SUT LLM provider (default: mistral)")
    g.add_argument("--llm-model", default=None, help="SUT LLM model (default: mistral-small-latest)")
    g.add_argument("--vision-provider", default=None, help="Vision provider (default: mistral)")
    g.add_argument("--vision-model", default=None, help="Vision model (default: mistral-small-latest)")
    g.add_argument(
        "--embedding-provider",
        default=None,
        help="Embedding provider (default: mistral)",
    )
    g.add_argument(
        "--embedding-model",
        default=None,
        help="Embedding model (default: mistral-embed)",
    )
    g.add_argument(
        "--embedding-dim",
        type=int,
        default=None,
        help="Embedding dimension (default: 1024 for mistral-embed)",
    )
    g.add_argument(
        "--llm-base-url",
        default=None,
        help="OpenAI-compatible base URL for SUT LLM/embed (default: https://api.mistral.ai/v1)",
    )
    g.add_argument(
        "--judge-provider",
        default=None,
        help="Judge LLM provider (default: same as --llm-provider)",
    )
    g.add_argument(
        "--judge-model",
        default=None,
        help="Judge LLM model for generation_eval (default: same as --llm-model)",
    )
    g.add_argument(
        "--judge-base-url",
        default=None,
        help="Judge OpenAI-compatible base URL (default: same as --llm-base-url)",
    )
    g.add_argument(
        "--judge-embedding-model",
        default=None,
        help=(
            "Metric-side embed for Acc cosine term (default: mistral-embed). "
            "Paper parity: BAAI/bge-large-en-v1.5"
        ),
    )
    g.add_argument(
        "--judge-embed-backend",
        default=None,
        choices=["auto", "openai_compat", "hf_bge"],
        help="auto: mistral-embed→API, BGE→HuggingFace (default: auto)",
    )
    g.add_argument(
        "--judge-temperature",
        type=float,
        default=None,
        help="Judge LLM temperature (default: 0.0)",
    )
    g.add_argument(
        "--acc-factuality-weight",
        type=float,
        default=None,
        help="Acc = w*F1 + (1-w)*embed_cosine (default: 0.75)",
    )
    g.add_argument(
        "--answer-style",
        default=None,
        choices=["gold", "concise", "default", "verbose"],
        help="SUT answer style (default: gold — short gold-like answers for Acc F1)",
    )
    g.add_argument(
        "--profile-id",
        default=None,
        help="Profile id recorded in scorecard (default: P0_mistral_mix)",
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="bench001",
        description="SPEC-001 EdgeQuake vs LightRAG on GraphRAG-Bench",
    )
    parser.add_argument("--version", action="version", version=__version__)
    sub = parser.add_subparsers(dest="cmd", required=True)

    p_doc = sub.add_parser("doctor", help="Preflight checks")
    p_doc.add_argument("--api", default=None)
    _add_provider_flags(p_doc)

    sub.add_parser("freeze-smoke", help="Download dataset + verify smoke fixture IDs")

    def add_run_flags(p: argparse.ArgumentParser) -> None:
        p.add_argument("--api", default=None)
        p.add_argument("--dry-run", action="store_true")
        p.add_argument("--query-only", action="store_true")
        p.add_argument("--force-ingest", action="store_true")
        p.add_argument("--eq-only", action="store_true")
        p.add_argument("--lr-only", action="store_true")
        p.add_argument("--i-accept-cost", action="store_true")
        p.add_argument(
            "--accept-proxy-judge",
            action="store_true",
            help="Allow rouge_proxy judge without marking as preferred official",
        )
        p.add_argument("--max-questions", type=int, default=None)
        p.add_argument(
            "--query-concurrency",
            type=int,
            default=None,
            help="In-flight mix queries per SUT (default: BENCH001_QUERY_CONCURRENCY or 8)",
        )
        p.add_argument(
            "--eval-concurrency",
            type=int,
            default=None,
            help="In-flight official generation_eval samples (default: BENCH001_EVAL_CONCURRENCY or 16)",
        )
        _add_provider_flags(p)

    for name, help_txt in (
        ("smoke", "Run full smoke (40 stratified medical IDs)"),
        (
            "smoke-fast",
            "Fast smoke gate (8 IDs: 2/type); reuses warm smoke indexes; high concurrency",
        ),
        ("core", "Run core (cost-gated)"),
    ):
        p = sub.add_parser(name, help=help_txt)
        add_run_flags(p)
        if name == "smoke-fast":
            p.set_defaults(query_only=True)  # overridden if --force-ingest / explicit

    p_rep = sub.add_parser("report", help="Print SUMMARY / compare scorecards")
    p_rep.add_argument("stage_or_path", help="smoke|core or path to artifacts dir")
    p_rep.add_argument("--compare", default=None)

    p_live = sub.add_parser(
        "live",
        help="Print LIVE.md progress board (ETA / phase / pipeline)",
    )
    p_live.add_argument(
        "stage",
        nargs="?",
        default=None,
        help="Optional stage (default: artifacts/LIVE.md or newest)",
    )

    p_can = sub.add_parser(
        "acc-canary",
        help="Judge-only Acc instrument canaries (paraphrase high / wrong-fact low)",
    )
    p_can.add_argument(
        "--eval-concurrency",
        type=int,
        default=None,
        help="In-flight judge samples (default: BENCH001_EVAL_CONCURRENCY or 16)",
    )
    _add_provider_flags(p_can)

    p_warm = sub.add_parser(
        "resolve-warm-workspace",
        help="Print latest successful full-corpus EQ workspace id (for make bench-warm)",
    )
    p_warm.add_argument(
        "--ignore-env",
        action="store_true",
        help="Ignore BENCH001_EQ_WORKSPACE_ID and resolve from artifacts only",
    )

    p_re = sub.add_parser(
        "rescore",
        help="Re-judge frozen predictions (score-only; Acc is post-hoc)",
    )
    p_re.add_argument(
        "--source",
        default="smoke",
        help="Source stage with predictions_*.json (default: smoke)",
    )
    p_re.add_argument(
        "--stage",
        default=None,
        help="Output stage name (default: <source>-rescore or <source>-paper)",
    )
    p_re.add_argument("--eval-concurrency", type=int, default=None)
    p_re.add_argument("--accept-proxy-judge", action="store_true")
    _add_provider_flags(p_re)

    args = parser.parse_args(argv)

    def _provider_kwargs() -> dict:
        return dict(
            llm_provider=getattr(args, "llm_provider", None),
            llm_model=getattr(args, "llm_model", None),
            vision_provider=getattr(args, "vision_provider", None),
            vision_model=getattr(args, "vision_model", None),
            embedding_provider=getattr(args, "embedding_provider", None),
            embedding_model=getattr(args, "embedding_model", None),
            embedding_dim=getattr(args, "embedding_dim", None),
            llm_base_url=getattr(args, "llm_base_url", None),
            judge_provider=getattr(args, "judge_provider", None),
            judge_model=getattr(args, "judge_model", None),
            judge_base_url=getattr(args, "judge_base_url", None),
            judge_embedding_model=getattr(args, "judge_embedding_model", None),
            profile_id=getattr(args, "profile_id", None),
        )

    def _apply_judge_tune_env() -> None:
        import os

        if getattr(args, "judge_temperature", None) is not None:
            os.environ["BENCH001_JUDGE_TEMPERATURE"] = str(args.judge_temperature)
        if getattr(args, "acc_factuality_weight", None) is not None:
            os.environ["BENCH001_ACC_FACTUALITY_WEIGHT"] = str(args.acc_factuality_weight)
        if getattr(args, "judge_embed_backend", None):
            os.environ["BENCH001_JUDGE_EMBED_BACKEND"] = args.judge_embed_backend
        if getattr(args, "answer_style", None):
            os.environ["BENCH001_ANSWER_STYLE"] = args.answer_style

    if args.cmd == "doctor":
        from .acc_env import ensure_acc_api_keys
        from .profiles import resolve_pins, set_active_pins

        ensure_acc_api_keys(verbose=True)
        _apply_judge_tune_env()
        set_active_pins(resolve_pins(**_provider_kwargs()))
        return doctor(base_url=args.api)
    if args.cmd == "freeze-smoke":
        freeze_smoke()
        return 0
    if args.cmd in {"smoke", "smoke-fast", "core"}:
        import os

        from .acc_env import (
            apply_acc_publication_pins,
            apply_acc_speed_defaults,
            ensure_acc_api_keys,
        )

        ensure_acc_api_keys(verbose=True)
        publication = os.environ.get("BENCH001_PUBLICATION", "").strip().lower() in {
            "1",
            "true",
            "yes",
        }
        if args.cmd == "smoke" and (
            publication or os.environ.get("BENCH001_FULL_ACC", "").strip() in {"1", "true", "yes"}
        ):
            # Publication / full Acc: force mistral-small + mistral-embed + full corpus.
            apply_acc_publication_pins(full_corpus=True, verbose=True)
        elif args.cmd == "smoke-fast":
            apply_acc_speed_defaults()
            # Still force model pins so ollama vision bleed cannot win scorecard lineage.
            apply_acc_publication_pins(full_corpus=False, clear_capped_workspace=False, verbose=True)
        _apply_judge_tune_env()
        q_conc = args.query_concurrency
        e_conc = args.eval_concurrency
        query_only = bool(args.query_only)
        if args.cmd == "smoke-fast":
            if q_conc is None:
                q_conc = int(os.environ.get("BENCH001_QUERY_CONCURRENCY", "12"))
            if e_conc is None:
                e_conc = int(os.environ.get("BENCH001_EVAL_CONCURRENCY", "16"))
            # Default query-only (set_defaults); --force-ingest implies re-ingest.
            if args.force_ingest:
                query_only = False
        # Publication Acc defaults to force-ingest unless explicitly query-only.
        if args.cmd == "smoke" and publication and not args.query_only:
            if os.environ.get("BENCH001_FORCE_INGEST", "").strip() in {"1", "true", "yes"}:
                query_only = False
                args.force_ingest = True
        return run_stage(
            args.cmd,
            api=args.api,
            dry_run=args.dry_run,
            query_only=query_only,
            force_ingest=args.force_ingest,
            eq_only=args.eq_only,
            lr_only=args.lr_only,
            i_accept_cost=args.i_accept_cost,
            accept_proxy_judge=args.accept_proxy_judge,
            max_questions=args.max_questions,
            concurrency=q_conc,
            eval_concurrency_n=e_conc,
            **_provider_kwargs(),
        )
    if args.cmd == "resolve-warm-workspace":
        from .warm_workspace import resolve_or_raise, resolve_warm_workspace_id

        if args.ignore_env:
            wid = resolve_warm_workspace_id(prefer_env=False)
            if not wid:
                raise SystemExit(
                    "No warm EQ workspace found in artifacts. "
                    "Run `make bench` once (cold ingest)."
                )
        else:
            wid = resolve_or_raise()
        print(wid)
        return 0
    if args.cmd == "report":
        return report(args.stage_or_path, compare=args.compare)
    if args.cmd == "live":
        return print_live(args.stage)
    if args.cmd == "acc-canary":
        from .acc_env import ensure_acc_api_keys

        ensure_acc_api_keys(verbose=True)
        _apply_judge_tune_env()
        from .profiles import resolve_pins, set_active_pins

        set_active_pins(resolve_pins(**_provider_kwargs()))
        return run_acc_canary_cmd(eval_concurrency_n=args.eval_concurrency)
    if args.cmd == "rescore":
        from .acc_env import ensure_acc_api_keys

        ensure_acc_api_keys(verbose=True)
        _apply_judge_tune_env()
        return rescore_stage(
            stage=args.stage or args.source,
            source_stage=args.source,
            eval_concurrency_n=args.eval_concurrency,
            accept_proxy_judge=args.accept_proxy_judge,
            **_provider_kwargs(),
        )
    return 1


if __name__ == "__main__":
    sys.exit(main())
