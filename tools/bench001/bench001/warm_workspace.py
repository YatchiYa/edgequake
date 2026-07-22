"""Resolve / persist warm EQ workspace for ``make bench-warm``.

Preference order:
1. ``BENCH001_EQ_WORKSPACE_ID`` (explicit)
2. ``e2e/artifacts/warm_workspace.json`` (latest successful full-corpus pointer)
3. Newest ``history/smoke-*/meta.json`` with ``eq_workspace_id`` + full corpus
4. ``e2e/artifacts/smoke/eq_workspace.json``
5. Newest ``ABLATION_NOTE.md`` ``Warm workspace:`` line
"""

from __future__ import annotations

import json
import os
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

from .paths import ARTIFACTS_DIR
from .progress import history_root

WARM_POINTER = ARTIFACTS_DIR / "warm_workspace.json"
_UUID_RE = re.compile(
    r"\b([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})\b",
    re.I,
)
_ABLATION_WS_RE = re.compile(
    r"Warm workspace:\**\s*`?([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})`?",
    re.I,
)


def _is_full_corpus(meta: dict[str, Any]) -> bool:
    """True when ingest was not Acc-capped (publication full corpus)."""
    ingest = meta.get("ingest") or {}
    if ingest.get("ingest_capped"):
        return False
    max_chars = ingest.get("ingest_max_chars")
    if max_chars is None:
        # Also check top-level / env-style fields
        max_chars = meta.get("ingest_max_chars")
    if max_chars in (None, "", 0, "0"):
        # Reject capped workspace names
        name = str(meta.get("eq_workspace_name") or "")
        if "c100000" in name or "c10000" in name or "c250000" in name:
            return False
        return True
    try:
        return int(max_chars) <= 0
    except (TypeError, ValueError):
        return False


def persist_warm_workspace(
    workspace_id: str,
    *,
    stage_dir: Path | None = None,
    archive_dir: Path | None = None,
    meta_extra: dict[str, Any] | None = None,
    full_corpus: bool = True,
) -> Path:
    """Write stage + global warm pointer after a successful EQ ingest/query."""
    wid = (workspace_id or "").strip()
    if not wid:
        raise ValueError("workspace_id required")
    payload: dict[str, Any] = {
        "workspace_id": wid,
        "full_corpus": bool(full_corpus),
        "updated_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    }
    if meta_extra:
        payload.update(meta_extra)
    if archive_dir is not None:
        payload["archive"] = str(archive_dir)
    if stage_dir is not None:
        payload["stage"] = str(stage_dir)

    ARTIFACTS_DIR.mkdir(parents=True, exist_ok=True)
    WARM_POINTER.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    eq_blob = {
        "workspace_id": wid,
        "full_corpus": bool(full_corpus),
        "updated_at_utc": payload["updated_at_utc"],
    }
    for d in (stage_dir, archive_dir):
        if d is None:
            continue
        d.mkdir(parents=True, exist_ok=True)
        (d / "eq_workspace.json").write_text(json.dumps(eq_blob, indent=2), encoding="utf-8")
        meta_path = d / "meta.json"
        if meta_path.exists():
            try:
                meta = json.loads(meta_path.read_text(encoding="utf-8"))
                if isinstance(meta, dict):
                    meta["eq_workspace_id"] = wid
                    meta_path.write_text(json.dumps(meta, indent=2), encoding="utf-8")
            except Exception:  # noqa: BLE001
                pass
    return WARM_POINTER


def _read_ws_file(path: Path) -> str | None:
    if not path.is_file():
        return None
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception:  # noqa: BLE001
        return None
    if not isinstance(data, dict):
        return None
    wid = (data.get("workspace_id") or data.get("eq_workspace_id") or "").strip()
    return wid or None


def _from_warm_pointer() -> str | None:
    if not WARM_POINTER.is_file():
        return None
    try:
        data = json.loads(WARM_POINTER.read_text(encoding="utf-8"))
    except Exception:  # noqa: BLE001
        return None
    if not isinstance(data, dict):
        return None
    # Prefer full-corpus pointer; still allow if flag missing (legacy).
    if data.get("full_corpus") is False:
        return None
    wid = (data.get("workspace_id") or "").strip()
    return wid or None


def _from_history_meta() -> str | None:
    """Newest smoke archive with eq_workspace_id and full corpus."""
    root = history_root()
    # Sort by directory name (timestamp suffix) descending.
    dirs = sorted(
        [p for p in root.iterdir() if p.is_dir() and p.name.startswith("smoke-")],
        key=lambda p: p.name,
        reverse=True,
    )
    for d in dirs:
        if d.name.startswith("smoke-fast") or d.name.startswith("smoke-dry"):
            continue
        meta_path = d / "meta.json"
        if not meta_path.is_file():
            continue
        try:
            meta = json.loads(meta_path.read_text(encoding="utf-8"))
        except Exception:  # noqa: BLE001
            continue
        if not isinstance(meta, dict):
            continue
        wid = (meta.get("eq_workspace_id") or "").strip()
        if not wid:
            wid = _read_ws_file(d / "eq_workspace.json") or ""
        if not wid:
            continue
        # Prefer publication / non-capped
        if not _is_full_corpus(meta) and meta.get("publication") not in {"1", "true", True}:
            # Still accept if scorecard valid and name is bench001-smoke
            name = str(meta.get("eq_workspace_name") or "")
            if name != "bench001-smoke":
                continue
        sc_path = d / "scorecard.json"
        if sc_path.is_file():
            try:
                sc = json.loads(sc_path.read_text(encoding="utf-8"))
                if sc.get("valid") is False:
                    continue
            except Exception:  # noqa: BLE001
                pass
        return wid
    return None


def _from_ablation_notes() -> str | None:
    root = history_root()
    notes = sorted(root.glob("smoke-*/ABLATION_NOTE.md"), reverse=True)
    for note in notes:
        if "smoke-fast" in note.parent.name or "dry" in note.parent.name:
            continue
        try:
            text = note.read_text(encoding="utf-8")
        except Exception:  # noqa: BLE001
            continue
        m = _ABLATION_WS_RE.search(text)
        if m:
            return m.group(1)
        m2 = _UUID_RE.search(text)
        if m2 and "workspace" in text.lower():
            return m2.group(1)
    return None


def resolve_warm_workspace_id(*, prefer_env: bool = True) -> str | None:
    """Return best warm full-corpus workspace id, or None."""
    if prefer_env:
        env = (os.environ.get("BENCH001_EQ_WORKSPACE_ID") or "").strip()
        if env and "c100000" not in env and "c10000" not in env:
            return env
    for finder in (
        _from_warm_pointer,
        _from_history_meta,
        lambda: _read_ws_file(ARTIFACTS_DIR / "smoke" / "eq_workspace.json"),
        _from_ablation_notes,
    ):
        wid = finder()
        if wid:
            return wid
    return None


def resolve_or_raise() -> str:
    wid = resolve_warm_workspace_id()
    if not wid:
        raise SystemExit(
            "No warm EQ workspace found. Run `make bench` once (cold ingest), "
            "or export BENCH001_EQ_WORKSPACE_ID=<uuid>."
        )
    return wid
