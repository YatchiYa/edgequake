"""Orchestrate SPEC-001 dual-SUT stages."""

from __future__ import annotations

import json
import os
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Any

from . import __version__
from .client import EdgeQuakeClient
from .download import download_dataset
from .eval_score import score_predictions
from .fixtures import (
    freeze_medical_full_verify,
    freeze_publish_verify,
    freeze_smoke_verify,
    load_corpus,
    select_questions,
    verify_fixtures,
)
from .lightrag_runner import available as lr_available
from .lightrag_runner import run_lightrag, stub_predictions
from .profiles import eval_concurrency as eval_concurrency_pin
from .profiles import query_concurrency
from .paths import (
    CORE_FIXTURE,
    DATASET_REVISION,
    FAST_SMOKE_FIXTURE,
    MEDICAL_FULL_FIXTURE,
    PUBLISH_FIXTURE,
    SMOKE_FIXTURE,
    api_base,
    corpus_path,
    lightrag_repo,
    questions_path,
    stage_artifact_dir,
)
from .progress import (
    archive_run,
    begin_run,
    empty_context_rate,
    format_duration,
    mark_phase,
    print_unit_progress,
    run_elapsed_s,
    write_progress_md,
)
from .scorecard import build_scorecard, write_summary


def doctor(*, base_url: str | None = None) -> int:
    print(f"bench001 {__version__}")
    from .acc_env import ensure_acc_api_keys

    ensure_acc_api_keys(verbose=True)
    base = base_url or api_base()
    fx = verify_fixtures()
    print(
        f"fixtures: smoke_n={fx['smoke_n']} fast_n={fx.get('fast_n')} "
        f"publish_n={fx.get('publish_n')} core_n={fx['core_n']}"
    )
    ok = True

    # dataset cache
    med_q = questions_path("medical")
    print(f"medical questions cached: {med_q.exists()} ({med_q})")
    print(f"medical corpus cached: {corpus_path('medical').exists()}")

    # API
    try:
        client = EdgeQuakeClient(base, workspace="bench001-doctor")
        h = client.health()
        print(f"EQ health @ {base}: status={h.get('status')} storage={h.get('storage_mode')}")
        providers = h.get("providers") or {}
        llm = providers.get("llm") or {}
        print(
            f"EQ providers: llm={llm.get('name')}/{llm.get('model')} "
            f"embed={(providers.get('embedding') or {}).get('name')}"
        )
        qe = ((h.get("operational") or {}).get("query_engine") or {})
        mix_fusion = qe.get("mix_fusion")
        print(f"EQ mix_fusion={mix_fusion} (Acc law 086 wants round_robin)")
        if mix_fusion and str(mix_fusion).lower() == "rrf":
            print(
                "WARN: mix_fusion=rrf — Acc law 086 is round_robin "
                "(set EDGEQUAKE_MIX_FUSION=round_robin + BENCH001_ALLOW_ROUND_ROBIN=1)"
            )
    except Exception as exc:  # noqa: BLE001
        print(f"EQ health FAIL @ {base}: {exc}")
        ok = False

    from .profiles import active_pins, mistral_api_key, resolve_pins, set_active_pins, sut_api_key

    pins = resolve_pins()
    set_active_pins(pins)
    print(f"profile: {pins.profile_id}")
    for k, v in pins.lineage().items():
        print(f"  {k}: {v}")
    has_key = bool(sut_api_key(pins))
    print(f"SUT API key present: {has_key}")
    if pins.llm_provider == "mistral" and not mistral_api_key():
        print("WARN: MISTRAL_API_KEY required for default Mistral SUT pins")
        ok = False
    elif not has_key:
        print("WARN: API key required for live dual-SUT (not dry-run)")
        ok = False

    # LightRAG
    repo = lightrag_repo()
    print(f"LightRAG repo: {repo}")
    print(f"LightRAG importable: {lr_available()}")

    if not fx["smoke_exists"] or fx["smoke_n"] != 40:
        print("WARN: smoke fixture missing or wrong size")
        ok = False
    if not fx.get("publish_exists") or fx.get("publish_n") != 200:
        print("WARN: medical-publish fixture missing or wrong size (expected n=200)")
        ok = False
    if not fx.get("medical_full_exists") or int(fx.get("medical_full_n") or 0) < 2000:
        print(
            "WARN: medical-full fixture missing or wrong size "
            f"(expected n≈2062, got {fx.get('medical_full_n')})"
        )
        ok = False
    return 0 if ok else 1


def freeze_smoke() -> None:
    download_dataset(revision=DATASET_REVISION)
    freeze_smoke_verify()
    freeze_publish_verify()
    freeze_medical_full_verify()
    print("freeze-smoke OK (smoke + medical-publish + medical-full)")


def _corpus_texts_for_questions(questions: list[dict[str, Any]]) -> list[str]:
    subsets = sorted({q.get("_subset") or "medical" for q in questions})
    texts: list[str] = []
    for subset in subsets:
        for item in load_corpus(subset):
            ctx = item.get("context") or ""
            if ctx:
                texts.append(ctx)
    return texts


def _prepare_ingest_corpus(
    questions: list[dict[str, Any]],
) -> tuple[list[str], dict[str, Any]]:
    """Load + optionally cap corpus for fast Acc force-ingest."""
    from .ingest_cap import apply_ingest_cap

    raw = _corpus_texts_for_questions(questions)
    capped, meta = apply_ingest_cap(raw)
    if meta.get("ingest_capped"):
        print(
            "INGEST CAP: truncated corpus for fast Acc "
            f"(max_chars={meta.get('ingest_max_chars')} "
            f"orig={meta.get('corpus_chars_original')} "
            f"eff={meta.get('corpus_chars_effective')})",
            flush=True,
        )
    return capped, meta


def _stage_ms_from_eq_resp(resp: dict[str, Any] | None) -> dict[str, Any]:
    """021 F3: project API QueryStats into prediction stage fields."""
    if not isinstance(resp, dict):
        return {}
    stats = resp.get("stats") or {}
    if not isinstance(stats, dict):
        return {}
    out: dict[str, Any] = {}
    for key in (
        "embedding_time_ms",
        "keyword_time_ms",
        "retrieval_time_ms",
        "rerank_time_ms",
        "generation_time_ms",
        "total_time_ms",
        "arm_local_ms",
        "arm_global_ms",
        "arm_naive_ms",
    ):
        val = stats.get(key)
        if val is not None:
            try:
                out[key] = int(val)
            except (TypeError, ValueError):
                pass
    # 022 P3a: surface LLM query_intent for Summarize truncation audits.
    intent = (
        stats.get("query_intent")
        or (resp.get("metadata") or {}).get("query_intent")
        or ((resp.get("context") or {}) if isinstance(resp.get("context"), dict) else {}).get(
            "query_intent"
        )
    )
    if isinstance(resp.get("context"), dict):
        meta = (resp.get("context") or {}).get("metadata") or {}
        if isinstance(meta, dict) and meta.get("query_intent"):
            intent = meta.get("query_intent")
    if intent:
        out["query_intent"] = str(intent)
    return out


def _eq_predict_one(
    client: EdgeQuakeClient,
    q: dict[str, Any],
    *,
    mode: str,
) -> dict[str, Any]:
    from .judge_tune import system_prompt_for_style

    t0 = time.perf_counter()
    stage: dict[str, Any] = {}
    try:
        resp = client.query(
            q["question"],
            mode=mode,
            system_prompt=system_prompt_for_style(),
            question_type=q.get("question_type"),
        )
        answer, context = client.extract_answer(resp)
        latency_ms = int((time.perf_counter() - t0) * 1000)
        stage = _stage_ms_from_eq_resp(resp)
    except Exception as exc:  # noqa: BLE001
        answer, context, latency_ms = "", f"error: {exc}", int((time.perf_counter() - t0) * 1000)
    gold = q.get("answer") or ""
    row: dict[str, Any] = {
        "id": q["id"],
        "question": q["question"],
        "source": q.get("_subset") or str(q.get("source", "")).lower(),
        "context": [context] if context else [""],
        "evidence": q.get("evidence") or [],
        "question_type": q["question_type"],
        "generated_answer": answer,
        "ground_truth": gold,
        "gold_answer": gold,
        "latency_ms": latency_ms,
    }
    row.update(stage)
    return row


def _run_eq(
    *,
    client: EdgeQuakeClient,
    corpus_texts: list[str],
    questions: list[dict[str, Any]],
    query_only: bool,
    force_ingest: bool,
    mode: str = "mix",
    concurrency: int | None = None,
) -> tuple[list[dict[str, Any]], float]:
    from .fair_pins import chunk_overlap_token_size, chunk_token_size

    ingest_wall = 0.0
    if not client.workspace_id:
        ws = client.ensure_workspace()
        print(f"EQ workspace={ws}")
    # force_ingest wins over query_only (CLI normally clears query_only already).
    do_ingest = (not query_only) or force_ingest
    if do_ingest:
        # NOTE: do not locally re-import mark_phase here — that binds it as a
        # function-local name and breaks query-only (_tick uses mark_phase).
        t0 = time.perf_counter()
        csize = chunk_token_size()
        coverlap = chunk_overlap_token_size()
        stage_name = os.environ.get("BENCH001_PROGRESS_STAGE") or "smoke-fast"
        print(
            f"EQ ingest chunk_token_size={csize} overlap={coverlap} "
            f"docs={len(corpus_texts)} force={force_ingest} "
            f"chars={[len(t) for t in corpus_texts]}",
            flush=True,
        )
        mark_phase(
            stage_name,
            "ingest_eq",
            status="running",
            detail=f"docs={len(corpus_texts)} chunk={csize}/{coverlap}",
            counts={"ingest_docs_total": len(corpus_texts), "ingest_docs_done": 0},
        )
        # Medical may be one giant blob — upload each context as a doc.
        for i, text in enumerate(corpus_texts):
            title = f"graphrag-bench-{i}"
            resp = client.upload_text(
                text,
                title=title,
                chunk_size=csize,
                chunk_overlap=coverlap,
                async_processing=True,
            )
            doc_id = resp.get("document_id") or resp.get("id")
            task_id = resp.get("task_id")
            if not doc_id:
                raise RuntimeError(f"upload missing document_id: {resp}")
            print(
                f"EQ upload {i+1}/{len(corpus_texts)} doc={doc_id} "
                f"task={task_id} status={resp.get('status')} chars={len(text)}",
                flush=True,
            )
            print(f"EQ workspace_id={client.workspace_id}", flush=True)

            def _on_prog(info: dict[str, Any], *, _i: int = i) -> None:
                from .progress import format_duration

                eta = info.get("eta_s")
                eta_s = format_duration(eta) if eta is not None else "—"
                pct = info.get("pct")
                pct_s = f"{float(pct)*100:.0f}%" if pct is not None else "—"
                chunks = info.get("chunk_count")
                mark_phase(
                    stage_name,
                    "ingest_eq",
                    status="running",
                    detail=(
                        f"doc {_i+1}/{len(corpus_texts)} "
                        f"{info.get('doc_status')}/{info.get('stage')} "
                        f"pct={pct_s} eta={eta_s}"
                        + (f" chunks={chunks}" if chunks not in (None, 0) else "")
                    ),
                    counts={
                        "ingest_docs_total": len(corpus_texts),
                        "ingest_docs_done": _i,
                        "n_docs": len(corpus_texts),
                        "chunk_count": chunks,
                        "ingest_pct": info.get("pct"),
                        "ingest_eta_s": info.get("eta_s"),
                        "ingest_elapsed_s": info.get("elapsed_s"),
                    },
                    phase_elapsed_s=float(info.get("elapsed_s") or 0.0) or None,
                    quiet=True,
                )

            try:
                client.wait_document(
                    str(doc_id),
                    task_id=str(task_id) if task_id else None,
                    timeout_s=float(os.environ.get("BENCH001_INGEST_TIMEOUT_S", "7200")),
                    progress_cb=_on_prog,
                )
                # 032: hold until entity vectors survive saga window (ENOSPC rollback).
                settle_s = float(os.environ.get("BENCH001_INGEST_SETTLE_S", "25"))
                min_ent = int(os.environ.get("BENCH001_MIN_ENTITY_VECTORS", "100"))
                client.assert_ingest_settled(
                    str(doc_id),
                    min_entity_vectors=min_ent,
                    settle_s=settle_s,
                )
            except Exception as exc:  # noqa: BLE001
                mark_phase(
                    stage_name,
                    "ingest_eq",
                    status="failed",
                    detail=str(exc)[:240],
                )
                raise
            print(f"EQ ingest complete doc={doc_id}", flush=True)
            mark_phase(
                stage_name,
                "ingest_eq",
                status="running",
                detail=f"doc {i+1}/{len(corpus_texts)} done",
                counts={
                    "ingest_docs_total": len(corpus_texts),
                    "ingest_docs_done": i + 1,
                },
            )
        ingest_wall = time.perf_counter() - t0
        mark_phase(
            stage_name,
            "ingest_eq",
            status="done",
            detail=f"wall_s={ingest_wall:.1f}",
            counts={"ingest_wall_s": ingest_wall},
        )

    n = query_concurrency(concurrency)
    print(f"EQ querying {len(questions)} questions concurrency={n}", flush=True)
    preds: list[dict[str, Any] | None] = [None] * len(questions)
    t_query = time.perf_counter()

    def _tick(done: int, qid: str | None = None) -> None:
        elapsed = max(time.perf_counter() - t_query, 1e-6)
        extra = f"id={qid}" if qid else ""
        print_unit_progress(
            "EQ query",
            done,
            len(questions),
            elapsed_s=elapsed,
            extra=extra,
        )
        stage_name = os.environ.get("BENCH001_PROGRESS_STAGE") or "smoke-fast"
        mark_phase(
            stage_name,
            "query_eq",
            status="running",
            detail=f"EQ query {done}/{len(questions)} {extra}".strip(),
            done=done,
            total=len(questions),
            phase_elapsed_s=elapsed,
            quiet=True,
        )

    if n == 1 or len(questions) <= 1:
        for i, q in enumerate(questions):
            qid = str(q.get("id") or q.get("question_id") or i)
            print(f"  EQ query start {qid}", flush=True)
            preds[i] = _eq_predict_one(client, q, mode=mode)
            if (i + 1) == 1 or (i + 1) % 5 == 0 or (i + 1) == len(questions):
                _tick(i + 1, qid)
    else:
        with ThreadPoolExecutor(max_workers=n) as pool:
            futs = {
                pool.submit(_eq_predict_one, client, q, mode=mode): (i, q)
                for i, q in enumerate(questions)
            }
            done = 0
            for fut in as_completed(futs):
                i, q = futs[fut]
                preds[i] = fut.result()
                done += 1
                qid = str(q.get("id") or q.get("question_id") or i)
                if done == 1 or done % 5 == 0 or done == len(questions):
                    _tick(done, qid)
    return [p for p in preds if p is not None], ingest_wall


def run_stage(
    stage: str,
    *,
    api: str | None = None,
    dry_run: bool = False,
    query_only: bool = False,
    force_ingest: bool = False,
    eq_only: bool = False,
    lr_only: bool = False,
    i_accept_cost: bool = False,
    accept_proxy_judge: bool = False,
    max_questions: int | None = None,
    concurrency: int | None = None,
    eval_concurrency_n: int | None = None,
    llm_provider: str | None = None,
    llm_model: str | None = None,
    vision_provider: str | None = None,
    vision_model: str | None = None,
    embedding_provider: str | None = None,
    embedding_model: str | None = None,
    embedding_dim: int | None = None,
    llm_base_url: str | None = None,
    judge_provider: str | None = None,
    judge_model: str | None = None,
    judge_base_url: str | None = None,
    judge_embedding_model: str | None = None,
    profile_id: str | None = None,
) -> int:
    from .profiles import resolve_pins, set_active_pins

    if stage == "core" and not i_accept_cost and not dry_run:
        print("core requires --i-accept-cost (or --dry-run)")
        return 2

    pins = resolve_pins(
        llm_provider=llm_provider,
        llm_model=llm_model,
        vision_provider=vision_provider,
        vision_model=vision_model,
        embedding_provider=embedding_provider,
        embedding_model=embedding_model,
        embedding_dim=embedding_dim,
        llm_base_url=llm_base_url,
        judge_provider=judge_provider,
        judge_model=judge_model,
        judge_base_url=judge_base_url,
        judge_embedding_model=judge_embedding_model,
        profile_id=profile_id,
    )
    set_active_pins(pins)
    print(f"provider profile={pins.profile_id}", flush=True)
    for k, v in pins.lineage().items():
        print(f"  lineage.{k}={v}", flush=True)

    if stage == "smoke-fast":
        fixture = FAST_SMOKE_FIXTURE
    elif stage == "smoke":
        fixture = SMOKE_FIXTURE
    elif stage == "medical-mid":
        fixture = PUBLISH_FIXTURE
    elif stage == "medical-full":
        fixture = MEDICAL_FULL_FIXTURE
    else:
        fixture = CORE_FIXTURE
    fixture_id = fixture.replace(".txt", "")
    # Never clobber live smoke/core artifacts with dry-run or truncated debug runs.
    art_stage = stage
    if dry_run:
        art_stage = f"{stage}-dry-run"
    elif max_questions is not None and stage != "smoke-fast":
        art_stage = f"{stage}-debug"
    art = stage_artifact_dir(art_stage)
    # Fast smoke / medical-mid reuse warm medical indexes (EQ workspace env + LR smoke dir).
    # When ingest is capped, isolate workspaces/dirs so full-corpus caches are not reused.
    from .ingest_cap import eq_workspace_name_for_cap, lr_stage_for_cap

    # medical-mid/full share the same FULL medical corpus as smoke — reuse smoke LR/EQ indexes.
    index_stage = (
        "smoke" if stage in {"smoke-fast", "medical-mid", "medical-full"} else stage
    )
    lr_index_stage = lr_stage_for_cap(index_stage)
    default_ws = (
        "bench001-smoke"
        if stage in {"smoke-fast", "medical-mid", "medical-full"}
        else f"bench001-{stage}"
    )
    eq_workspace_name = eq_workspace_name_for_cap(
        os.environ.get("BENCH001_EQ_WORKSPACE_NAME") or default_ws
    )
    os.environ["BENCH001_PROGRESS_STAGE"] = art_stage

    if not dry_run:
        download_dataset(revision=DATASET_REVISION)

    questions = select_questions(fixture)
    if max_questions is not None:
        questions = questions[:max_questions]

    corpus_texts, ingest_meta = _prepare_ingest_corpus(questions)
    from .acc_env import assert_publication_ingest, backend_pin_mismatches

    assert_publication_ingest(ingest_meta)
    q_conc = query_concurrency(concurrency)
    e_conc = eval_concurrency_pin(eval_concurrency_n)
    if concurrency is not None:
        os.environ["BENCH001_QUERY_CONCURRENCY"] = str(q_conc)
    if eval_concurrency_n is not None:
        os.environ["BENCH001_EVAL_CONCURRENCY"] = str(e_conc)
    from .fair_pins import chunk_overlap_token_size, chunk_token_size

    csize = chunk_token_size()
    coverlap = chunk_overlap_token_size()
    corpus_chars = sum(len(t) for t in corpus_texts)
    meta = {
        "stage": stage,
        "fixture_id": fixture_id,
        "n_questions": len(questions),
        "n_corpus_docs": len(corpus_texts),
        "n_corpus_chunks": len(corpus_texts),
        "corpus_chars": corpus_chars,
        "chunk_token_size": csize,
        "chunk_overlap_token_size": coverlap,
        "dry_run": dry_run,
        "query_only": query_only,
        "eq_only": eq_only,
        "lr_only": lr_only,
        "query_concurrency": q_conc,
        "eval_concurrency": e_conc,
        "lineage": pins.lineage(),
        "provider_pins": pins.to_pin_fields(),
        "ingest": ingest_meta,
        "lr_index_stage": lr_index_stage,
        "eq_workspace_name": eq_workspace_name,
        "publication": os.environ.get("BENCH001_PUBLICATION", ""),
    }
    (art / "meta.json").write_text(json.dumps(meta, indent=2), encoding="utf-8")
    print(f"query_concurrency={q_conc} (fair EQ↔LR) eval_concurrency={e_conc}", flush=True)
    print(
        f"corpus docs={len(corpus_texts)} chars={corpus_chars} "
        f"chunk={csize}/{coverlap} capped={ingest_meta.get('ingest_capped')} "
        f"max_chars={ingest_meta.get('ingest_max_chars')}",
        flush=True,
    )
    begin_run(
        art_stage,
        detail=(
            f"n={len(questions)} docs={len(corpus_texts)} chars={corpus_chars} "
            f"chunk={csize}/{coverlap} q∥={q_conc} eval∥={e_conc} "
            f"force_ingest={force_ingest} query_only={query_only}"
        ),
    )
    mark_phase(
        art_stage,
        "prepare",
        status="done",
        detail=(
            f"n_questions={len(questions)} docs={len(corpus_texts)} "
            f"chars={corpus_chars} chunk={csize}/{coverlap}"
        ),
        counts={
            "n_questions": len(questions),
            "n_docs": len(corpus_texts),
            "corpus_chars": corpus_chars,
            "chunk_token_size": csize,
            "chunk_overlap_token_size": coverlap,
            "ingest_capped": ingest_meta.get("ingest_capped"),
            "query_concurrency": q_conc,
            "eval_concurrency": e_conc,
        },
    )

    eq_preds: list[dict[str, Any]] = []
    lr_preds: list[dict[str, Any]] = []
    ingest_wall = 0.0
    reasons: list[str] = []
    eq_workspace_id: str | None = None

    if dry_run:
        mark_phase(art_stage, "query", status="running", detail="dry_run stubs")
        eq_preds = stub_predictions(questions, label="eq")
        lr_preds = stub_predictions(questions, label="lr")
        reasons.append("dry_run")
        mark_phase(art_stage, "query", status="done", detail="dry_run")
    else:
        base = api or api_base()
        run_eq = not lr_only
        run_lr = not eq_only

        # Fail closed on Acc pin drift (ollama vision / wrong embed) before spend.
        try:
            probe = EdgeQuakeClient(base, workspace="bench001-pin-check")
            health = probe.health()
            bad = backend_pin_mismatches(health)
            if bad:
                msg = "backend Acc pin mismatch: " + "; ".join(bad)
                print(f"ERROR: {msg}", flush=True)
                print(
                    "Fix: make bench001-acc-backend  "
                    "(or unset BENCH001_SKIP_BACKEND_RESTART)",
                    flush=True,
                )
                if os.environ.get("BENCH001_PUBLICATION", "").strip() in {"1", "true", "yes"}:
                    raise RuntimeError(msg)
                print("WARN: continuing despite pin mismatch (non-publication)", flush=True)
        except RuntimeError:
            raise
        except Exception as exc:  # noqa: BLE001
            print(f"WARN: could not verify backend Acc pins: {exc}", flush=True)

        def _eq_job() -> tuple[list[dict[str, Any]], float, str | None]:
            ws_env = os.environ.get("BENCH001_EQ_WORKSPACE_ID")
            client = EdgeQuakeClient(
                base,
                workspace=eq_workspace_name,
                workspace_id=ws_env,
            )
            # Fail closed / auto-heal: pinned workspace must exist for query-only.
            if query_only and ws_env and not client.workspace_exists(ws_env):
                raise RuntimeError(
                    f"BENCH001_EQ_WORKSPACE_ID={ws_env} not found on {base}. "
                    "Re-ingest without --query-only (unset the pin or let "
                    "ensure_workspace create a fresh workspace)."
                )
            if not query_only:
                client.ensure_workspace()
                print(f"EQ workspace={client.workspace_id}", flush=True)
            preds, wall = _run_eq(
                client=client,
                corpus_texts=corpus_texts,
                questions=questions,
                query_only=query_only,
                force_ingest=force_ingest,
                concurrency=q_conc,
            )
            return preds, wall, client.workspace_id

        def _lr_job() -> tuple[list[dict[str, Any]], str | None]:
            if not (lr_available() or lightrag_repo() is not None):
                return stub_predictions(questions, label="lr_unavailable"), "lr_unavailable"
            preds = run_lightrag(
                corpus_texts=corpus_texts,
                questions=questions,
                stage=lr_index_stage,
                mode="mix",
                force_ingest=force_ingest,
                query_only=query_only,
                concurrency=q_conc,
            )
            return preds, None

        def _reuse_preds(path: Path, *, label: str) -> list[dict[str, Any]]:
            if path.exists():
                try:
                    prev = json.loads(path.read_text(encoding="utf-8"))
                    if isinstance(prev, list) and prev and not prev[0].get("stub"):
                        print(f"Reusing existing {label} predictions ({len(prev)})")
                        return prev
                except Exception as exc:  # noqa: BLE001
                    print(f"Could not reuse {label} predictions: {exc}")
            return stub_predictions(questions, label="skipped")

        # Fair dual-SUT: query EQ ∥ LR when both are live (each fans out internally).
        if run_eq and run_lr:
            mark_phase(
                art_stage,
                "query_parallel",
                status="running",
                detail=f"EQ∥LR concurrency={q_conc}",
            )
            with ThreadPoolExecutor(max_workers=2) as pool:
                fut_eq = pool.submit(_eq_job)
                fut_lr = pool.submit(_lr_job)
                # Dual-SUT lag board: heartbeat until both futures complete.
                t_board = time.perf_counter()
                while True:
                    eq_done, lr_done = fut_eq.done(), fut_lr.done()
                    board_elapsed = time.perf_counter() - t_board
                    print(
                        f"  dual-SUT lag board "
                        f"EQ={'done' if eq_done else 'running'} "
                        f"LR={'done' if lr_done else 'running'} "
                        f"elapsed={format_duration(board_elapsed)} "
                        f"run={format_duration(run_elapsed_s())}",
                        flush=True,
                    )
                    mark_phase(
                        art_stage,
                        "query_parallel",
                        status="running",
                        detail=(
                            f"EQ={'done' if eq_done else 'running'} "
                            f"LR={'done' if lr_done else 'running'}"
                        ),
                        phase_elapsed_s=board_elapsed,
                        quiet=True,
                    )
                    if eq_done and lr_done:
                        break
                    time.sleep(15)
                try:
                    eq_preds, ingest_wall, eq_workspace_id = fut_eq.result()
                except Exception as exc:  # noqa: BLE001
                    print(f"EQ run failed: {exc}")
                    reasons.append(f"eq_failed:{exc}")
                    eq_preds = stub_predictions(questions, label="eq_fail")
                try:
                    lr_preds, lr_reason = fut_lr.result()
                    if lr_reason:
                        reasons.append(lr_reason)
                except Exception as exc:  # noqa: BLE001
                    print(f"LightRAG run failed: {exc}")
                    reasons.append(f"lr_failed:{exc}")
                    lr_preds = stub_predictions(questions, label="lr_fail")
            mark_phase(
                art_stage,
                "query_parallel",
                status="done",
                detail=f"eq={len(eq_preds)} lr={len(lr_preds)}",
                counts={
                    "eq_empty_context_rate": empty_context_rate(eq_preds),
                    "lr_empty_context_rate": empty_context_rate(lr_preds),
                },
            )
        elif run_eq:
            mark_phase(art_stage, "query_eq", status="running")
            try:
                eq_preds, ingest_wall, eq_workspace_id = _eq_job()
            except Exception as exc:  # noqa: BLE001
                print(f"EQ run failed: {exc}")
                reasons.append(f"eq_failed:{exc}")
                eq_preds = stub_predictions(questions, label="eq_fail")
            reasons.append("eq_only")
            lr_preds = _reuse_preds(art / "predictions_lr.json", label="LR")
            if lr_preds and not lr_preds[0].get("stub"):
                reasons = [r for r in reasons if r != "eq_only"]
            mark_phase(art_stage, "query_eq", status="done", detail=f"n={len(eq_preds)}")
        else:
            mark_phase(art_stage, "query_lr", status="running")
            reasons.append("lr_only")
            eq_preds = _reuse_preds(art / "predictions_eq.json", label="EQ")
            if eq_preds and not eq_preds[0].get("stub"):
                reasons = [r for r in reasons if r != "lr_only"]
            try:
                lr_preds, lr_reason = _lr_job()
                if lr_reason:
                    reasons.append(lr_reason)
            except Exception as exc:  # noqa: BLE001
                print(f"LightRAG run failed: {exc}")
                reasons.append(f"lr_failed:{exc}")
                lr_preds = stub_predictions(questions, label="lr_fail")
            mark_phase(art_stage, "query_lr", status="done", detail=f"n={len(lr_preds)}")

    (art / "predictions_eq.json").write_text(
        json.dumps(eq_preds, indent=2), encoding="utf-8"
    )
    (art / "predictions_lr.json").write_text(
        json.dumps(lr_preds, indent=2), encoding="utf-8"
    )

    prefer_official = not dry_run
    accept_proxy = accept_proxy_judge or dry_run
    mark_phase(
        art_stage,
        "score_parallel",
        status="running",
        detail=f"prefer_official={prefer_official} eval_concurrency={e_conc}",
    )
    t_score = time.perf_counter()
    with ThreadPoolExecutor(max_workers=2) as pool:
        fut_eq = pool.submit(
            score_predictions,
            eq_preds,
            eval_out=art / "eval_eq.json",
            prefer_official=prefer_official,
            accept_proxy=accept_proxy,
        )
        fut_lr = pool.submit(
            score_predictions,
            lr_preds,
            eval_out=art / "eval_lr.json",
            prefer_official=prefer_official,
            accept_proxy=accept_proxy,
        )
        while True:
            eq_done, lr_done = fut_eq.done(), fut_lr.done()
            score_elapsed = time.perf_counter() - t_score
            print(
                f"  score lag board "
                f"EQ={'done' if eq_done else 'running'} "
                f"LR={'done' if lr_done else 'running'} "
                f"elapsed={format_duration(score_elapsed)} "
                f"run={format_duration(run_elapsed_s())}",
                flush=True,
            )
            mark_phase(
                art_stage,
                "score_parallel",
                status="running",
                detail=(
                    f"EQ={'done' if eq_done else 'running'} "
                    f"LR={'done' if lr_done else 'running'} "
                    f"eval∥={e_conc}"
                ),
                phase_elapsed_s=score_elapsed,
                quiet=True,
            )
            if eq_done and lr_done:
                break
            time.sleep(15)
        eq_metrics, eq_official = fut_eq.result()
        lr_metrics, lr_official = fut_lr.result()
    mark_phase(
        art_stage,
        "score_parallel",
        status="done",
        detail=f"elapsed={format_duration(time.perf_counter() - t_score)}",
    )

    judge = "generation_eval" if (eq_official and lr_official) else eq_metrics.get("judge", "rouge_proxy")
    if judge != "generation_eval":
        reasons.append(f"judge:{judge}")

    from .fair_pins import (
        PUBLISH_EMPTY_ANSWER_MAX,
        PUBLISH_EMPTY_CONTEXT_MAX,
        publish_fairness_enabled,
        retrieve_topk,
    )
    from .scorecard import empty_rate

    ans_max = PUBLISH_EMPTY_ANSWER_MAX if publish_fairness_enabled() else 0.10
    ctx_max = PUBLISH_EMPTY_CONTEXT_MAX if publish_fairness_enabled() else 0.10

    # Validity: dual live SUTs + official judge + empty-answer/context gates
    valid = not reasons
    if (
        valid
        and not dry_run
        and (empty_rate(eq_preds) > ans_max or empty_rate(lr_preds) > ans_max)
    ):
        reasons.append("empty_answer_rate")
        valid = False
    # Empty retrieved context invalidates Creative Faithfulness (official metric).
    if (
        valid
        and not dry_run
        and (
            empty_context_rate(eq_preds) > ctx_max
            or empty_context_rate(lr_preds) > ctx_max
        )
    ):
        reasons.append("empty_context_rate")
        valid = False

    # Publishable runs require L2 retrieval metrics (2026 RAG Triad practice).
    if valid and not dry_run and publish_fairness_enabled() and stage in {
        "smoke",
        "smoke-fast",
        "medical-mid",
        "medical-full",
        "core",
    }:
        eq_l2 = bool(eq_metrics.get("l2_retrieval"))
        lr_l2 = bool(lr_metrics.get("l2_retrieval"))
        if not (eq_l2 and lr_l2):
            reasons.append("l2_retrieval_missing")
            valid = False
        # Acc components (F1 + cos) required for publishable Acc claims (P15).
        from .acc_stats import components_present

        if not (components_present(eq_metrics) and components_present(lr_metrics)):
            reasons.append("acc_components_missing")
            valid = False

    if max_questions is not None:
        reasons.append("max_questions_truncate")
        valid = False

    invalid_reason = ";".join(reasons) if reasons else None
    scorecard = build_scorecard(
        stage=art_stage,
        fixture_id=fixture_id,
        eq_metrics=eq_metrics,
        lr_metrics=lr_metrics,
        eq_preds=eq_preds,
        lr_preds=lr_preds,
        valid=valid,
        invalid_reason=invalid_reason,
        judge=str(judge),
        ingest_wall_s=ingest_wall,
        provider_pins=pins,
        retrieve_topk=retrieve_topk(),
    )
    (art / "scorecard.json").write_text(json.dumps(scorecard, indent=2), encoding="utf-8")
    write_summary(scorecard, art / "SUMMARY.md")
    # First principles: never overwrite Acc *global* warm pointer on invalid /
    # empty-context runs (032 B3b: disk-full saga rollback left WS empty).
    # Also skip global warm on labeled peers (SKIP_PUBLISH_LATEST / PUBLISH_PEER)
    # so gap-close / ingest ablations cannot hijack Acc B5 warm (080 D4).
    # Still write per-stage eq_workspace.json for audit/forensics.
    persist_warm = bool(eq_workspace_id) and (not dry_run) and bool(valid)
    _skip_latest = (os.environ.get("BENCH001_SKIP_PUBLISH_LATEST") or "").strip().lower() in {
        "1",
        "true",
        "yes",
        "on",
    }
    _labeled_peer = bool((os.environ.get("BENCH001_PUBLISH_PEER") or "").strip())
    update_global_warm = persist_warm and not _skip_latest and not _labeled_peer
    if eq_workspace_id and not dry_run:
        meta["eq_workspace_id"] = eq_workspace_id
        (art / "meta.json").write_text(json.dumps(meta, indent=2), encoding="utf-8")
        eq_blob = {
            "workspace_id": eq_workspace_id,
            "full_corpus": not bool(ingest_meta.get("ingest_capped")),
            "eq_workspace_name": eq_workspace_name,
            "valid": bool(valid),
            "invalid_reason": invalid_reason,
        }
        (art / "eq_workspace.json").write_text(
            json.dumps(eq_blob, indent=2) + "\n", encoding="utf-8"
        )
        if update_global_warm:
            from .warm_workspace import persist_warm_workspace

            persist_warm_workspace(
                eq_workspace_id,
                stage_dir=art,
                full_corpus=not bool(ingest_meta.get("ingest_capped")),
                meta_extra={
                    "eq_workspace_name": eq_workspace_name,
                    "stage": art_stage,
                    "publication": os.environ.get("BENCH001_PUBLICATION", ""),
                },
            )
            print(f"EQ warm workspace pointer: {eq_workspace_id}", flush=True)
        elif persist_warm:
            print(
                f"EQ warm workspace NOT updated (labeled peer / skip-latest); "
                f"keeping Acc warm pointer (this run ws={eq_workspace_id})",
                flush=True,
            )
        else:
            print(
                f"EQ warm workspace NOT updated (valid=False reason={invalid_reason}); "
                f"keeping prior pointer (this run ws={eq_workspace_id})",
                flush=True,
            )
    hist = archive_run(art_stage, scorecard)
    if update_global_warm:
        from .warm_workspace import persist_warm_workspace

        persist_warm_workspace(
            eq_workspace_id,
            stage_dir=art,
            archive_dir=hist,
            full_corpus=not bool(ingest_meta.get("ingest_capped")),
            meta_extra={
                "eq_workspace_name": eq_workspace_name,
                "stage": art_stage,
            },
        )
    elif eq_workspace_id and hist is not None:
        eq_blob = {
            "workspace_id": eq_workspace_id,
            "full_corpus": not bool(ingest_meta.get("ingest_capped")),
            "eq_workspace_name": eq_workspace_name,
            "valid": bool(valid),
            "invalid_reason": invalid_reason,
        }
        (hist / "eq_workspace.json").write_text(
            json.dumps(eq_blob, indent=2) + "\n", encoding="utf-8"
        )
    from .business_report import print_business_verdict, write_publish_pack

    latest = write_publish_pack(scorecard, stage_dir=art, archive_dir=hist)
    mark_phase(
        art_stage,
        "report",
        status="done",
        detail=f"valid={valid} archive={hist.name}",
        counts={
            "eq_acc": scorecard["metrics"]["eq"]["overall_acc"],
            "lr_acc": scorecard["metrics"]["lr"]["overall_acc"],
        },
    )
    print(f"Wrote {art / 'SUMMARY.md'}")
    print(f"Progress ledger: {write_progress_md()}")
    print(f"Archived run: {hist}")
    print(f"Publish pack: {latest}")
    print(f"valid={valid} reason={invalid_reason}")
    eq_m = scorecard["metrics"]["eq"]
    lr_m = scorecard["metrics"]["lr"]
    print(
        f"EQ Acc={eq_m['overall_acc']:.4f} F1={eq_m.get('overall_f1')} "
        f"cos={eq_m.get('overall_cos')} | "
        f"LR Acc={lr_m['overall_acc']:.4f} F1={lr_m.get('overall_f1')} "
        f"cos={lr_m.get('overall_cos')} | "
        f"Δ={scorecard['metrics']['delta_eq_minus_lr']['overall_acc']:+.4f}"
    )
    print_business_verdict(scorecard)
    return 0 if (valid or dry_run) else 1


def rescore_stage(
    stage: str = "smoke",
    *,
    source_stage: str | None = None,
    eval_concurrency_n: int | None = None,
    llm_provider: str | None = None,
    llm_model: str | None = None,
    vision_provider: str | None = None,
    vision_model: str | None = None,
    embedding_provider: str | None = None,
    embedding_model: str | None = None,
    embedding_dim: int | None = None,
    llm_base_url: str | None = None,
    judge_provider: str | None = None,
    judge_model: str | None = None,
    judge_base_url: str | None = None,
    judge_embedding_model: str | None = None,
    profile_id: str | None = None,
    accept_proxy_judge: bool = False,
) -> int:
    """Re-run judgment on frozen predictions (Acc is post-hoc)."""
    from .fair_pins import publish_fairness_enabled, retrieve_topk
    from .profiles import resolve_pins, set_active_pins
    from .acc_stats import components_present

    pins = resolve_pins(
        llm_provider=llm_provider,
        llm_model=llm_model,
        vision_provider=vision_provider,
        vision_model=vision_model,
        embedding_provider=embedding_provider,
        embedding_model=embedding_model,
        embedding_dim=embedding_dim,
        llm_base_url=llm_base_url,
        judge_provider=judge_provider,
        judge_model=judge_model,
        judge_base_url=judge_base_url,
        judge_embedding_model=judge_embedding_model,
        profile_id=profile_id,
    )
    set_active_pins(pins)
    src = source_stage or stage
    # Paper / rescore writes to a distinct artifact dir when profile differs.
    art_stage = stage
    if pins.profile_id and "paper" in pins.profile_id.lower():
        art_stage = f"{src}-paper"
    elif stage.endswith("-rescore") or stage != src:
        art_stage = stage
    else:
        art_stage = f"{src}-rescore"

    src_dir = stage_artifact_dir(src)
    eq_path = src_dir / "predictions_eq.json"
    lr_path = src_dir / "predictions_lr.json"
    if not eq_path.exists() or not lr_path.exists():
        print(f"missing predictions in {src_dir}")
        return 2
    eq_preds = json.loads(eq_path.read_text(encoding="utf-8"))
    lr_preds = json.loads(lr_path.read_text(encoding="utf-8"))
    meta_src = src_dir / "meta.json"
    fixture_id = "unknown"
    if meta_src.exists():
        fixture_id = json.loads(meta_src.read_text(encoding="utf-8")).get(
            "fixture_id", fixture_id
        )

    art = stage_artifact_dir(art_stage)
    (art / "predictions_eq.json").write_text(
        json.dumps(eq_preds, indent=2), encoding="utf-8"
    )
    (art / "predictions_lr.json").write_text(
        json.dumps(lr_preds, indent=2), encoding="utf-8"
    )
    e_conc = eval_concurrency_pin(eval_concurrency_n)
    if eval_concurrency_n is not None:
        os.environ["BENCH001_EVAL_CONCURRENCY"] = str(e_conc)
    print(
        f"rescore source={src} → {art_stage} profile={pins.profile_id} "
        f"eval_concurrency={e_conc}",
        flush=True,
    )
    prefer_official = True
    accept_proxy = accept_proxy_judge
    with ThreadPoolExecutor(max_workers=2) as pool:
        fut_eq = pool.submit(
            score_predictions,
            eq_preds,
            eval_out=art / "eval_eq.json",
            prefer_official=prefer_official,
            accept_proxy=accept_proxy,
        )
        fut_lr = pool.submit(
            score_predictions,
            lr_preds,
            eval_out=art / "eval_lr.json",
            prefer_official=prefer_official,
            accept_proxy=accept_proxy,
        )
        eq_metrics, eq_official = fut_eq.result()
        lr_metrics, lr_official = fut_lr.result()

    reasons: list[str] = []
    judge = (
        "generation_eval"
        if (eq_official and lr_official)
        else eq_metrics.get("judge", "rouge_proxy")
    )
    if judge != "generation_eval":
        reasons.append(f"judge:{judge}")
    if publish_fairness_enabled():
        if not (eq_metrics.get("l2_retrieval") and lr_metrics.get("l2_retrieval")):
            reasons.append("l2_retrieval_missing")
        if not (components_present(eq_metrics) and components_present(lr_metrics)):
            reasons.append("acc_components_missing")
    valid = not reasons
    scorecard = build_scorecard(
        stage=art_stage,
        fixture_id=fixture_id,
        eq_metrics=eq_metrics,
        lr_metrics=lr_metrics,
        eq_preds=eq_preds,
        lr_preds=lr_preds,
        valid=valid,
        invalid_reason=";".join(reasons) if reasons else None,
        judge=str(judge),
        provider_pins=pins,
        retrieve_topk=retrieve_topk(),
    )
    (art / "scorecard.json").write_text(json.dumps(scorecard, indent=2), encoding="utf-8")
    write_summary(scorecard, art / "SUMMARY.md")
    hist = archive_run(art_stage, scorecard)
    from .business_report import print_business_verdict, write_publish_pack

    latest = write_publish_pack(scorecard, stage_dir=art, archive_dir=hist)
    print(f"Wrote {art / 'SUMMARY.md'}")
    print(f"Archived run: {hist}")
    print(f"Publish pack: {latest}")
    print(f"valid={valid} reason={scorecard.get('invalid_reason')}")
    print(
        f"EQ Acc={scorecard['metrics']['eq']['overall_acc']:.4f} "
        f"F1={scorecard['metrics']['eq'].get('overall_f1')} "
        f"cos={scorecard['metrics']['eq'].get('overall_cos')}"
    )
    print(
        f"LR Acc={scorecard['metrics']['lr']['overall_acc']:.4f} "
        f"F1={scorecard['metrics']['lr'].get('overall_f1')} "
        f"cos={scorecard['metrics']['lr'].get('overall_cos')}"
    )
    print_business_verdict(scorecard)
    return 0 if valid else 1


def run_acc_canary_cmd(*, eval_concurrency_n: int | None = None) -> int:
    from .acc_canary import run_acc_canary

    report = run_acc_canary(eval_concurrency=eval_concurrency_n)
    print(f"acc-canary passed={report['passed']} failures={report.get('failures')}")
    print(f"SUMMARY: {stage_artifact_dir('acc-canary') / 'SUMMARY.md'}")
    return 0 if report["passed"] else 1


def report(stage_or_path: str, *, compare: str | None = None) -> int:
    path = Path(stage_or_path)
    if not path.exists():
        path = stage_artifact_dir(stage_or_path)
    summary = path / "SUMMARY.md"
    scorecard = path / "scorecard.json"
    if summary.exists():
        print(summary.read_text(encoding="utf-8"))
    elif scorecard.exists():
        print(scorecard.read_text(encoding="utf-8"))
    else:
        print(f"no SUMMARY/scorecard in {path}")
        return 1
    if compare:
        other = Path(compare)
        if not other.exists():
            # Allow history/<run> relative to artifacts.
            from .paths import ARTIFACTS_DIR

            cand = ARTIFACTS_DIR / compare
            if cand.exists():
                other = cand
            else:
                other = stage_artifact_dir(compare)
        a = json.loads((path / "scorecard.json").read_text(encoding="utf-8"))
        b = json.loads((other / "scorecard.json").read_text(encoding="utf-8"))
        da = a["metrics"]["eq"]["overall_acc"] - b["metrics"]["eq"]["overall_acc"]
        dl = a["metrics"]["lr"]["overall_acc"] - b["metrics"]["lr"]["overall_acc"]
        print(f"\nCompare vs {other}:")
        print(f"  EQ Acc Δ: {da:+.4f}")
        print(f"  LR Acc Δ: {dl:+.4f}")
    prog = write_progress_md()
    if prog.exists():
        print(f"\n--- progression ({prog}) ---")
        print(prog.read_text(encoding="utf-8"))
    return 0
