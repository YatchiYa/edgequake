"""Corpus ingest caps for fast, reliable Acc iteration (SPEC-001).

Full GraphRAG-Bench medical context is ~1.05MB → ~188 chunks @1200 → tens of
thousands of entities. Graph merge can take >1h and fail (duplicate-key /
compensation wipe). Smoke-fast Acc force-ingest should use a capped slice so
operators get progress + a valid index in minutes, not hours.

Set ``BENCH001_INGEST_MAX_CHARS=0`` (or unset with no Makefile default) for the
full corpus. Acc Makefile defaults smoke-fast to 100_000 chars (~25–35 chunks).
"""

from __future__ import annotations

import os
from typing import Any


# ~100k chars ≈ 25–35 chunks at chunk_token_size=1200 (with overlap) — enough for
# smoke-fast n=8 relevance checks without hour-long merge.
DEFAULT_SMOKE_FAST_INGEST_MAX_CHARS = 100_000


def ingest_max_chars() -> int | None:
    """Return max chars per corpus blob, or None for unlimited.

    ``BENCH001_INGEST_MAX_CHARS=0`` / empty / ``full`` → unlimited.
    """
    raw = (os.environ.get("BENCH001_INGEST_MAX_CHARS") or "").strip().lower()
    if not raw or raw in {"0", "full", "none", "unlimited", "off"}:
        return None
    try:
        n = int(raw)
    except ValueError:
        return None
    return n if n > 0 else None


def apply_ingest_cap(
    texts: list[str],
    *,
    max_chars: int | None = None,
) -> tuple[list[str], dict[str, Any]]:
    """Truncate each corpus text to ``max_chars`` (word-boundary soft cut).

    Returns ``(texts, meta)`` where meta records original/capped sizes for pins.
    """
    cap = ingest_max_chars() if max_chars is None else max_chars
    meta: dict[str, Any] = {
        "ingest_max_chars": cap,
        "ingest_capped": False,
        "corpus_chars_original": [len(t) for t in texts],
        "corpus_chars_effective": [len(t) for t in texts],
    }
    if cap is None:
        return list(texts), meta

    out: list[str] = []
    effective: list[int] = []
    capped = False
    for t in texts:
        if len(t) <= cap:
            out.append(t)
            effective.append(len(t))
            continue
        capped = True
        slice_ = t[:cap]
        # Prefer cutting on whitespace near the end to avoid mid-token junk.
        sp = slice_.rfind(" ")
        if sp > int(cap * 0.8):
            slice_ = slice_[:sp]
        out.append(slice_.rstrip() + "\n")
        effective.append(len(out[-1]))
    meta["ingest_capped"] = capped
    meta["corpus_chars_effective"] = effective
    return out, meta


def lr_stage_for_cap(base_stage: str, *, max_chars: int | None = None) -> str:
    """Isolate LR working dirs when corpus is capped (avoid full-corpus cache)."""
    cap = ingest_max_chars() if max_chars is None else max_chars
    if cap is None:
        return base_stage
    return f"{base_stage}_c{cap}"


def eq_workspace_name_for_cap(base_name: str, *, max_chars: int | None = None) -> str:
    """Isolate EQ workspaces when corpus is capped."""
    cap = ingest_max_chars() if max_chars is None else max_chars
    if cap is None:
        return base_name
    return f"{base_name}-c{cap}"
