"""W0 retrieval diagnostics — code-is-law metrics from API `sources` + `stats`.

SOLID:
- Single responsibility: page-hit math vs response parsing vs aggregation.
- Open/closed: prefer engine `stats.context_empty` when present; fall back only
  when older servers omit the field (no Acc-chasing heuristics).

DRY:
- One `DEFAULT_KS`, one page parser, one chunk filter.

Law (EdgeQuake):
- Pages: `SourceReference.page_start` (chunk), not `page_id`.
- Gold `evidence_pages` compared offline only (never fed to retrieve).
- Engine `QueryStats.context_empty` / `arms_run` / `arm_*_chunks` are SSOT when
  projected by `query_stats_mapper::from_engine_stats` (SPEC-047 W0b).
"""

from __future__ import annotations

from typing import Any, Iterable, Sequence

from .subset import parse_list_field

DEFAULT_KS: tuple[int, ...] = (1, 3, 5, 10)
_NON_CHUNK_TYPES = frozenset({"entity", "relationship"})
_CHUNK_TYPES = frozenset({"chunk", "document_chunk"})


def as_int_pages(raw: Any) -> list[int]:
    """Normalize gold or retrieved page lists to ints."""
    if raw is None:
        return []
    if isinstance(raw, list):
        out: list[int] = []
        for x in raw:
            try:
                out.append(int(x))
            except (TypeError, ValueError):
                continue
        return out
    return [int(x) for x in parse_list_field(str(raw)) if str(x).strip()]


def chunk_sources(sources: Sequence[dict[str, Any]] | None) -> list[dict[str, Any]]:
    """Keep only chunk sources (entities/relationships have no page_start)."""
    if not sources:
        return []
    out: list[dict[str, Any]] = []
    for s in sources:
        st = (s.get("source_type") or "").lower()
        if st in _NON_CHUNK_TYPES:
            continue
        if st in _CHUNK_TYPES or s.get("page_start") is not None:
            out.append(s)
        elif st == "" and (s.get("snippet") is not None or s.get("id") is not None):
            out.append(s)
    return out


def retrieved_pages_ordered(sources: Sequence[dict[str, Any]] | None) -> list[int]:
    """Ordered unique page_start values from chunk sources (retrieval rank order)."""
    pages: list[int] = []
    seen: set[int] = set()
    for s in chunk_sources(sources):
        p = s.get("page_start")
        if p is None:
            continue
        try:
            pi = int(p)
        except (TypeError, ValueError):
            continue
        if pi not in seen:
            seen.add(pi)
            pages.append(pi)
    return pages


def page_hit_at_k(gold_pages: Iterable[int], retrieved_pages: Sequence[int], k: int) -> bool:
    """True iff any gold page appears in the first k retrieved pages."""
    gold = set(int(p) for p in gold_pages)
    if not gold:
        return False
    return bool(gold & set(retrieved_pages[:k]))


def page_recall_at_k(gold_pages: Iterable[int], retrieved_pages: Sequence[int], k: int) -> float:
    gold = set(int(p) for p in gold_pages)
    if not gold:
        return 0.0
    return len(gold & set(retrieved_pages[:k])) / len(gold)


def resolve_context_empty(resp: dict[str, Any]) -> bool:
    """Prefer engine `stats.context_empty`; else derive from sources (compat)."""
    stats = resp.get("stats") or {}
    if "context_empty" in stats and stats["context_empty"] is not None:
        return bool(stats["context_empty"])
    n = stats.get("sources_retrieved")
    if n is not None and int(n) == 0:
        return True
    chunks = chunk_sources(resp.get("sources"))
    if not chunks:
        return True
    return not any(str(c.get("snippet") or "").strip() for c in chunks)


def extract_arm_stats(stats: dict[str, Any]) -> dict[str, Any]:
    """Project Hybrid/Mix arm fields from HTTP QueryStats (W0b)."""
    keys = (
        "arms_run",
        "arms_gated",
        "arm_local_ms",
        "arm_global_ms",
        "arm_naive_ms",
        "arm_local_chunks",
        "arm_global_chunks",
        "arm_naive_chunks",
        "context_truncated",
    )
    return {k: stats[k] for k in keys if k in stats and stats[k] is not None}


def build_retrieval_diagnostics(
    resp: dict[str, Any],
    *,
    evidence_pages: Any,
    ks: Sequence[int] = DEFAULT_KS,
) -> dict[str, Any]:
    """Build W0 diagnostic block for one prediction row."""
    sources = resp.get("sources") or []
    stats = resp.get("stats") or {}
    chunks = chunk_sources(sources)
    pages = retrieved_pages_ordered(sources)
    gold = as_int_pages(evidence_pages)
    gold_set = set(gold)

    out: dict[str, Any] = {
        "retrieved_chunk_ids": [str(c.get("id")) for c in chunks if c.get("id") is not None],
        "retrieved_pages": pages,
        "retrieved_document_ids": sorted(
            {str(c.get("document_id")) for c in chunks if c.get("document_id")}
        ),
        "n_chunk_sources": len(chunks),
        "n_sources_total": len(sources),
        "context_empty": resolve_context_empty(resp),
        "gold_evidence_pages": gold,
        "gold_page_in_retrieved": sorted(gold_set & set(pages)),
        "stats_sources_retrieved": stats.get("sources_retrieved"),
        "stats_retrieval_time_ms": stats.get("retrieval_time_ms"),
    }
    for k in ks:
        out[f"page_hit@{k}"] = page_hit_at_k(gold, pages, k)
        out[f"page_recall@{k}"] = page_recall_at_k(gold, pages, k)
    out.update(extract_arm_stats(stats))
    return out


def aggregate_page_hit_metrics(
    samples: list[dict[str, Any]],
    *,
    ks: Sequence[int] = DEFAULT_KS,
    answerable_only: bool = True,
) -> dict[str, Any]:
    """Aggregate page_hit@k over samples that carry retrieval diagnostics."""
    rows: list[dict[str, Any]] = []
    for s in samples:
        diag = s.get("retrieval") or {}
        if not diag and "page_hit@5" not in s:
            continue
        if answerable_only and not is_answerable_gold(s):
            continue
        rows.append(diag if diag else s)

    if not rows:
        return {"n_with_retrieval_diag": 0, "answerable_only": answerable_only}

    out: dict[str, Any] = {
        "n_with_retrieval_diag": len(rows),
        "answerable_only": answerable_only,
        "context_empty_rate": sum(1 for r in rows if r.get("context_empty")) / len(rows),
        "mean_n_chunk_sources": sum(int(r.get("n_chunk_sources") or 0) for r in rows)
        / len(rows),
    }
    for k in ks:
        key = f"page_hit@{k}"
        vals = [bool(r.get(key)) for r in rows if key in r]
        out[key] = (sum(vals) / len(vals)) if vals else None
        rkey = f"page_recall@{k}"
        rvals = [float(r[rkey]) for r in rows if r.get(rkey) is not None]
        out[rkey] = (sum(rvals) / len(rvals)) if rvals else None

    # Arm contribution averages (when W0b fields present)
    for arm in ("local", "global", "naive"):
        ckey = f"arm_{arm}_chunks"
        cvals = [int(r[ckey]) for r in rows if r.get(ckey) is not None]
        if cvals:
            out[f"mean_{ckey}"] = sum(cvals) / len(cvals)

    return out


# --- SPEC-047 / 020 A2 + B1: refusal + arm-gate honesty ---------------------------

_NOT_ANSWERABLE_ALIASES = frozenset(
    {
        "not answerable",
        "not answerable.",
        "insufficient evidence",
        "cannot find it in the provided context",
        "i cannot find it in the provided context",
    }
)


def is_not_answerable_pred(pred: Any) -> bool:
    """True if short-answer prediction is a refusal / Not answerable."""
    s = str(pred or "").strip().lower()
    if not s:
        return False
    if s in _NOT_ANSWERABLE_ALIASES:
        return True
    return s.startswith("not answerable")


def is_answerable_gold(sample: dict[str, Any]) -> bool:
    """True if gold is answerable (MMLongBench: answer != 'Not answerable')."""
    return str(sample.get("answer") or "").strip() != "Not answerable"


def _page_hit5(sample: dict[str, Any]) -> bool | None:
    diag = sample.get("retrieval") or {}
    if "page_hit@5" in diag:
        return bool(diag.get("page_hit@5"))
    if "page_hit@5" in sample:
        return bool(sample.get("page_hit@5"))
    return None


def is_false_refusal(sample: dict[str, Any]) -> bool:
    """020 A2: answerable gold ∧ pred≈Not answerable."""
    return is_answerable_gold(sample) and is_not_answerable_pred(sample.get("pred"))


def aggregate_false_refusal_metrics(samples: list[dict[str, Any]]) -> dict[str, Any]:
    """Aggregate false-refusal rates (020 A2).

    - `false_refusal_rate`: among answerable golds
    - `false_refusal_given_page_hit@5`: among answerable ∧ page_hit@5
    """
    answerable = [s for s in samples if is_answerable_gold(s)]
    n_ans = len(answerable)
    n_fr = sum(1 for s in answerable if is_false_refusal(s))

    hit_rows = [s for s in answerable if _page_hit5(s) is True]
    n_hit = len(hit_rows)
    n_fr_hit = sum(1 for s in hit_rows if is_false_refusal(s))

    return {
        "n_answerable": n_ans,
        "n_false_refusal": n_fr,
        "false_refusal_rate": (n_fr / n_ans) if n_ans else None,
        "n_answerable_page_hit@5": n_hit,
        "n_false_refusal_page_hit@5": n_fr_hit,
        "false_refusal_given_page_hit@5": (n_fr_hit / n_hit) if n_hit else None,
    }


def _arms_run_tokens(arms_run: Any) -> set[str]:
    """Normalize `arms_run` (str CSV or list) to a token set."""
    if arms_run is None:
        return set()
    if isinstance(arms_run, list):
        return {str(x).strip().lower() for x in arms_run if str(x).strip()}
    s = str(arms_run).strip().lower()
    if not s or s == "none":
        return set()
    return {p.strip() for p in s.replace("|", ",").split(",") if p.strip()}


def aggregate_arm_gate_metrics(samples: list[dict[str, Any]]) -> dict[str, Any]:
    """020 B1/B2: planned arms (`arms_run`) vs productive chunk counts.

    - `*_present_rate` / `naive_only_rate`: arms that returned chunks (productivity).
    - `planned_*`: what the engine *scheduled* (gate honesty). B2 can pass planned
      while local still returns 0 chunks (empty entity neighborhood).
    """
    rows: list[dict[str, Any]] = []
    for s in samples:
        diag = s.get("retrieval") or {}
        if "arms_run" in diag or "arms_gated" in diag:
            rows.append(diag)
    if not rows:
        return {"n_with_arm_diag": 0}

    n = len(rows)
    gated = sum(1 for r in rows if r.get("arms_gated") is True)
    has_local = sum(1 for r in rows if int(r.get("arm_local_chunks") or 0) > 0)
    has_global = sum(1 for r in rows if int(r.get("arm_global_chunks") or 0) > 0)
    has_graph = sum(
        1
        for r in rows
        if int(r.get("arm_local_chunks") or 0) > 0 or int(r.get("arm_global_chunks") or 0) > 0
    )
    naive_only = sum(
        1
        for r in rows
        if int(r.get("arm_naive_chunks") or 0) > 0
        and int(r.get("arm_local_chunks") or 0) == 0
        and int(r.get("arm_global_chunks") or 0) == 0
    )

    planned = [_arms_run_tokens(r.get("arms_run")) for r in rows]
    n_planned = sum(1 for t in planned if t)
    planned_local = sum(1 for t in planned if "local" in t)
    planned_global = sum(1 for t in planned if "global" in t)
    planned_graph = sum(1 for t in planned if "local" in t or "global" in t)
    planned_naive_only = sum(
        1 for t in planned if t == {"naive"} or t == {"naive_only"}
    )

    out: dict[str, Any] = {
        "n_with_arm_diag": n,
        "arms_gated_rate": gated / n,
        "arm_local_present_rate": has_local / n,
        "arm_global_present_rate": has_global / n,
        "arm_graph_present_rate": has_graph / n,
        "naive_only_rate": naive_only / n,
    }
    if n_planned:
        out.update(
            {
                "n_with_arms_run": n_planned,
                "planned_local_rate": planned_local / n_planned,
                "planned_global_rate": planned_global / n_planned,
                "planned_graph_rate": planned_graph / n_planned,
                "planned_naive_only_rate": planned_naive_only / n_planned,
            }
        )
    return out


# Back-compat aliases (avoid breaking older imports / tests)
_as_int_pages = as_int_pages
context_empty_from_response = resolve_context_empty
