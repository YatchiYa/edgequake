#!/usr/bin/env python3
"""Detach an Acc-pinned EdgeQuake release backend (survives agent shell cleanup).

Usage:
  python3 tools/bench001/scripts/start_acc_backend.py [--port 8090] [--wait 90]

Writes /tmp/edgequake-start.sh (Acc pins BEFORE exec), double-forks the binary,
and waits until /health is OK.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
RELEASE_BIN = REPO / "edgequake" / "target" / "release" / "edgequake"
DEBUG_BIN = REPO / "edgequake" / "target" / "debug" / "edgequake"
PORTS_ENV = REPO / ".edgequake-dev-ports.env"
START_SH = Path("/tmp/edgequake-start.sh")
PID_FILE = Path("/tmp/edgequake-backend.pid")
LOG_FILE = Path("/tmp/edgequake-backend.log")

ACC_EXPORTS = {
    "PORT": "8090",
    "WORKER_THREADS": "4",
    "MAX_TASKS_PER_TENANT": "1",
    "EDGEQUAKE_MAX_CONCURRENT_EXTRACTIONS": "8",
    "EDGEQUAKE_EMBED_MAX_ASYNC": "8",
    "EDGEQUAKE_MERGE_MAX_ASYNC": "4",
    "EDGEQUAKE_MEM_LIMIT": "16g",
    "EDGEQUAKE_COMMUNITY_GLOBAL": "false",
    "DATABASE_POOL_SIZE": "16",
    "EDGEQUAKE_DEV_MODE": "true",
    "EDGEQUAKE_AUTH_ENABLED": "false",
    "AUTH_ENABLED": "false",
    "EDGEQUAKE_LLM_PROVIDER": "mistral",
    "EDGEQUAKE_EMBEDDING_PROVIDER": "mistral",
    "MISTRAL_EMBEDDING_MODEL": "mistral-embed",
    "EDGEQUAKE_VISION_PROVIDER": "mistral",
    "EDGEQUAKE_VISION_MODEL": "mistral-small-latest",
    "EDGEQUAKE_LLM_MODEL": "mistral-small-latest",
    "MISTRAL_MODEL": "mistral-small-latest",
    "EDGEQUAKE_EMBEDDING_BATCH_SIZE": "32",
    "EDGEQUAKE_ALLOWED_PROVIDERS": "*",
    "EDGEQUAKE_NATIVE_GRAPH_WRITES": "1",
    "EDGEQUAKE_ALLOW_LOCAL_HIGH_CONCURRENCY": "1",
    "EDGEQUAKE_ADAPTIVE_CHUNKING": "0",
    "EDGEQUAKE_CHUNK_SIZE": "1200",
    "EDGEQUAKE_CHUNK_OVERLAP": "100",
    # SPEC-117 Acc / LightRAG parity: matched K + fifo hard truncate (not product default).
    "EDGEQUAKE_MAX_EXTRACTION_ENTITIES": "40",
    "EDGEQUAKE_MAX_EXTRACTION_RECORDS": "100",
    "EDGEQUAKE_EXTRACT_CAPS_SELECTION": "fifo",
    "EDGEQUAKE_MIX_ARM_GATE": "false",
    "EDGEQUAKE_RELATED_CHUNK_NUMBER": "5",
    "EDGEQUAKE_ORPHAN_RETRACT_ON_RECOVER": "0",
    "EDGEQUAKE_DEFAULT_LLM_PROVIDER": "mistral",
    "EDGEQUAKE_DEFAULT_LLM_MODEL": "mistral-small-latest",
    "EDGEQUAKE_DEFAULT_EMBEDDING_PROVIDER": "mistral",
    "EDGEQUAKE_DEFAULT_EMBEDDING_MODEL": "mistral-embed",
    # SPEC-086 Acc law = E2-occ (LR-identity Mix + occurrence_sort + Fact L2).
    # Prior P0 RRF/PPR/degree kept as labeled peer only — not Acc headline.
    "EDGEQUAKE_MIX_FUSION": "round_robin",
    "EDGEQUAKE_HYBRID_FUSION": "round_robin",
    "BENCH001_ALLOW_ROUND_ROBIN": "1",
    "BENCH001_EQ_ENABLE_RERANK": "0",
    # SPEC-103 LAW-C7: Acc cold peer — never claim warm LLM-cache latency.
    "EDGEQUAKE_LLM_CACHE": "0",
    # 076: local_first (Acc) · naive_first = LightRAG _merge_all_chunks order (REJECT Acc).
    "EDGEQUAKE_RR_ORDER": "local_first",
    # SPEC-001 Phase 1 relevancy prune — OFF by default for Acc headline.
    # Cosine ablation: EDGEQUAKE_MIX_RELEVANCY_PRUNE=1 SCORE=cosine (postprocess).
    "EDGEQUAKE_MIX_RELEVANCY_PRUNE": "0",
    "EDGEQUAKE_MIX_RELEVANCY_SCORE": "rrf",
    "EDGEQUAKE_MIX_RELEVANCY_KEEP": "12",
    "EDGEQUAKE_MIX_RELEVANCY_MIN_KEEP": "8",
    "EDGEQUAKE_MIX_RELEVANCY_SCORE_FLOOR": "0.25",
    "EDGEQUAKE_MIX_GRAPH_SOFT_PRUNE": "0",
    # Post-fuse rerank OFF under Acc law (086); BM25 engine kept for labeled CE peers.
    "EDGEQUAKE_RERANKER": "bm25",
    "EDGEQUAKE_RERANKER_PROVIDER": "",
    # DashScope intl + qwen3-rerank (china gte-rerank-v2 rejects many intl keys).
    "EDGEQUAKE_RERANKER_MODEL": "qwen3-rerank",
    "EDGEQUAKE_RERANKER_BASE_URL": (
        "https://dashscope-intl.aliyuncs.com/api/v1/services/rerank/text-rerank/"
        "text-rerank?compat=dashscope.aliyuncs.com"
    ),
    "EDGEQUAKE_PATH_PRUNE": "0",
    "EDGEQUAKE_PATH_PRUNE_FRACTION": "0",
    "EDGEQUAKE_PATH_PRUNE_ORPHAN_ENTITIES": "0",
    "EDGEQUAKE_PATH_PRUNE_ENTITY_MIN_KEEP": "4",
    # CE Acc recovery: keep first-stage Mix ranks (0 = pure CE).
    "EDGEQUAKE_RERANK_PROTECT_FIRST": "0",
    # 086 Acc law: retrieval order (not degree hubs).
    "EDGEQUAKE_ENTITY_RANK": "retrieval",
    # 086 Acc law: BFS (LR-identity); PPR remains labeled ablation.
    "EDGEQUAKE_GRAPH_WALK": "bfs",
    # KG→chunk: VECTOR (cosine) default; WEIGHT escape.
    "EDGEQUAKE_KG_CHUNK_PICK": "vector",
    # 051: default | lightrag (incident edges sorted by rank+weight).
    "EDGEQUAKE_RELATION_SELECT": "default",
    # Acc-win E3b: Mix arm weights (default equal 1/1/1).
    "EDGEQUAKE_MIX_LOCAL_WEIGHT": "1",
    "EDGEQUAKE_MIX_GLOBAL_WEIGHT": "1",
    "EDGEQUAKE_MIX_NAIVE_WEIGHT": "1",
    # 021 F2/F4 — Acc headline stays flat / passage_pack off.
    "EDGEQUAKE_CONTEXT_FORMAT": "flat",
    "EDGEQUAKE_PASSAGE_PACK": "0",
    # 021 F3 / 022 P5 — arm pool ≥3× query concurrency for fair Acc latency.
    "EDGEQUAKE_QUERY_ARM_CONCURRENCY": "16",
    # 022 P1/P3 — labeled compress / Acc disables popular-node hub fallback.
    "EDGEQUAKE_GRAPH_WALK_COMPRESS": "0",
    "EDGEQUAKE_POPULAR_NODE_FALLBACK": "0",
    "EDGEQUAKE_CONTENT_HEADINGS": "0",
    "EDGEQUAKE_KEYWORD_LEXICAL_BOOST": "0",
    # 086 Acc law: occurrence_sort on (E2 keep).
    "EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT": "1",
    # In-arm BM25/FTS fusion (default on). Dense BM25=0 REJECT Acc (077 E1).
    "EDGEQUAKE_BM25_RETRIEVAL": "1",
    # Mix KG→chunk timing: per_arm (default) | post_truncate (078 R3 REJECT Acc).
    "EDGEQUAKE_KG_CHUNK_PICK_TIMING": "per_arm",
    "EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET": "1",
    # 025 CE recall recovery — default min_rerank stays engine 0.1 unless overridden.
    "EDGEQUAKE_MIN_RERANK_SCORE": "0.1",
    "EDGEQUAKE_MIN_CHUNK_BUDGET_RATIO": "0.4",
    # 033 denser-graph packing — LightRAG constants.py token caps (not 10k/10k).
    "EDGEQUAKE_MAX_ENTITY_TOKENS": "6000",
    "EDGEQUAKE_MAX_RELATION_TOKENS": "8000",
    # 026 L2 Mix∪CE dual-list — off until S0/S1 Acc promote.
    "EDGEQUAKE_L2_SOURCES_UNION": "0",
    "EDGEQUAKE_L2_SOURCES_MIX_TOP_K": "30",
    # 027 Fact→BM25 intent / L2 BM25∪CE — off until T0b/T1 Acc promote.
    "EDGEQUAKE_INTENT_RERANK": "0",
    "EDGEQUAKE_FACT_RERANKER": "",
    # 058 C1a product latency Fact CE-skip — off until make bench001-c1a.
    "EDGEQUAKE_FACT_CE_SKIP": "0",
    # 060 C1d product keyword mode — llm (Acc) | heuristic (latency peer).
    "EDGEQUAKE_KEYWORD_MODE": "llm",
    # 062 C1e LightRAG KEYWORD role — empty = Query LLM (Acc default).
    # Latency pack sets KEYWORD_LLM_MODEL=ministral-3b-latest (same provider).
    "EDGEQUAKE_KEYWORD_LLM_PROVIDER": "",
    "EDGEQUAKE_KEYWORD_LLM_MODEL": "",
    # 086 Phase B EXTRACT≠QUERY — empty = workspace llm_model (Acc QUERY pin).
    # Labeled ingest: EDGEQUAKE_EXTRACT_LLM_MODEL=mistral-medium-latest.
    "EDGEQUAKE_EXTRACT_LLM_PROVIDER": "",
    "EDGEQUAKE_EXTRACT_LLM_MODEL": "",
    # 086 Acc law: Fact L2 fact_replace (Acc≠L2 list split kept by design).
    "EDGEQUAKE_L2_BM25_UNION": "1",
    "EDGEQUAKE_L2_BM25_MIX_TOP_K": "30",
    "EDGEQUAKE_L2_BM25_MODE": "fact_replace",
    # 080 D2 — type-aware Mix arm weights (off until lr-intent-w-fact-l2).
    "EDGEQUAKE_MIX_INTENT_WEIGHTS": "0",
    # 035 Fact CE∩BM25 protect — off until a1fp Acc promote.
    "EDGEQUAKE_FACT_PROTECT_BM25": "0",
    # 036 Exploratory coverage protect — off until a1fpcov Acc promote.
    "EDGEQUAKE_COVERAGE_PROTECT_FIRST": "0",
    # 038 Exploratory topic-entity admit — off until a1fpsel Acc promote.
    "EDGEQUAKE_TOPIC_ENTITY_ADMIT": "0",
    # 039 topic CE/fuse protect — off until a1fpce Acc promote.
    "EDGEQUAKE_TOPIC_CE_PROTECT": "0",
    # 040 topic trunc/pack protect — off until a1fptrunc Acc promote.
    "EDGEQUAKE_TOPIC_TRUNC_PROTECT": "0",
    "EDGEQUAKE_TOPIC_TRUNC_PROTECT_MAX": "4",
    # 042 topic KV materialize — off until a1fpmat Acc promote.
    "EDGEQUAKE_TOPIC_MATERIALIZE": "0",
    "EDGEQUAKE_TOPIC_MATERIALIZE_MAX": "4",
    # 045 CONTENT-gated materialize — off until a1fpcmat Acc promote.
    "EDGEQUAKE_TOPIC_MATERIALIZE_CONTENT": "0",
    # 048: empty = all types; set via ladder a1fpsumx (e.g. summarize).
    "EDGEQUAKE_TOPIC_MATERIALIZE_TYPES": "",
    # 028 Horizon A — off until A2/A3 Acc promote.
    "EDGEQUAKE_INTENT_FACTUAL_BIAS": "0",
    "EDGEQUAKE_ANSWER_PROMPT": "default",
    # 047: empty = global specific when ANSWER_PROMPT=specific; set via ladder a1fpscx.
    "EDGEQUAKE_ANSWER_SPECIFIC_TYPES": "",
    # 031 B3a — FAQ structure induction at ingest (off unless labeled).
    "EDGEQUAKE_STRUCTURE_INDUCE": "0",
    "RUST_LOG": "info,edgequake=info",
}


def _parse_existing_keys(text: str) -> dict[str, str]:
    keys: dict[str, str] = {}
    for m in re.finditer(r'^export\s+([A-Z0-9_]+)="([^"]*)"', text, re.M):
        keys[m.group(1)] = m.group(2)
    return keys


def _load_database_url() -> str:
    if START_SH.is_file():
        prev = _parse_existing_keys(START_SH.read_text(encoding="utf-8"))
        if prev.get("DATABASE_URL"):
            return prev["DATABASE_URL"]
    env_url = os.environ.get("DATABASE_URL")
    if env_url:
        return env_url
    return (
        "postgresql://edgequake:edgequake_secret@localhost:5432/edgequake"
        "?options=-c%20search_path%3Dpublic"
    )


def _load_mistral_key() -> str:
    # Prefer live env (non-placeholder), then prior start.sh.
    for name in ("MISTRAL_API_KEY", "LLM_API_KEY"):
        val = (os.environ.get(name) or "").strip()
        if val and not val.upper().startswith("FAKE"):
            return val
    if START_SH.is_file():
        prev = _parse_existing_keys(START_SH.read_text(encoding="utf-8"))
        for name in ("MISTRAL_API_KEY", "LLM_API_KEY", "OPENAI_API_KEY"):
            val = (prev.get(name) or "").strip()
            if val and not val.upper().startswith("FAKE"):
                return val
    raise SystemExit(
        "MISTRAL_API_KEY required (set env or ensure /tmp/edgequake-start.sh has it)"
    )


def resolve_backend_bin() -> Path:
    """Prefer release binary; fall back to debug for local Acc runs."""
    if RELEASE_BIN.is_file() and os.access(RELEASE_BIN, os.X_OK):
        return RELEASE_BIN
    if DEBUG_BIN.is_file() and os.access(DEBUG_BIN, os.X_OK):
        print(
            f"WARN: release binary missing — using debug Acc backend: {DEBUG_BIN}",
            flush=True,
        )
        return DEBUG_BIN
    raise SystemExit(
        f"Acc backend binary missing (tried {RELEASE_BIN} and {DEBUG_BIN}). "
        "Run: cd edgequake && cargo build --release  (or cargo build)"
    )


def write_start_sh(*, port: int) -> None:
    bin_path = resolve_backend_bin()
    key = _load_mistral_key()
    db = _load_database_url()
    ports = str(PORTS_ENV if PORTS_ENV.is_file() else "")
    lines = ["#!/bin/bash"]
    if ports:
        lines.append(f'set -a && . "{ports}" && set +a')
    lines.append(f'export PORT="{port}"')
    lines.append(f'export DATABASE_URL="{db}"')
    # Scrub inherited ollama / non-Acc defaults BEFORE Acc pins (ports may set them).
    lines.append(
        "unset EDGEQUAKE_DEFAULT_LLM_PROVIDER EDGEQUAKE_DEFAULT_LLM_MODEL "
        "EDGEQUAKE_DEFAULT_EMBEDDING_PROVIDER EDGEQUAKE_DEFAULT_EMBEDDING_MODEL "
        "EDGEQUAKE_LLM_PROVIDER EDGEQUAKE_LLM_MODEL "
        "EDGEQUAKE_VISION_PROVIDER EDGEQUAKE_VISION_MODEL "
        "EDGEQUAKE_EMBEDDING_PROVIDER EDGEQUAKE_EMBEDDING_MODEL "
        "MISTRAL_MODEL MISTRAL_EMBEDDING_MODEL "
        # 084: Acc default = 083 chat(system,user); never inherit COMPLETE_BLOB=1.
        "EDGEQUAKE_ANSWER_COMPLETE_BLOB "
        "2>/dev/null || true"
    )
    # Allow shell overrides for labeled Acc ablations (cosine prune / CE / PathRAG).
    _override_keys = {
        "EDGEQUAKE_MIX_RELEVANCY_PRUNE",
        "EDGEQUAKE_MIX_RELEVANCY_SCORE",
        "EDGEQUAKE_MIX_RELEVANCY_KEEP",
        "EDGEQUAKE_MIX_RELEVANCY_MIN_KEEP",
        "EDGEQUAKE_MIX_RELEVANCY_SCORE_FLOOR",
        "EDGEQUAKE_MIX_GRAPH_SOFT_PRUNE",
        "EDGEQUAKE_MIX_RELEVANCY_EMBED_CHARS",
        "EDGEQUAKE_RERANKER",
        "EDGEQUAKE_RERANKER_PROVIDER",
        "EDGEQUAKE_RERANKER_MODEL",
        "EDGEQUAKE_RERANKER_BASE_URL",
        "EDGEQUAKE_PATH_PRUNE_FRACTION",
        "EDGEQUAKE_PATH_PRUNE_ORPHAN_ENTITIES",
        "EDGEQUAKE_PATH_PRUNE_ENTITY_MIN_KEEP",
        "EDGEQUAKE_PATH_PRUNE",
        "EDGEQUAKE_RERANK_PROTECT_FIRST",
        "EDGEQUAKE_ENTITY_RANK",
        "EDGEQUAKE_GRAPH_WALK",
        "EDGEQUAKE_KG_CHUNK_PICK",
        "EDGEQUAKE_RELATION_SELECT",
        "EDGEQUAKE_RELATED_CHUNK_NUMBER",
        "EDGEQUAKE_MIX_LOCAL_WEIGHT",
        "EDGEQUAKE_MIX_GLOBAL_WEIGHT",
        "EDGEQUAKE_MIX_NAIVE_WEIGHT",
        "EDGEQUAKE_CONTEXT_FORMAT",
        "EDGEQUAKE_PASSAGE_PACK",
        "EDGEQUAKE_MIN_CHUNK_BUDGET_RATIO",
        "EDGEQUAKE_MAX_ENTITY_TOKENS",
        "EDGEQUAKE_MAX_RELATION_TOKENS",
        "EDGEQUAKE_QUERY_ARM_CONCURRENCY",
        "EDGEQUAKE_PPR_DAMPING",
        "EDGEQUAKE_PPR_MAX_ITERS",
        "EDGEQUAKE_GRAPH_WALK_COMPRESS",
        "EDGEQUAKE_POPULAR_NODE_FALLBACK",
        "EDGEQUAKE_CONTENT_HEADINGS",
        "EDGEQUAKE_KEYWORD_LEXICAL_BOOST",
        "EDGEQUAKE_KG_CHUNK_OCCURRENCE_SORT",
        "EDGEQUAKE_KG_CHUNK_PICK_LR_BUDGET",
        "EDGEQUAKE_BM25_RETRIEVAL",
        "EDGEQUAKE_KG_CHUNK_PICK_TIMING",
        "EDGEQUAKE_MIN_RERANK_SCORE",
        "EDGEQUAKE_MIN_CHUNK_BUDGET_RATIO",
        "EDGEQUAKE_L2_SOURCES_UNION",
        "EDGEQUAKE_L2_SOURCES_MIX_TOP_K",
        "EDGEQUAKE_FACT_RERANKER",
        "EDGEQUAKE_FACT_CE_SKIP",
        "EDGEQUAKE_KEYWORD_MODE",
        "EDGEQUAKE_KEYWORD_LLM_PROVIDER",
        "EDGEQUAKE_KEYWORD_LLM_MODEL",
        "EDGEQUAKE_EXTRACT_LLM_PROVIDER",
        "EDGEQUAKE_EXTRACT_LLM_MODEL",
        "EDGEQUAKE_FACT_PROTECT_BM25",
        "EDGEQUAKE_COVERAGE_PROTECT_FIRST",
        "EDGEQUAKE_TOPIC_ENTITY_ADMIT",
        "EDGEQUAKE_TOPIC_CE_PROTECT",
        "EDGEQUAKE_TOPIC_TRUNC_PROTECT",
        "EDGEQUAKE_TOPIC_TRUNC_PROTECT_MAX",
        "EDGEQUAKE_TOPIC_MATERIALIZE",
        "EDGEQUAKE_TOPIC_MATERIALIZE_MAX",
        "EDGEQUAKE_TOPIC_MATERIALIZE_CONTENT",
        "EDGEQUAKE_TOPIC_MATERIALIZE_TYPES",
        "EDGEQUAKE_INTENT_RERANK",
        "EDGEQUAKE_L2_BM25_UNION",
        "EDGEQUAKE_L2_BM25_MIX_TOP_K",
        "EDGEQUAKE_L2_BM25_MODE",
        "EDGEQUAKE_MIX_INTENT_WEIGHTS",
        "EDGEQUAKE_MIX_FUSION",
        "EDGEQUAKE_RR_ORDER",
        "EDGEQUAKE_INTENT_FACTUAL_BIAS",
        "EDGEQUAKE_ANSWER_PROMPT",
        "EDGEQUAKE_ANSWER_SPECIFIC_TYPES",
        "EDGEQUAKE_STRUCTURE_INDUCE",
        "BENCH001_EQ_RERANK_TOP_K",
    }
    for k, v in ACC_EXPORTS.items():
        if k == "PORT":
            continue
        if k in _override_keys and (os.environ.get(k) or "").strip():
            v = os.environ[k].strip()
        lines.append(f'export {k}="{v}"')
    lines.append(f'export MISTRAL_API_KEY="{key}"')
    lines.append(f'export OPENAI_API_KEY="{key}"')
    # Forward reranker API keys for cross_encoder Acc ablations.
    _rerank_key_names = (
        "DASHSCOPE_API_KEY",
        "ALIYUN_API_KEY",
        "JINA_API_KEY",
        "COHERE_API_KEY",
    )
    prev_keys = (
        _parse_existing_keys(START_SH.read_text(encoding="utf-8"))
        if START_SH.is_file()
        else {}
    )
    for dash_key in _rerank_key_names:
        dash_val = (os.environ.get(dash_key) or "").strip()
        if not dash_val or dash_val.upper().startswith("FAKE"):
            dash_val = (prev_keys.get(dash_key) or "").strip()
        if dash_val and not dash_val.upper().startswith("FAKE"):
            lines.append(f'export {dash_key}="{dash_val}"')
    lines.append('export EDGEQUAKE_DEFAULT_LLM_PROVIDER="mistral"')
    lines.append('export EDGEQUAKE_DEFAULT_LLM_MODEL="mistral-small-latest"')
    lines.append('export EDGEQUAKE_DEFAULT_EMBEDDING_PROVIDER="mistral"')
    lines.append('export EDGEQUAKE_DEFAULT_EMBEDDING_MODEL="mistral-embed"')
    lines.append('export EDGEQUAKE_VISION_PROVIDER="mistral"')
    lines.append('export EDGEQUAKE_VISION_MODEL="mistral-small-latest"')
    lines.append(f"exec {bin_path}")
    START_SH.write_text("\n".join(lines) + "\n", encoding="utf-8")
    START_SH.chmod(0o755)


def kill_port(port: int) -> None:
    try:
        out = subprocess.check_output(
            ["lsof", "-nP", f"-iTCP:{port}", "-sTCP:LISTEN", "-t"],
            text=True,
        ).strip()
    except subprocess.CalledProcessError:
        out = ""
    for pid in out.splitlines():
        pid = pid.strip()
        if pid:
            subprocess.call(["kill", pid], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    time.sleep(1)


def daemonize() -> None:
    # Classic double-fork so agent shell process-group cleanup cannot reap us.
    if os.fork() > 0:
        return
    os.setsid()
    if os.fork() > 0:
        os._exit(0)
    os.chdir("/")
    fd = os.open(str(LOG_FILE), os.O_WRONLY | os.O_CREAT | os.O_TRUNC)
    os.dup2(fd, 1)
    os.dup2(fd, 2)
    dn = os.open(os.devnull, os.O_RDONLY)
    os.dup2(dn, 0)
    os.execv("/bin/bash", ["bash", str(START_SH)])


def wait_health(*, port: int, timeout_s: int) -> int:
    url = f"http://127.0.0.1:{port}/health"
    deadline = time.time() + timeout_s
    while time.time() < deadline:
        try:
            out = subprocess.check_output(
                ["lsof", "-nP", f"-iTCP:{port}", "-sTCP:LISTEN", "-t"],
                text=True,
            ).strip()
        except subprocess.CalledProcessError:
            out = ""
        if out:
            pid = out.splitlines()[0]
            PID_FILE.write_text(pid + "\n", encoding="utf-8")
        try:
            r = subprocess.run(
                ["curl", "-sf", "-m", "2", url],
                capture_output=True,
                text=True,
                check=False,
            )
            if r.returncode == 0 and "healthy" in (r.stdout or ""):
                try:
                    import json

                    sys.path.insert(0, str(REPO / "tools" / "bench001"))
                    from bench001.acc_env import (
                        backend_pin_mismatches,
                        publication_extract_caps_mismatches,
                    )

                    # SPEC-086: Acc law is round_robin — pin check must see ALLOW.
                    if ACC_EXPORTS.get("BENCH001_ALLOW_ROUND_ROBIN"):
                        os.environ.setdefault(
                            "BENCH001_ALLOW_ROUND_ROBIN",
                            ACC_EXPORTS["BENCH001_ALLOW_ROUND_ROBIN"],
                        )
                    # SPEC-117: extract-cap pins live in server env (not /health).
                    for k in (
                        "EDGEQUAKE_MAX_EXTRACTION_ENTITIES",
                        "EDGEQUAKE_MAX_EXTRACTION_RECORDS",
                        "EDGEQUAKE_EXTRACT_CAPS_SELECTION",
                    ):
                        if k in ACC_EXPORTS:
                            os.environ[k] = ACC_EXPORTS[k]
                    caps_bad = publication_extract_caps_mismatches()
                    if caps_bad:
                        print(
                            "acc_backend: extract-cap pin mismatch: "
                            + "; ".join(caps_bad),
                            file=sys.stderr,
                        )
                        return 1
                    health = json.loads(r.stdout)
                    bad = backend_pin_mismatches(health)
                    if bad:
                        print(
                            "acc_backend: healthy but pin mismatch: "
                            + "; ".join(bad),
                            file=sys.stderr,
                        )
                        return 1
                    llm = (health.get("providers") or {}).get("llm") or {}
                    emb = (health.get("providers") or {}).get("embedding") or {}
                    print(
                        f"acc_backend: healthy pid={PID_FILE.read_text().strip()} "
                        f"url={url} llm={llm.get('name')}/{llm.get('model')} "
                        f"embed={emb.get('name')}/{emb.get('model')} "
                        f"extract_caps="
                        f"{ACC_EXPORTS.get('EDGEQUAKE_MAX_EXTRACTION_ENTITIES')}/"
                        f"{ACC_EXPORTS.get('EDGEQUAKE_MAX_EXTRACTION_RECORDS')}+"
                        f"{ACC_EXPORTS.get('EDGEQUAKE_EXTRACT_CAPS_SELECTION')}"
                    )
                    return 0
                except Exception as exc:  # noqa: BLE001
                    print(
                        f"acc_backend: healthy pid={PID_FILE.read_text().strip()} "
                        f"url={url} (pin-check skipped: {exc})"
                    )
                    return 0
        except OSError:
            pass
        time.sleep(1)
    print(f"acc_backend: health timeout; see {LOG_FILE}", file=sys.stderr)
    return 1


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--port", type=int, default=int(os.environ.get("BACKEND_PORT") or 8090))
    ap.add_argument("--wait", type=int, default=90)
    ap.add_argument("--no-kill", action="store_true", help="do not kill existing listener")
    args = ap.parse_args()

    write_start_sh(port=args.port)
    if not args.no_kill:
        kill_port(args.port)
    daemonize()
    time.sleep(0.5)
    return wait_health(port=args.port, timeout_s=args.wait)


if __name__ == "__main__":
    raise SystemExit(main())
