"""Scoring: local ROUGE-L Acc proxy + optional official generation_eval wrap."""

from __future__ import annotations

import json
import os
import re
import subprocess
import tempfile
from collections import defaultdict
from pathlib import Path
from typing import Any

from .paths import QUESTION_TYPES


def _tokenize(s: str) -> list[str]:
    return [t for t in "".join(c.lower() if c.isalnum() else " " for c in s).split() if t]


def rouge_l_f1(pred: str, gold: str) -> float:
    """Simple character-agnostic ROUGE-L F1 via LCS on tokens."""
    a, b = _tokenize(pred), _tokenize(gold)
    if not a and not b:
        return 1.0
    if not a or not b:
        return 0.0
    m, n = len(a), len(b)
    dp = [0] * (n + 1)
    for i in range(1, m + 1):
        prev = 0
        for j in range(1, n + 1):
            cur = dp[j]
            if a[i - 1] == b[j - 1]:
                dp[j] = prev + 1
            else:
                dp[j] = max(dp[j], dp[j - 1])
            prev = cur
    lcs = dp[n]
    prec = lcs / len(a)
    rec = lcs / len(b)
    if prec + rec == 0:
        return 0.0
    return 2 * prec * rec / (prec + rec)


def token_overlap_acc(pred: str, gold: str, *, threshold: float = 0.5) -> float:
    """Binary Acc proxy: ROUGE-L F1 >= threshold."""
    return 1.0 if rouge_l_f1(pred, gold) >= threshold else 0.0


def score_predictions_local(predictions: list[dict[str, Any]]) -> dict[str, Any]:
    """Local proxy judge (no LLM). Used for dry-run and when official eval unavailable."""
    by_type: dict[str, dict[str, list[float]]] = defaultdict(lambda: defaultdict(list))
    accs: list[float] = []
    for item in predictions:
        qtype = item.get("question_type", "Uncategorized")
        pred = item.get("generated_answer") or ""
        gold = item.get("ground_truth") or item.get("gold_answer") or ""
        r = rouge_l_f1(pred, gold)
        a = token_overlap_acc(pred, gold)
        by_type[qtype]["rouge_score"].append(r)
        by_type[qtype]["answer_correctness"].append(a)
        accs.append(a)

    out_by_type: dict[str, dict[str, float]] = {}
    for qtype in QUESTION_TYPES:
        metrics = by_type.get(qtype, {})
        out_by_type[qtype] = {
            k: (sum(v) / len(v) if v else 0.0) for k, v in metrics.items()
        }
        if qtype in ("Contextual Summarize", "Creative Generation"):
            out_by_type[qtype]["coverage_score"] = out_by_type[qtype].get("rouge_score", 0.0)
        if qtype == "Creative Generation":
            out_by_type[qtype]["faithfulness"] = out_by_type[qtype].get("rouge_score", 0.0)

    return {
        "judge": "rouge_proxy",
        "overall_acc": (sum(accs) / len(accs)) if accs else 0.0,
        "by_type": out_by_type,
        "n": len(predictions),
    }


def _to_eval_json(predictions: list[dict[str, Any]], path: Path) -> None:
    """Normalize to generation_eval expected fields."""
    rows = []
    for p in predictions:
        ctx = p.get("context") or []
        if isinstance(ctx, str):
            ctx = [ctx]
        rows.append(
            {
                "id": p["id"],
                "question": p["question"],
                "question_type": p["question_type"],
                "generated_answer": p.get("generated_answer") or "",
                "ground_truth": p.get("ground_truth") or p.get("gold_answer") or "",
                "context": ctx if ctx else [""],
            }
        )
    path.write_text(json.dumps(rows, indent=2), encoding="utf-8")


def _to_retrieval_eval_json(predictions: list[dict[str, Any]], path: Path) -> None:
    """Normalize to retrieval_eval fields (context + gold evidence)."""
    rows = []
    for p in predictions:
        ctx = p.get("context") or []
        if isinstance(ctx, str):
            ctx = [ctx]
        ev = p.get("evidence") or []
        if isinstance(ev, str):
            ev = [ev]
        rows.append(
            {
                "id": p["id"],
                "question": p["question"],
                "question_type": p["question_type"],
                "context": ctx if ctx else [""],
                "evidence": ev if ev else [""],
            }
        )
    path.write_text(json.dumps(rows, indent=2), encoding="utf-8")


def _eval_root() -> Path | None:
    eval_root = os.environ.get("BENCH001_GRAPHRAG_BENCH_REPO")
    if eval_root:
        return Path(eval_root)
    from .paths import REPO_ROOT

    cand = REPO_ROOT.parent / "GraphRAG-Benchmark"
    if cand.exists():
        return cand
    cached = Path.home() / ".cache/edgequake/bench001/GraphRAG-Benchmark"
    return cached if cached.exists() else None


def _subprocess_env(eval_root: Path) -> dict[str, str]:
    env = os.environ.copy()
    env["PYTHONPATH"] = (
        f"{eval_root}{os.pathsep}{env['PYTHONPATH']}"
        if env.get("PYTHONPATH")
        else str(eval_root)
    )
    return env



def _run_eval_subprocess(cmd: list[str], *, cwd: str, env: dict[str, str], timeout: float) -> None:
    """Run official eval with live stdout/stderr tee + sample progress ETA."""
    import threading
    import time as _time

    from .progress import format_duration, mark_phase, print_unit_progress, run_elapsed_s

    proc = subprocess.Popen(
        cmd,
        cwd=cwd,
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1,
    )
    assert proc.stdout is not None
    start = _time.time()
    last_hb = start
    lines = 0
    sample_done = 0
    sample_total = 0
    sample_re = re.compile(r"Completed sample\s+(\d+)\s*/\s*(\d+)", re.IGNORECASE)
    stage_name = os.environ.get("BENCH001_PROGRESS_STAGE") or "smoke-fast"
    label = "judge"
    # Heuristic label from cmd path / args.
    joined = " ".join(cmd)
    if "generation_eval" in joined:
        label = "judge generation"
    elif "retrieval_eval" in joined:
        label = "judge retrieval"

    def _heartbeat() -> None:
        nonlocal last_hb
        while proc.poll() is None:
            now = _time.time()
            if now - last_hb >= 30.0:
                elapsed = int(now - start)
                prog = (
                    f" samples={sample_done}/{sample_total}"
                    if sample_total
                    else f" lines={lines}"
                )
                print(
                    f"  {label} subprocess still running pid={proc.pid}{prog} "
                    f"elapsed={format_duration(elapsed)} "
                    f"run={format_duration(run_elapsed_s())}",
                    flush=True,
                )
                last_hb = now
            _time.sleep(5)

    hb = threading.Thread(target=_heartbeat, daemon=True)
    hb.start()
    try:
        for line in proc.stdout:
            lines += 1
            text = line.rstrip("\n")
            print(text, flush=True)
            last_hb = _time.time()
            m = sample_re.search(text)
            if m:
                sample_done = max(sample_done, int(m.group(1)))
                sample_total = max(sample_total, int(m.group(2)))
                elapsed = max(_time.time() - start, 1e-6)
                print_unit_progress(
                    label,
                    sample_done,
                    sample_total,
                    elapsed_s=elapsed,
                )
                mark_phase(
                    stage_name,
                    "score_parallel",
                    status="running",
                    detail=f"{label} {sample_done}/{sample_total}",
                    done=sample_done,
                    total=sample_total,
                    phase_elapsed_s=elapsed,
                    quiet=True,
                )
            if (_time.time() - start) > timeout:
                proc.kill()
                raise subprocess.TimeoutExpired(cmd, timeout)
        rc = proc.wait()
    except Exception:
        if proc.poll() is None:
            proc.kill()
        raise
    if rc != 0:
        raise subprocess.CalledProcessError(rc, cmd)


def try_official_generation_eval(
    predictions: list[dict[str, Any]],
    *,
    output_file: Path,
    model: str | None = None,
    base_url: str | None = None,
    embedding_model: str | None = None,
    concurrency: int | None = None,
) -> dict[str, Any] | None:
    """
    Invoke GraphRAG-Bench Evaluation.generation_eval.

    Judge LLM + metric embed come from active ProviderPins / judge_tune env.
    Default metric embed is mistral-embed (OpenAI-compatible); BGE remains
    available for paper parity via --judge-embedding-model BAAI/bge-large-en-v1.5.
    """
    from .judge_tune import export_judge_env
    from .profiles import (
        active_pins,
        eval_concurrency,
        judge_api_key,
    )

    pins = active_pins()
    api_key = judge_api_key(pins)
    if not api_key:
        return None
    model = model or pins.judge_model
    base_url = base_url or pins.judge_base_url
    embedding_model = embedding_model or pins.judge_embedding_model
    conc = eval_concurrency(concurrency)

    eval_root = _eval_root()
    if eval_root is None:
        return None

    # Force-assign: agent shells may inject placeholder LLM_API_KEY=FAKE* that
    # setdefault would keep, causing official judge 401 → rouge_proxy fallback.
    os.environ["LLM_API_KEY"] = api_key
    os.environ["BENCH001_EVAL_CONCURRENCY"] = str(conc)
    for k, v in export_judge_env(embed_base_url=pins.judge_base_url).items():
        os.environ[k] = v
    # Metric embed API key: same as judge when using mistral-embed.
    os.environ["MISTRAL_API_KEY"] = api_key
    os.environ["OPENAI_API_KEY"] = api_key

    _ensure_generation_eval_patches(eval_root)
    print(
        f"official generation_eval concurrency={conc} "
        f"judge_model={model} metric_embed={embedding_model} "
        f"embed_backend={os.environ.get('BENCH001_JUDGE_EMBED_BACKEND', 'auto')} "
        f"temp={os.environ.get('BENCH001_JUDGE_TEMPERATURE')} "
        f"fact_w={os.environ.get('BENCH001_ACC_FACTUALITY_WEIGHT')}",
        flush=True,
    )
    with tempfile.TemporaryDirectory() as tmp:
        data_file = Path(tmp) / "preds.json"
        _to_eval_json(predictions, data_file)
        os.environ.setdefault("BENCH001_ACC_DECOMPOSE", "1")
        cmd = [
            "python3",
            "-m",
            "Evaluation.generation_eval",
            "--mode",
            "API",
            "--model",
            model,
            "--base_url",
            base_url,
            "--embedding_model",
            embedding_model,
            "--data_file",
            str(data_file),
            "--output_file",
            str(output_file),
            "--max_concurrent",
            str(conc),
            "--detailed_output",
        ]
        try:
            _run_eval_subprocess(
                cmd,
                cwd=str(eval_root),
                env=_subprocess_env(eval_root),
                timeout=3600,
            )
        except (subprocess.CalledProcessError, FileNotFoundError, subprocess.TimeoutExpired) as exc:
            err = getattr(exc, "stderr", None) or getattr(exc, "stdout", None) or exc
            print(f"official generation_eval failed: {err}", flush=True)
            return None

    if not output_file.exists():
        return None
    raw = json.loads(output_file.read_text(encoding="utf-8"))
    raw_path = output_file.with_suffix(".raw.json")
    raw_path.write_text(json.dumps(raw, indent=2), encoding="utf-8")
    return _normalize_official(raw)


def try_official_retrieval_eval(
    predictions: list[dict[str, Any]],
    *,
    output_file: Path,
    model: str | None = None,
    base_url: str | None = None,
    embedding_model: str | None = None,
    concurrency: int | None = None,
) -> dict[str, Any] | None:
    """Invoke GraphRAG-Bench Evaluation.retrieval_eval (L2 Evidence Recall + Relevancy)."""
    from .judge_tune import export_judge_env
    from .profiles import active_pins, eval_concurrency, judge_api_key

    pins = active_pins()
    api_key = judge_api_key(pins)
    if not api_key:
        return None
    model = model or pins.judge_model
    base_url = base_url or pins.judge_base_url
    embedding_model = embedding_model or pins.judge_embedding_model
    conc = eval_concurrency(concurrency)
    eval_root = _eval_root()
    if eval_root is None:
        return None

    # Force-assign: agent shells may inject placeholder LLM_API_KEY=FAKE* that
    # setdefault would keep, causing official judge 401 → rouge_proxy fallback.
    os.environ["LLM_API_KEY"] = api_key
    os.environ["BENCH001_EVAL_CONCURRENCY"] = str(conc)
    for k, v in export_judge_env(embed_base_url=pins.judge_base_url).items():
        os.environ[k] = v
    os.environ["MISTRAL_API_KEY"] = api_key
    os.environ["OPENAI_API_KEY"] = api_key

    _ensure_retrieval_eval_patches(eval_root)
    print(
        f"official retrieval_eval concurrency={conc} judge_model={model}",
        flush=True,
    )
    with tempfile.TemporaryDirectory() as tmp:
        data_file = Path(tmp) / "preds.json"
        _to_retrieval_eval_json(predictions, data_file)
        cmd = [
            "python3",
            "-m",
            "Evaluation.retrieval_eval",
            "--mode",
            "API",
            "--model",
            model,
            "--base_url",
            base_url,
            "--embedding_model",
            embedding_model,
            "--data_file",
            str(data_file),
            "--output_file",
            str(output_file),
            "--max_concurrent",
            str(conc),
        ]
        try:
            _run_eval_subprocess(
                cmd,
                cwd=str(eval_root),
                env=_subprocess_env(eval_root),
                timeout=3600,
            )
        except (subprocess.CalledProcessError, FileNotFoundError, subprocess.TimeoutExpired) as exc:
            err = getattr(exc, "stderr", None) or getattr(exc, "stdout", None) or exc
            print(f"official retrieval_eval failed: {err}", flush=True)
            out = getattr(exc, "stdout", None)
            if out:
                print(f"official retrieval_eval stdout (tail): {str(out)[-2000:]}", flush=True)
            return None

    if not output_file.exists():
        return None
    raw = json.loads(output_file.read_text(encoding="utf-8"))
    raw_path = output_file.with_name(output_file.stem + ".raw.json")
    raw_path.write_text(json.dumps(raw, indent=2), encoding="utf-8")
    return _normalize_retrieval(raw)


def _ensure_generation_eval_patches(eval_root: Path) -> None:
    """Patch upstream generation_eval + answer_accuracy for bench001 judge knobs."""
    _patch_generation_eval_py(eval_root / "Evaluation" / "generation_eval.py")
    _patch_answer_accuracy_py(eval_root / "Evaluation" / "metrics" / "answer_accuracy.py")


def _ensure_retrieval_eval_patches(eval_root: Path) -> None:
    """Patch upstream retrieval_eval for Mistral (no seed) + concurrency."""
    path = eval_root / "Evaluation" / "retrieval_eval.py"
    if not path.exists():
        return
    text = path.read_text(encoding="utf-8")
    original = text

    # Upstream imports ragas.embeddings (API path unused). Soften to optional
    # so broken instructor↔mistralai stacks do not block L2 scoring.
    if (
        "except ImportError:  # BENCH001 optional" not in text
        and "from ragas.embeddings import LangchainEmbeddingsWrapper\n" in text
    ):
        text = text.replace(
            "from ragas.embeddings import LangchainEmbeddingsWrapper\n",
            "try:\n"
            "    from ragas.embeddings import LangchainEmbeddingsWrapper\n"
            "except ImportError:  # BENCH001 optional\n"
            "    LangchainEmbeddingsWrapper = None  # type: ignore\n",
            1,
        )

    if "--max_concurrent" not in text:
        text = text.replace(
            "results = await evaluate_dataset(\n"
            "            dataset=dataset,\n"
            "            llm=llm, \n"
            "            embeddings=embedding,\n"
            "            detailed_output=args.detailed_output\n"
            "        )",
            "results = await evaluate_dataset(\n"
            "            dataset=dataset,\n"
            "            llm=llm,\n"
            "            embeddings=embedding,\n"
            "            max_concurrent=args.max_concurrent,\n"
            "            detailed_output=args.detailed_output\n"
            "        )",
            1,
        )
        insert = '''
    parser.add_argument(
        "--max_concurrent",
        type=int,
        default=int(os.environ.get("BENCH001_EVAL_CONCURRENCY", "8")),
        help="In-flight sample evaluations",
    )
'''
        text = text.replace(
            '    parser.add_argument(\n'
            '        "--detailed_output",\n'
            '        action="store_true",\n'
            '        help="Whether to include detailed output"\n'
            "    )\n"
            "    \n"
            "    # Parse arguments",
            '    parser.add_argument(\n'
            '        "--detailed_output",\n'
            '        action="store_true",\n'
            '        help="Whether to include detailed output"\n'
            "    )\n"
            + insert
            + "\n"
            "    # Parse arguments",
            1,
        )

    # Mistral rejects seed in ChatOpenAI model_kwargs — strip it.
    if "BENCH001_JUDGE_TEMPERATURE" not in text and "model_kwargs={" in text:
        text = re.sub(
            r"llm = ChatOpenAI\(\s*model=args\.model,\s*base_url=args\.base_url,\s*"
            r"api_key=SecretStr\(api_key\),\s*temperature=[^\n]+,\s*max_retries=3,\s*"
            r"timeout=\d+,\s*model_kwargs=\{[^}]+\}\s*\)",
            "llm = ChatOpenAI(\n"
            "            model=args.model,\n"
            "            base_url=args.base_url,\n"
            "            api_key=SecretStr(api_key),\n"
            "            temperature=float(os.environ.get('BENCH001_JUDGE_TEMPERATURE', '0.0')),\n"
            "            max_retries=3,\n"
            "            timeout=120,\n"
            "        )",
            text,
            count=1,
            flags=re.S,
        )

    # Metric embed: openai_compat for mistral-embed (unused by metrics today, but
    # avoids HuggingFace download failures on API-only hosts).
    if "BENCH001_EMBED_BACKEND" not in text:
        old = "embedding = HuggingFaceBgeEmbeddings(model_name=args.embedding_model)"
        new = '''_emb_backend = os.environ.get("BENCH001_JUDGE_EMBED_BACKEND", "auto")
        _emb_name = (args.embedding_model or "").lower()
        if _emb_backend == "auto":
            _emb_backend = (
                "hf_bge"
                if ("bge" in _emb_name or _emb_name.startswith("baai/"))
                else "openai_compat"
            )
        if _emb_backend == "openai_compat":
            from langchain_openai import OpenAIEmbeddings
            embedding = OpenAIEmbeddings(
                model=args.embedding_model,
                base_url=os.environ.get("BENCH001_JUDGE_EMBED_BASE_URL") or args.base_url,
                api_key=SecretStr(api_key),
                check_embedding_ctx_length=False,
            )
        else:
            embedding = HuggingFaceBgeEmbeddings(model_name=args.embedding_model)'''
        if old in text:
            text = text.replace(old, new, 1)

    # Parallelize question-type loops (same wall-time win as generation_eval).
    if "BENCH001_PARALLEL_QTYPES" not in text:
        old_loop = (
            "    all_results = {}\n"
            "    \n"
            "    # Evaluate each question type\n"
            "    for question_type in list(grouped_data.keys()):\n"
            "        print(f\"\\n{'='*50}\")\n"
            "        print(f\"Evaluating question type: {question_type}\")\n"
            "        print(f\"{'='*50}\")\n"
            "        \n"
            "        # Prepare data from grouped items\n"
            "        group_items = grouped_data[question_type]\n"
            "\n"
            "        ids = [item['id'] for item in group_items]\n"
            "        questions = [item['question'] for item in group_items]\n"
            "        evidences = [item['evidence'] for item in group_items]\n"
            "        contexts = [item['context'] for item in group_items]\n"
            "        \n"
            "        # Create dataset\n"
            "        data = {\n"
            "            \"id\": ids,\n"
            "            \"question\": questions,\n"
            "            \"contexts\": contexts,\n"
            "            \"evidences\": evidences\n"
            "        }\n"
            "        dataset = Dataset.from_dict(data)\n"
            "        \n"
            "        # If sample\n"
            "        if args.num_samples:\n"
            "            dataset = dataset.select([i for i in list(range(args.num_samples))])\n"
            "        \n"
            "        # Perform evaluation\n"
            "        results = await evaluate_dataset(\n"
            "            dataset=dataset,\n"
            "            llm=llm,\n"
            "            embeddings=embedding,\n"
            "            max_concurrent=args.max_concurrent,\n"
            "            detailed_output=args.detailed_output\n"
            "        )\n"
            "        \n"
            "        all_results[question_type] = results\n"
            "        print(f\"\\nResults for {question_type}:\")\n"
            "        if args.detailed_output:\n"
            "            for metric, score in results[\"average_scores\"].items():\n"
            "                print(f\"  {metric}: {score:.4f}\")\n"
            "        else:\n"
            "            for metric, score in results.items():\n"
            "                print(f\"  {metric}: {score:.4f}\")"
        )
        new_loop = (
            "    all_results = {}\n"
            "\n"
            "    # BENCH001_PARALLEL_QTYPES — evaluate question types concurrently\n"
            "    async def _bench001_eval_qtype(question_type):\n"
            "        print(f\"\\n{'='*50}\")\n"
            "        print(f\"Evaluating question type: {question_type}\")\n"
            "        print(f\"{'='*50}\")\n"
            "        group_items = grouped_data[question_type]\n"
            "        data = {\n"
            "            \"id\": [item['id'] for item in group_items],\n"
            "            \"question\": [item['question'] for item in group_items],\n"
            "            \"contexts\": [item['context'] for item in group_items],\n"
            "            \"evidences\": [item['evidence'] for item in group_items],\n"
            "        }\n"
            "        dataset = Dataset.from_dict(data)\n"
            "        if args.num_samples:\n"
            "            dataset = dataset.select([i for i in list(range(args.num_samples))])\n"
            "        results = await evaluate_dataset(\n"
            "            dataset=dataset,\n"
            "            llm=llm,\n"
            "            embeddings=embedding,\n"
            "            max_concurrent=args.max_concurrent,\n"
            "            detailed_output=args.detailed_output,\n"
            "        )\n"
            "        print(f\"\\nResults for {question_type}:\")\n"
            "        scores = results[\"average_scores\"] if args.detailed_output else results\n"
            "        for metric, score in scores.items():\n"
            "            print(f\"  {metric}: {score:.4f}\")\n"
            "        return question_type, results\n"
            "\n"
            "    _qtypes = list(grouped_data.keys())\n"
            "    _gathered = await asyncio.gather(*[_bench001_eval_qtype(qt) for qt in _qtypes])\n"
            "    for question_type, results in _gathered:\n"
            "        all_results[question_type] = results"
        )
        if old_loop in text:
            text = text.replace(old_loop, new_loop, 1)

    if text != original:
        path.write_text(text, encoding="utf-8")
        print(f"patched {path} for bench001 retrieval_eval", flush=True)


def _patch_generation_eval_py(path: Path) -> None:
    if not path.exists():
        return
    text = path.read_text(encoding="utf-8")
    original = text

    if "--max_concurrent" not in text:
        text = text.replace(
            "detailed_output=args.detailed_output\n        )",
            "max_concurrent=args.max_concurrent,\n"
            "            detailed_output=args.detailed_output,\n"
            "        )",
            1,
        )
        insert = '''
    parser.add_argument(
        "--max_concurrent",
        type=int,
        default=int(os.environ.get("BENCH001_EVAL_CONCURRENCY", "16")),
        help="In-flight sample evaluations per question_type",
    )
'''
        text = text.replace(
            '    parser.add_argument(\n        "--detailed_output",\n        action="store_true",\n        help="Whether to include detailed output"\n    )\n    \n    args = parser.parse_args()',
            '    parser.add_argument(\n        "--detailed_output",\n        action="store_true",\n        help="Whether to include detailed output"\n    )\n'
            + insert
            + "\n    args = parser.parse_args()",
            1,
        )

    # Unpack Acc component dicts from compute_answer_correctness (F1 + cos).
    if "BENCH001_ACC_DECOMPOSE_UNPACK" not in text:
        old_agg = (
            "                if isinstance(result, dict):\n"
            "                    metrics_dict = result.get(\"metrics\")\n"
            "                    if isinstance(metrics_dict, dict):\n"
            "                        for metric, score in metrics_dict.items():\n"
            "                            if isinstance(score, (int, float)) "
            "and not np.isnan(score):\n"
            "                                results[metric].append(score)"
        )
        new_agg = (
            "                # BENCH001_ACC_DECOMPOSE_UNPACK\n"
            "                if isinstance(result, dict):\n"
            "                    metrics_dict = result.get(\"metrics\")\n"
            "                    if isinstance(metrics_dict, dict):\n"
            "                        for metric, score in metrics_dict.items():\n"
            "                            if isinstance(score, (int, float)) "
            "and not np.isnan(score):\n"
            "                                results.setdefault(metric, []).append(score)"
        )
        if old_agg in text:
            text = text.replace(old_agg, new_agg, 1)
        old_plain = (
            "                if isinstance(result, dict):\n"
            "                    for metric, score in result.items():\n"
            "                        if isinstance(score, (int, float)) "
            "and not np.isnan(score):\n"
            "                            results[metric].append(score)"
        )
        new_plain = (
            "                if isinstance(result, dict):\n"
            "                    for metric, score in result.items():\n"
            "                        if isinstance(score, (int, float)) "
            "and not np.isnan(score):\n"
            "                            results.setdefault(metric, []).append(score)"
        )
        if old_plain in text:
            text = text.replace(old_plain, new_plain, 1)
        old_sample = (
            "    for i, metric in enumerate(tasks.keys()):\n"
            "        results[metric] = task_results[i]\n"
            "    \n"
            "    return results"
        )
        new_sample = (
            "    # BENCH001_ACC_DECOMPOSE_UNPACK — expand Acc component dicts\n"
            "    for i, metric in enumerate(tasks.keys()):\n"
            "        val = task_results[i]\n"
            "        if isinstance(val, dict):\n"
            "            results.update(val)\n"
            "        else:\n"
            "            results[metric] = val\n"
            "    \n"
            "    return results"
        )
        if old_sample in text:
            text = text.replace(old_sample, new_sample, 1)

    # Parallelize question-type loops (Fact/Reasoning/Summarize/Creative).
    if "BENCH001_PARALLEL_QTYPES" not in text:
        old_loop = (
            "    all_results = {}\n"
            "    \n"
            "    # Evaluate each found question type (only those in metric_config)\n"
            "    for question_type in list(grouped_data.keys()):\n"
            "        # Skip types not defined in metric_config\n"
            "        if question_type not in metric_config:\n"
            "            print(f\"Skipping undefined question type: {question_type}\")\n"
            "            continue\n"
            "            \n"
            "        print(f\"\\n{'='*50}\")\n"
            "        print(f\"Evaluating question type: {question_type}\")\n"
            "        print(f\"{'='*50}\")\n"
            "        \n"
            "        # Prepare data from grouped items\n"
            "        group_items = grouped_data[question_type]\n"
            "        ids = [item['id'] for item in group_items]\n"
            "        questions = [item['question'] for item in group_items]\n"
            "        ground_truths = [item['ground_truth'] for item in group_items]\n"
            "        answers = [item['generated_answer'] for item in group_items]\n"
            "        contexts = [item['context'] for item in group_items]\n"
            "        \n"
            "        # Create dataset\n"
            "        data = {\n"
            "            \"id\": ids,\n"
            "            \"question\": questions,\n"
            "            \"answer\": answers,\n"
            "            \"contexts\": contexts,\n"
            "            \"ground_truth\": ground_truths\n"
            "        }\n"
            "        dataset = Dataset.from_dict(data)\n"
            "\n"
            "        # If sample\n"
            "        if args.num_samples:\n"
            "            dataset = dataset.select([i for i in list(range(args.num_samples))])\n"
            "\n"
            "        # Perform evaluation\n"
            "        results = await evaluate_dataset(\n"
            "            dataset=dataset,\n"
            "            metrics=metric_config[question_type],\n"
            "            llm=llm,\n"
            "            embeddings=embedding,\n"
            "            max_concurrent=args.max_concurrent,\n"
            "            detailed_output=args.detailed_output,\n"
            "        )\n"
            "        \n"
            "        all_results[question_type] = results\n"
            "        print(f\"\\nResults for {question_type}:\")\n"
            "        if args.detailed_output:\n"
            "            for metric, score in results[\"average_scores\"].items():\n"
            "                print(f\"  {metric}: {score:.4f}\")\n"
            "        else:\n"
            "            for metric, score in results.items():\n"
            "                print(f\"  {metric}: {score:.4f}\")"
        )
        new_loop = (
            "    all_results = {}\n"
            "\n"
            "    # BENCH001_PARALLEL_QTYPES — evaluate question types concurrently\n"
            "    async def _bench001_eval_qtype(question_type):\n"
            "        print(f\"\\n{'='*50}\")\n"
            "        print(f\"Evaluating question type: {question_type}\")\n"
            "        print(f\"{'='*50}\")\n"
            "        group_items = grouped_data[question_type]\n"
            "        data = {\n"
            "            \"id\": [item['id'] for item in group_items],\n"
            "            \"question\": [item['question'] for item in group_items],\n"
            "            \"answer\": [item['generated_answer'] for item in group_items],\n"
            "            \"contexts\": [item['context'] for item in group_items],\n"
            "            \"ground_truth\": [item['ground_truth'] for item in group_items],\n"
            "        }\n"
            "        dataset = Dataset.from_dict(data)\n"
            "        if args.num_samples:\n"
            "            dataset = dataset.select([i for i in list(range(args.num_samples))])\n"
            "        results = await evaluate_dataset(\n"
            "            dataset=dataset,\n"
            "            metrics=metric_config[question_type],\n"
            "            llm=llm,\n"
            "            embeddings=embedding,\n"
            "            max_concurrent=args.max_concurrent,\n"
            "            detailed_output=args.detailed_output,\n"
            "        )\n"
            "        print(f\"\\nResults for {question_type}:\")\n"
            "        scores = results[\"average_scores\"] if args.detailed_output else results\n"
            "        for metric, score in scores.items():\n"
            "            print(f\"  {metric}: {score:.4f}\")\n"
            "        return question_type, results\n"
            "\n"
            "    _qtypes = [qt for qt in grouped_data.keys() if qt in metric_config]\n"
            "    for qt in grouped_data.keys():\n"
            "        if qt not in metric_config:\n"
            "            print(f\"Skipping undefined question type: {qt}\")\n"
            "    _gathered = await asyncio.gather(*[_bench001_eval_qtype(qt) for qt in _qtypes])\n"
            "    for question_type, results in _gathered:\n"
            "        all_results[question_type] = results"
        )
        if old_loop in text:
            text = text.replace(old_loop, new_loop, 1)

    # Mistral-safe ChatOpenAI + tunable temperature.
    if "BENCH001_JUDGE_TEMPERATURE" not in text or '"seed": SEED' in text:
        text = re.sub(
            r"llm = ChatOpenAI\(\s*model=args\.model,\s*base_url=args\.base_url,\s*"
            r"api_key=SecretStr\(api_key\),\s*temperature=[^\n]+,\s*max_retries=3,\s*"
            r"timeout=\d+,\s*(?:model_kwargs=\{[^}]+\}\s*)?\)",
            "llm = ChatOpenAI(\n"
            "            model=args.model,\n"
            "            base_url=args.base_url,\n"
            "            api_key=SecretStr(api_key),\n"
            "            temperature=float(os.environ.get('BENCH001_JUDGE_TEMPERATURE', '0.0')),\n"
            "            max_retries=3,\n"
            "            timeout=120,\n"
            "        )",
            text,
            count=1,
            flags=re.S,
        )

    # Metric embed: mistral-embed / OpenAI-compat OR HuggingFace BGE (paper).
    if "BENCH001_EMBED_BACKEND" not in text:
        old = "embedding = HuggingFaceBgeEmbeddings(model_name=args.embedding_model)"
        new = '''# BENCH001_EMBED_BACKEND — mistral-embed via OpenAI-compat, or HF BGE for paper parity
        _emb_backend = os.environ.get("BENCH001_JUDGE_EMBED_BACKEND", "auto")
        _emb_name = (args.embedding_model or "").lower()
        if _emb_backend == "auto":
            _emb_backend = (
                "hf_bge"
                if ("bge" in _emb_name or _emb_name.startswith("baai/"))
                else "openai_compat"
            )
        if _emb_backend == "openai_compat":
            from langchain_openai import OpenAIEmbeddings
            embedding = OpenAIEmbeddings(
                model=args.embedding_model,
                base_url=os.environ.get("BENCH001_JUDGE_EMBED_BASE_URL") or args.base_url,
                api_key=SecretStr(api_key),
                check_embedding_ctx_length=False,
            )
        else:
            embedding = HuggingFaceBgeEmbeddings(model_name=args.embedding_model)'''
        if old in text:
            text = text.replace(old, new, 1)

    if text != original:
        path.write_text(text, encoding="utf-8")
        print(f"patched {path} for bench001 judge embed/temp/concurrency", flush=True)


def _patch_answer_accuracy_py(path: Path) -> None:
    if not path.exists():
        return
    text = path.read_text(encoding="utf-8")
    original = text

    if "BENCH001_ACC_FACTUALITY_WEIGHT" not in text:
        needle = (
            '    """Compute answer correctness score combining factuality '
            'and semantic similarity"""\n'
        )
        insert = (
            needle
            + "    # BENCH001_ACC_FACTUALITY_WEIGHT — override Acc mix "
            "(default 0.75 F1 / 0.25 sim)\n"
            + "    import os as _bench001_os\n"
            + "    _fw = _bench001_os.environ.get('BENCH001_ACC_FACTUALITY_WEIGHT')\n"
            + "    if _fw is not None and str(_fw).strip():\n"
            + "        try:\n"
            + "            _w = max(0.0, min(1.0, float(_fw)))\n"
            + "            weights = [_w, 1.0 - _w]\n"
            + "        except ValueError:\n"
            + "            pass\n"
        )
        if needle in text:
            text = text.replace(needle, insert, 1)

    # Return Acc components (F1 + cos) when BENCH001_ACC_DECOMPOSE != 0 (default on).
    if "BENCH001_ACC_DECOMPOSE" not in text:
        old_ret = (
            "    # Combine scores using weighted average\n"
            "    return float(np.average([factuality_score, similarity_score], weights=weights))"
        )
        new_ret = (
            "    # Combine scores using weighted average\n"
            "    _acc = float(np.average([factuality_score, similarity_score], weights=weights))\n"
            "    # BENCH001_ACC_DECOMPOSE — export F1 + cos for decision Acc (default on)\n"
            "    import os as _bench001_os2\n"
            "    _decomp = (_bench001_os2.environ.get('BENCH001_ACC_DECOMPOSE') or '1')"
            ".strip().lower()\n"
            "    if _decomp not in {'0', 'false', 'no', 'off'}:\n"
            "        return {\n"
            "            'answer_correctness': _acc,\n"
            "            'factuality_f1': float(factuality_score),\n"
            "            'embed_cosine': float(similarity_score),\n"
            "        }\n"
            "    return _acc"
        )
        if old_ret in text:
            text = text.replace(old_ret, new_ret, 1)

    # Robust JSON parse for statement classification (Mistral often wraps markdown).
    if "BENCH001_FACTUALITY_JSON_FALLBACK" not in text:
        old_cls = (
            "    response = await llm.ainvoke(prompt, config={\"callbacks\": callbacks})\n"
            "    \n"
            "    try:\n"
            "        classification = ClassificationWithReason(**json.loads(response.content))\n"
            "        tp = len(classification.TP)\n"
            "        fp = len(classification.FP)\n"
            "        fn = len(classification.FN)\n"
            "        return fbeta_score(tp, fp, fn, beta)\n"
            "    except (json.JSONDecodeError, TypeError):\n"
            "        return 0.0  # Return minimum score on failure"
        )
        new_cls = (
            "    response = await llm.ainvoke(prompt, config={\"callbacks\": callbacks})\n"
            "    \n"
            "    # BENCH001_FACTUALITY_JSON_FALLBACK — Mistral-safe JSON parse\n"
            "    try:\n"
            "        _handler = JSONHandler()\n"
            "        _parsed = await _handler.parse_with_fallbacks(response.content)\n"
            "        if isinstance(_parsed, dict):\n"
            "            classification = ClassificationWithReason(**_parsed)\n"
            "        else:\n"
            "            classification = ClassificationWithReason(**json.loads(response.content))\n"
            "        tp = len(classification.TP)\n"
            "        fp = len(classification.FP)\n"
            "        fn = len(classification.FN)\n"
            "        return fbeta_score(tp, fp, fn, beta)\n"
            "    except Exception:\n"
            "        try:\n"
            "            _raw = response.content if isinstance(response.content, str) else str(response.content)\n"
            "            _raw = _raw.strip()\n"
            "            if _raw.startswith('```'):\n"
            "                _raw = _raw.strip('`')\n"
            "                if _raw.lower().startswith('json'):\n"
            "                    _raw = _raw[4:].lstrip()\n"
            "                if '```' in _raw:\n"
            "                    _raw = _raw.split('```')[0]\n"
            "            classification = ClassificationWithReason(**json.loads(_raw))\n"
            "            tp = len(classification.TP)\n"
            "            fp = len(classification.FP)\n"
            "            fn = len(classification.FN)\n"
            "            return fbeta_score(tp, fp, fn, beta)\n"
            "        except Exception:\n"
            "            return 0.0  # Return minimum score on failure"
        )
        if old_cls in text:
            text = text.replace(old_cls, new_cls, 1)

    if text != original:
        path.write_text(text, encoding="utf-8")
        print(f"patched {path} for Acc weight/decompose", flush=True)


def _finite(v: Any) -> float | None:
    try:
        f = float(v)
    except (TypeError, ValueError):
        return None
    if f != f:  # NaN
        return None
    return f


def _normalize_official(raw: dict[str, Any]) -> dict[str, Any] | None:
    by_type: dict[str, dict[str, float]] = {}
    accs: list[float] = []
    f1s: list[float] = []
    coss: list[float] = []
    for qtype, block in raw.items():
        metrics = block.get("average_scores", block) if isinstance(block, dict) else {}
        if not isinstance(metrics, dict):
            continue
        cleaned: dict[str, float] = {}
        for k, v in metrics.items():
            fv = _finite(v)
            if fv is not None:
                cleaned[k] = fv
        if not cleaned:
            continue
        by_type[qtype] = cleaned
        if "answer_correctness" in cleaned:
            accs.append(cleaned["answer_correctness"])
        if "factuality_f1" in cleaned:
            f1s.append(cleaned["factuality_f1"])
        if "embed_cosine" in cleaned:
            coss.append(cleaned["embed_cosine"])
    if not accs:
        return None
    from .judge_tune import judge_tune_pin_fields
    from .profiles import active_pins

    pins = active_pins()
    out: dict[str, Any] = {
        "judge": "generation_eval",
        "judge_model": pins.judge_model,
        "judge_provider": pins.judge_provider,
        "judge_base_url": pins.judge_base_url,
        "judge_embedding_model": pins.judge_embedding_model,
        **judge_tune_pin_fields(),
        "overall_acc": sum(accs) / len(accs),
        "by_type": by_type,
        "raw": raw,
    }
    if f1s:
        out["overall_f1"] = sum(f1s) / len(f1s)
    if coss:
        out["overall_cos"] = sum(coss) / len(coss)
    return out


def _normalize_retrieval(raw: dict[str, Any]) -> dict[str, Any] | None:
    """Flatten retrieval_eval per-type averages into overall + by_type."""
    by_type: dict[str, dict[str, float]] = {}
    rels: list[float] = []
    recalls: list[float] = []
    for qtype, block in raw.items():
        metrics = block.get("average_scores", block) if isinstance(block, dict) else {}
        if not isinstance(metrics, dict):
            continue
        cleaned: dict[str, float] = {}
        for k, v in metrics.items():
            fv = _finite(v)
            if fv is not None:
                cleaned[k] = fv
        if not cleaned:
            continue
        by_type[qtype] = cleaned
        if "context_relevancy" in cleaned:
            rels.append(cleaned["context_relevancy"])
        if "evidence_recall" in cleaned:
            recalls.append(cleaned["evidence_recall"])
    if not by_type:
        return None
    return {
        "judge": "retrieval_eval",
        "overall_context_relevancy": (sum(rels) / len(rels)) if rels else None,
        "overall_evidence_recall": (sum(recalls) / len(recalls)) if recalls else None,
        "by_type": by_type,
        "raw": raw,
    }


def score_predictions(
    predictions: list[dict[str, Any]],
    *,
    eval_out: Path,
    prefer_official: bool = True,
    accept_proxy: bool = False,
    run_retrieval: bool | None = None,
) -> tuple[dict[str, Any], bool]:
    """
    Returns (metrics_block, used_official_generation).

    When ``run_retrieval`` is True (default for live runs), also attaches L2
    ``retrieval`` metrics from official ``retrieval_eval``.

    First principles (Jul 2026): L0/L1 generation_eval and L2 retrieval_eval are
    independent given predictions — run them in parallel to cut wall time.
    """
    from concurrent.futures import ThreadPoolExecutor

    if run_retrieval is None:
        run_retrieval = prefer_official and not accept_proxy

    ret_out = eval_out.with_name(
        eval_out.name.replace("eval_", "retrieval_", 1)
        if "eval_" in eval_out.name
        else eval_out.stem + "_retrieval.json"
    )

    metrics: dict[str, Any] | None = None
    used_official = False
    retrieval: dict[str, Any] | None = None

    if prefer_official and run_retrieval:
        # Gen ∥ retrieval — biggest judgment-phase wall-time win.
        with ThreadPoolExecutor(max_workers=2) as pool:
            fut_gen = pool.submit(
                try_official_generation_eval, predictions, output_file=eval_out
            )
            fut_ret = pool.submit(
                try_official_retrieval_eval, predictions, output_file=ret_out
            )
            official = fut_gen.result()
            retrieval = fut_ret.result()
        if official is not None:
            metrics = official
            used_official = True
    elif prefer_official:
        official = try_official_generation_eval(predictions, output_file=eval_out)
        if official is not None:
            metrics = official
            used_official = True
    elif run_retrieval:
        retrieval = try_official_retrieval_eval(predictions, output_file=ret_out)

    if metrics is None:
        local = score_predictions_local(predictions)
        metrics = local

    if run_retrieval:
        if retrieval is not None:
            metrics["retrieval"] = {
                "overall_context_relevancy": retrieval.get("overall_context_relevancy"),
                "overall_evidence_recall": retrieval.get("overall_evidence_recall"),
                "by_type": retrieval.get("by_type") or {},
            }
            metrics["l2_retrieval"] = True
            ret_out.write_text(json.dumps(retrieval, indent=2), encoding="utf-8")
        else:
            metrics["l2_retrieval"] = False

    eval_out.write_text(json.dumps(metrics, indent=2), encoding="utf-8")
    return metrics, used_official
