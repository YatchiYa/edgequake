"""End-to-end stage runner: ingest → query → extract → score."""

from __future__ import annotations

import json
import os
import statistics
import subprocess
import threading
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Any, Optional

from .client import EdgeQuakeClient
from .diagnostics import (
    aggregate_arm_gate_metrics,
    aggregate_false_refusal_metrics,
    aggregate_page_hit_metrics,
    build_retrieval_diagnostics,
)
from .download import download_pdfs, ensure_qa
from .extract import extract_answer
from .paths import documents_dir, stage_artifact_dir
from .profiles import BenchProfile, get_profile
from .score import append_jsonl, build_scorecard, load_jsonl, score_sample, write_summary
from .subset import load_qa_df, questions_for_docs, read_doc_ids


FIXTURE_FOR_STAGE = {
    "smoke": "smoke_doc_ids_v1.txt",
    "core": "core_doc_ids_v1.txt",
    "full": None,  # all docs
}


def _fixture_name_for_stage(stage: str) -> Optional[str]:
    """Allow EDGEQUAKE_BENCH_FIXTURE override (e.g. chart-subset for G-A)."""
    override = (os.environ.get("EDGEQUAKE_BENCH_FIXTURE") or "").strip()
    if override:
        return override
    return FIXTURE_FOR_STAGE.get(stage)


def _git_sha() -> str:
    try:
        return (
            subprocess.check_output(["git", "rev-parse", "--short=8", "HEAD"], cwd=Path(__file__).parents[3])
            .decode()
            .strip()
        )
    except Exception:
        return "unknown"


def doctor(base_url: Optional[str] = None, profile: Optional[BenchProfile] = None) -> int:
    profile = profile or get_profile()
    client = EdgeQuakeClient(base_url=base_url)
    h = client.health()
    print(json.dumps(h, indent=2)[:2000])
    providers = h.get("providers") or {}
    llm = (providers.get("llm") or {}).get("name") or h.get("llm_provider_name")
    emb = providers.get("embedding") or {}
    emb_name = emb.get("name")
    emb_dim = emb.get("dimension")
    emb_model = emb.get("model")
    ok = True
    storage = (h.get("storage_mode") or "").lower()
    if storage not in {"postgresql", "postgres"}:
        print(f"FAIL: storage_mode={storage!r} expected postgresql (SPEC-047 requires Postgres)")
        ok = False
    else:
        print(f"OK: storage_mode={storage}")
    if llm != profile.llm_provider:
        print(f"FAIL: llm provider={llm} expected={profile.llm_provider}")
        ok = False
    if emb_name != profile.embedding_provider:
        print(f"FAIL: embedding provider={emb_name} expected={profile.embedding_provider}")
        ok = False
    if emb_dim and int(emb_dim) != profile.embedding_dim:
        print(f"FAIL: embedding dim={emb_dim} expected={profile.embedding_dim}")
        ok = False
    if emb_model and profile.embedding_model not in str(emb_model):
        print(f"WARN: embedding model={emb_model} expected contains {profile.embedding_model}")
    llm_model = (providers.get("llm") or {}).get("model")
    if llm_model and profile.llm_model not in str(llm_model) and "mistral" in str(llm_model):
        print(
            f"WARN: server llm model={llm_model} (profile wants {profile.llm_model}); "
            "workspace create will pin the profile model for this run"
        )
    # vision capability: models.toml must mark small as vision; env must match
    vision_model = os.environ.get("EDGEQUAKE_VISION_MODEL", profile.vision_model)
    if vision_model != profile.vision_model:
        print(f"WARN: EDGEQUAKE_VISION_MODEL={vision_model} profile={profile.vision_model}")
    if profile.pdf_parser_backend == "vision":
        print(f"OK: vision model intended={profile.vision_model} (provider={profile.vision_provider})")
    if profile.process_options:
        print(f"OK: process_options={profile.process_options}")
    if profile.requires_vlm_process:
        vlm = (os.environ.get("VLM_PROCESS_ENABLE") or "").strip().lower()
        if vlm in {"", "0", "false", "no", "off"}:
            print(
                "FAIL: profile requires VLM_PROCESS_ENABLE=true on the API host "
                f"(got {os.environ.get('VLM_PROCESS_ENABLE')!r})"
            )
            ok = False
        else:
            print(f"OK: VLM_PROCESS_ENABLE={os.environ.get('VLM_PROCESS_ENABLE')}")
    if not os.environ.get("MISTRAL_API_KEY"):
        print("FAIL: MISTRAL_API_KEY unset")
        ok = False
    if profile.extractor.startswith("gpt") and not os.environ.get("OPENAI_API_KEY"):
        print("FAIL: OPENAI_API_KEY unset (official extractor)")
        ok = False
    if profile.extractor.startswith("mistral") and not os.environ.get("MISTRAL_API_KEY"):
        print("FAIL: MISTRAL_API_KEY unset (mistral extractor)")
        ok = False
    print("doctor:", "PASS" if ok else "FAIL")
    return 0 if ok else 2


def run_stage(
    stage: str,
    *,
    profile_name: str = "P0_primary",
    base_url: Optional[str] = None,
    resume: bool = True,
    accept_cost: bool = False,
    max_docs: Optional[int] = None,
    max_questions: Optional[int] = None,
    ingest_only: bool = False,
    query_only: bool = False,
    document_scope: bool = False,
    workers: int = 1,
) -> int:
    profile = get_profile(profile_name)
    if stage in {"core", "full"} and not accept_cost:
        print("Refusing core/full without --i-accept-cost (vision ingest is expensive).")
        return 3

    ensure_qa()
    df = load_qa_df()
    if stage == "full":
        doc_ids = sorted(df["doc_id"].unique().tolist())
        fixture_id = "full"
    else:
        fixture = _fixture_name_for_stage(stage)
        if not fixture:
            raise SystemExit(f"No fixture configured for stage={stage}")
        doc_ids = read_doc_ids(fixture)
        fixture_id = fixture.replace(".txt", "")
    if max_docs is not None:
        doc_ids = doc_ids[:max_docs]

    art = stage_artifact_dir(stage)
    ingest_path = art / "ingest.jsonl"
    pred_path = art / "predictions.jsonl"
    meta_path = art / "meta.json"

    client = EdgeQuakeClient(base_url=base_url)
    health = client.health()
    # doctor soft-check
    if doctor(base_url=base_url, profile=profile) != 0:
        scorecard = build_scorecard(
            stage=stage,
            profile=profile,
            samples=[],
            pins_extra={"fixture_id": fixture_id, "edgequake_git_sha": _git_sha(),
                        "edgequake_version": health.get("version", "unknown")},
            ops={"n_docs": 0, "n_questions": 0, "ingest_coverage": 0.0, "n_skipped_ingest_failed": 0},
            valid=False,
            invalid_reason="INVALID_VISION_CONFIG_OR_PROVIDER",
        )
        (art / "scorecard.json").write_text(json.dumps(scorecard, indent=2))
        write_summary(scorecard, art / "SUMMARY.md")
        return 2

    # Fresh workspace every run (slug collision reused Large LLM workspace previously).
    # On --resume, reuse workspace from meta so partial ingest can continue in-place.
    ws_slug = f"bench047-{stage}-{_git_sha()}-{profile.profile_id.lower()}-{int(time.time())}"
    if not query_only:
        resumed = False
        if resume and meta_path.exists():
            meta = json.loads(meta_path.read_text())
            wid = meta.get("workspace_id")
            if wid and meta.get("profile") == profile.profile_id:
                client.workspace_id = wid
                ws_slug = meta.get("workspace_slug") or ws_slug
                print("resume workspace:", client.workspace_id, "slug=", ws_slug)
                resumed = True
        if not resumed:
            ws = client.create_workspace(
                name=f"SPEC-047 {stage} {profile.profile_id}",
                slug=ws_slug,
                llm_provider=profile.llm_provider,
                llm_model=profile.llm_model,
                embedding_provider=profile.embedding_provider,
                embedding_model=profile.embedding_model,
                embedding_dimension=profile.embedding_dim,
                vision_llm_provider=profile.vision_provider,
                vision_llm_model=profile.vision_model,
            )
            pinned_llm = f"{ws.get('llm_provider')}/{ws.get('llm_model')}"
            pinned_vis = f"{ws.get('vision_llm_provider')}/{ws.get('vision_llm_model')}"
            print("workspace:", client.workspace_id, "slug=", ws_slug)
            print("  pinned llm=", pinned_llm, "vision=", pinned_vis, "storage=postgresql")
            if profile.llm_model not in str(ws.get("llm_model") or ""):
                raise RuntimeError(
                    f"Workspace LLM pin mismatch: got {ws.get('llm_model')!r} want {profile.llm_model!r}"
                )
            if profile.vision_model not in str(ws.get("vision_llm_model") or ""):
                raise RuntimeError(
                    f"Workspace vision pin mismatch: got {ws.get('vision_llm_model')!r} "
                    f"want {profile.vision_model!r}"
                )
    elif not client.workspace_id:
        # recover from meta
        if meta_path.exists():
            meta = json.loads(meta_path.read_text())
            client.workspace_id = meta.get("workspace_id")
            ws_slug = meta.get("workspace_slug") or ws_slug

    run_workspace_id = client.workspace_id
    if not run_workspace_id:
        raise RuntimeError("No workspace_id (create workspace or pass --query-only with meta.json)")

    # download PDFs for selected docs
    pdf_paths = {p.name: p for p in download_pdfs(doc_ids)}
    # also check documents_dir
    for d in doc_ids:
        alt = documents_dir() / d
        if alt.exists():
            pdf_paths[d] = alt

    # Always load ingest ledger for query-only / resume (document_id map is required).
    done_ingest = {r["doc_id"]: r for r in load_jsonl(ingest_path)}
    if not resume and not query_only:
        done_ingest = {}
    completed_docs: dict[str, Any] = {
        k: v for k, v in done_ingest.items() if (v.get("status") or "").lower() == "completed"
    }
    failed_docs: dict[str, Any] = {
        k: v for k, v in done_ingest.items() if (v.get("status") or "").lower() == "failed"
    }

    if not query_only:
        for i, doc_id in enumerate(doc_ids, 1):
            if resume and doc_id in completed_docs:
                print(f"[{i}/{len(doc_ids)}] skip ingest {doc_id}")
                continue
            path = pdf_paths.get(doc_id)
            if not path or not path.exists():
                row = {"doc_id": doc_id, "status": "failed", "error": "pdf_missing"}
                append_jsonl(ingest_path, row)
                failed_docs[doc_id] = row
                continue
            # First principles: force_reindex wipes markdown + checkpoints and re-runs
            # full vision+multimodal (117-page docs = hours). Only force on fresh runs
            # (--no-resume). On --resume, allow checkpoint / stored-markdown shortcut.
            force_reindex = not resume
            print(
                f"[{i}/{len(doc_ids)}] upload {doc_id} ({path.stat().st_size} bytes)"
                f" force_reindex={force_reindex}"
            )
            try:
                up = client.upload_pdf(
                    path,
                    enable_vision=profile.pdf_parser_backend == "vision",
                    vision_provider=profile.vision_provider,
                    vision_model=profile.vision_model,
                    pdf_parser_backend=profile.pdf_parser_backend,
                    force_reindex=force_reindex,
                    process_options=profile.process_options,
                )
                pdf_id = up.get("pdf_id") or up.get("id")
                # Resume + checksum hit returns "duplicate" without enqueueing. If the
                # PDF is still processing/failed after a crash, soft-recover (keeps
                # stored markdown — skips re-vision).
                up_status = (up.get("status") or "").lower()
                if resume and not force_reindex and up_status == "duplicate" and pdf_id:
                    st0 = client.pdf_status(pdf_id)
                    pdf_st = (st0.get("status") or "").lower()
                    eq_doc = st0.get("document_id")
                    if pdf_st in {"processing", "pending", "failed"}:
                        print(
                            f"  soft-reprocess pdf {pdf_id[:8]}… "
                            f"(status={pdf_st}; keep markdown, skip re-vision)"
                        )
                        if pdf_st == "failed" and eq_doc:
                            client.reprocess_failed(
                                document_id=str(eq_doc), mode="entities_only"
                            )
                        else:
                            rec = client.recover_stuck(stuck_threshold_minutes=0)
                            requeued = int(rec.get("requeued") or 0)
                            print(
                                f"  recover_stuck requeued={requeued} "
                                f"found={rec.get('stuck_found')}"
                            )
                            # If PDF row says processing but no worker is alive,
                            # recover may no-op; force soft reprocess of the doc.
                            if requeued == 0 and eq_doc:
                                print(
                                    f"  soft reprocess document {str(eq_doc)[:8]}… "
                                    "(entities_only force; no active worker)"
                                )
                                client.reprocess_failed(
                                    document_id=str(eq_doc),
                                    mode="entities_only",
                                    max_documents=1,
                                    force=True,
                                )
                st = client.wait_pdf(pdf_id, timeout_s=10800)
                status = (st.get("status") or "").lower()
                row = {
                    "doc_id": doc_id,
                    "pdf_id": pdf_id,
                    "document_id": st.get("document_id") or up.get("document_id"),
                    "status": "completed" if status in {"completed", "duplicate"} else status,
                    "raw_status": status,
                    "page_count": (st.get("metadata") or {}).get("page_count"),
                }
                append_jsonl(ingest_path, row)
                if row["status"] == "completed":
                    completed_docs[doc_id] = row
                else:
                    failed_docs[doc_id] = row
            except Exception as e:
                row = {"doc_id": doc_id, "status": "failed", "error": str(e)}
                append_jsonl(ingest_path, row)
                failed_docs[doc_id] = row
                print(f"  ERROR: {e}")

    if ingest_only:
        print("ingest-only done")
        return 0

    ok_docs = set(completed_docs)
    qdf = questions_for_docs(df, [d for d in doc_ids if d in ok_docs])
    if max_questions is not None:
        qdf = qdf.head(max_questions)

    done_pred = {}
    if resume:
        for r in load_jsonl(pred_path):
            key = f"{r.get('doc_id')}::{r.get('question')}"
            if "score" in r:
                done_pred[key] = r
    else:
        # Fresh query pass — avoid duplicate JSONL rows
        pred_path.write_text("")

    latencies: list[float] = []
    extract_fails = 0
    empty_answers = 0
    samples: list[dict[str, Any]] = list(done_pred.values())
    pred_lock = threading.Lock()
    stats_lock = threading.Lock()

    pending_rows: list[tuple[int, Any]] = []
    for idx, (_, row) in enumerate(qdf.iterrows(), 1):
        doc_id = row["doc_id"]
        question = row["question"]
        key = f"{doc_id}::{question}"
        if key in done_pred:
            print(f"[{idx}/{len(qdf)}] skip query")
            continue
        pending_rows.append((idx, row))

    def _run_one_query(item: tuple[int, Any]) -> dict[str, Any]:
        nonlocal empty_answers, extract_fails
        idx, row = item
        doc_id = row["doc_id"]
        question = row["question"]
        worker = EdgeQuakeClient(base_url=base_url, workspace_id=run_workspace_id)
        print(f"[{idx}/{len(qdf)}] query {doc_id[:40]}… (worker)")
        t0 = time.time()
        query_error = None
        ingest_row = completed_docs.get(doc_id) or {}
        eq_document_id = ingest_row.get("document_id")
        scope_ids = [eq_document_id] if document_scope and eq_document_id else None
        try:
            resp = worker.query(
                question,
                mode=profile.query_mode,
                document_ids=scope_ids,
            )
            long_ans = resp.get("answer") or resp.get("response") or ""
            if not str(long_ans).strip() and isinstance(resp.get("error"), str):
                query_error = resp["error"]
        except Exception as e:
            long_ans = ""
            query_error = str(e)
            resp = {"error": query_error, "sources": [], "stats": {}}
        latency = (time.time() - t0) * 1000
        with stats_lock:
            latencies.append(latency)
        if not str(long_ans).strip():
            with stats_lock:
                empty_answers += 1
            if query_error:
                print(f"  query empty/error ({latency:.0f}ms): {query_error[:160]}")
            elif latency < 50:
                print(f"  WARN: empty answer in {latency:.1f}ms (backend likely down or wrong field)")
        try:
            short = extract_answer(question, long_ans, extractor=profile.extractor)
        except Exception as e:
            short = "Failed"
            with stats_lock:
                extract_fails += 1
            print(f"  extract fail: {e}")
        sc = score_sample(row["answer"], short, row["answer_format"])
        retrieval = build_retrieval_diagnostics(resp, evidence_pages=row["evidence_pages"])
        sample = {
            "doc_id": doc_id,
            "doc_type": row["doc_type"],
            "question": question,
            "answer": row["answer"],
            "pred": short,
            "answer_long": long_ans,
            "answer_format": row["answer_format"],
            "evidence_pages": row["evidence_pages"],
            "evidence_sources": row["evidence_sources"],
            "score": sc,
            "latency_ms": latency,
            "mode": profile.query_mode,
            "edgequake_document_id": eq_document_id,
            "document_scope": bool(scope_ids),
            "retrieval": retrieval,
        }
        if query_error:
            sample["query_error"] = query_error
        with pred_lock:
            append_jsonl(pred_path, sample)
        hit5 = retrieval.get("page_hit@5")
        print(
            f"  score={sc:.2f} pred={short[:80]!r} "
            f"page_hit@5={hit5} ctx_empty={retrieval.get('context_empty')}"
        )
        return sample

    if pending_rows:
        print(f"Query phase: {len(pending_rows)} questions, workers={workers}")
        if workers <= 1:
            for item in pending_rows:
                samples.append(_run_one_query(item))
        else:
            with ThreadPoolExecutor(max_workers=workers) as pool:
                futures = [pool.submit(_run_one_query, item) for item in pending_rows]
                for fut in as_completed(futures):
                    samples.append(fut.result())

    ingest_cov = (len(ok_docs) / len(doc_ids)) if doc_ids else 0.0
    empty_rate = (
        sum(1 for s in samples if not str(s.get("answer_long") or "").strip()) / max(1, len(samples))
    )
    retrieval_ops = aggregate_page_hit_metrics(samples, answerable_only=True)
    refusal_ops = aggregate_false_refusal_metrics(samples)
    arm_ops = aggregate_arm_gate_metrics(samples)
    ops = {
        "n_docs": len(doc_ids),
        "n_questions": len(qdf),
        "n_skipped_ingest_failed": len(doc_ids) - len(ok_docs),
        "ingest_coverage": ingest_cov,
        "p50_query_latency_ms": statistics.median(latencies) if latencies else None,
        "p95_query_latency_ms": (
            statistics.quantiles(latencies, n=20)[18] if len(latencies) >= 20 else (max(latencies) if latencies else None)
        ),
        "answer_empty_rate": empty_rate,
        "extractor_fail_rate": (extract_fails / max(1, len(samples))),
        "document_scope": document_scope,
        "page_hit_rate": retrieval_ops.get("page_hit@5"),
        "query_workers": workers,
        "retrieval": retrieval_ops,
        "false_refusal": refusal_ops,
        "arm_gates": arm_ops,
    }
    valid = ingest_cov >= (0.9 if stage == "smoke" else 0.8) and len(samples) > 0
    invalid_reason = None if valid else ("PARTIAL_INGEST" if ingest_cov < 0.9 else "NO_SAMPLES")
    # Fail closed: empty RAG answers inflate Acc via "Not answerable" extractor default
    if valid and empty_rate > 0.2:
        valid = False
        invalid_reason = "EMPTY_ANSWERS"
        print(f"INVALID: answer_empty_rate={empty_rate:.2f} > 0.20 (backend/query failure, not model quality)")
    if valid and ops["p50_query_latency_ms"] is not None and ops["p50_query_latency_ms"] < 20:
        valid = False
        invalid_reason = "QUERY_TOO_FAST"
        print(
            f"INVALID: p50_query_latency_ms={ops['p50_query_latency_ms']:.2f} "
            "(suspiciously fast — likely connection failures)"
        )

    scorecard = build_scorecard(
        stage=stage,
        profile=profile,
        samples=samples,
        pins_extra={
            "fixture_id": fixture_id,
            "edgequake_git_sha": _git_sha(),
            "edgequake_version": health.get("version", "unknown"),
        },
        ops=ops,
        valid=valid,
        invalid_reason=invalid_reason,
    )
    (art / "scorecard.json").write_text(json.dumps(scorecard, indent=2))
    write_summary(scorecard, art / "SUMMARY.md")
    meta_path.write_text(
        json.dumps(
            {
                "workspace_id": client.workspace_id,
                "workspace_slug": ws_slug,
                "stage": stage,
                "profile": profile.profile_id,
                "doc_ids": doc_ids,
                "api_base": client.base,
                "document_scope": document_scope,
                "query_workers": workers,
            },
            indent=2,
        )
    )
    print(f"Wrote {art / 'scorecard.json'}")
    print(f"Wrote {art / 'SUMMARY.md'}")
    print(f"Acc={scorecard['metrics']['accuracy']:.4f} F1={scorecard['metrics']['f1']:.4f} valid={valid}")
    if not valid:
        return 2
    # gate: smoke always returns 0 if valid (no F1 threshold yet)
    return 0
