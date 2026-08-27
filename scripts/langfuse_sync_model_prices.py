#!/usr/bin/env python3
"""Sync EdgeQuake model prices into a Langfuse instance.

WHY
---
Langfuse computes observation cost from **its own** model catalogue, and
EdgeQuake deliberately never emits cost attributes (LAW-124-12: Langfuse stays
the single source of truth for cost). A self-hosted Langfuse therefore only
prices the models its bundled catalogue knows — Langfuse 3.1 ships a 2024 list
with `gpt-4o` but no `gpt-5`, `gemini-1.5` but no `gemini-2.5`, `claude-3.5`
but no `claude-4.x`, and no Mistral at all. Every newer model then costs $0.00.

This script pushes EdgeQuake's own `models.toml` pricing into Langfuse via
`POST /api/public/models`, so cost works for every provider EdgeQuake supports
without touching the export path.

Prices: `models.toml` is per **1k tokens**; Langfuse expects per **token**.

Usage
-----
    python3 scripts/langfuse_sync_model_prices.py \
        --base-url http://localhost:3320 \
        --public-key pk-lf-... --secret-key sk-lf-...

Credentials default to LANGFUSE_BASE_URL / LANGFUSE_PUBLIC_KEY /
LANGFUSE_SECRET_KEY from the environment. `--dry-run` prints without writing.
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import re
import sys
import urllib.error
import urllib.request

try:
    import tomllib  # Python >= 3.11
except ModuleNotFoundError:  # pragma: no cover - older interpreters
    import tomli as tomllib  # type: ignore

DEFAULT_TOML = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "edgequake",
    "models.toml",
)


def api(base_url: str, path: str) -> str:
    return f"{base_url.rstrip('/')}{path}"


def request(url: str, token: str, method: str = "GET", payload: dict | None = None):
    data = json.dumps(payload).encode() if payload is not None else None
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Authorization", f"Basic {token}")
    req.add_header("Content-Type", "application/json")
    with urllib.request.urlopen(req, timeout=30) as resp:
        body = resp.read().decode()
        return resp.status, (json.loads(body) if body.strip() else {})


def existing_model_names(base_url: str, token: str) -> set[str]:
    """Names already priced in Langfuse (paginated)."""
    names: set[str] = set()
    page = 1
    while True:
        try:
            _, data = request(api(base_url, f"/api/public/models?page={page}&limit=100"), token)
        except urllib.error.HTTPError:
            break
        items = data.get("data") or []
        if not items:
            break
        names.update(m.get("modelName", "") for m in items)
        if len(items) < 100:
            break
        page += 1
    return names


def collect_prices(toml_path: str) -> list[dict]:
    """Model → per-token prices, skipping free (local) models."""
    with open(toml_path, "rb") as fh:
        doc = tomllib.load(fh)

    out: list[dict] = []
    seen: set[str] = set()
    for provider in doc.get("providers", []):
        for model in provider.get("models", []):
            name = model.get("name")
            cost = model.get("cost") or {}
            if not name or name in seen:
                continue

            in_1k = float(cost.get("input_per_1k") or 0.0)
            out_1k = float(cost.get("output_per_1k") or 0.0)
            emb_1k = float(cost.get("embedding_per_1k") or 0.0)

            # Embedding models price the input side only.
            if model.get("model_type") == "embedding" or (emb_1k and not in_1k):
                in_1k, out_1k = emb_1k, 0.0

            # Local runtimes (ollama, lmstudio, …) are free — nothing to price.
            if in_1k <= 0 and out_1k <= 0:
                continue

            seen.add(name)
            out.append(
                {
                    "modelName": name,
                    "matchPattern": f"(?i)^({re.escape(name)})$",
                    "unit": "TOKENS",
                    "inputPrice": in_1k / 1000.0,
                    "outputPrice": out_1k / 1000.0,
                }
            )
    return out


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--models-toml", default=DEFAULT_TOML)
    ap.add_argument("--base-url", default=os.environ.get("LANGFUSE_BASE_URL", ""))
    ap.add_argument("--public-key", default=os.environ.get("LANGFUSE_PUBLIC_KEY", ""))
    ap.add_argument("--secret-key", default=os.environ.get("LANGFUSE_SECRET_KEY", ""))
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--force", action="store_true", help="push even if the name already exists")
    args = ap.parse_args()

    missing = [n for n, v in (("--base-url", args.base_url), ("--public-key", args.public_key), ("--secret-key", args.secret_key)) if not v]
    if missing:
        print(f"missing: {', '.join(missing)} (or set LANGFUSE_* env vars)", file=sys.stderr)
        return 2

    models = collect_prices(args.models_toml)
    print(f"{len(models)} priced models in {os.path.relpath(args.models_toml)}")

    if args.dry_run:
        for m in models:
            print(f"  {m['modelName']:38s} in={m['inputPrice']:.10f} out={m['outputPrice']:.10f}")
        return 0

    token = base64.b64encode(f"{args.public_key}:{args.secret_key}".encode()).decode()
    known = set() if args.force else existing_model_names(args.base_url, token)
    if known:
        print(f"{len(known)} already in the Langfuse catalogue — skipping those (use --force to overwrite)")

    created = skipped = failed = 0
    for m in models:
        if m["modelName"] in known:
            skipped += 1
            continue
        try:
            status, _ = request(api(args.base_url, "/api/public/models"), token, "POST", m)
            if 200 <= status < 300:
                created += 1
            else:
                failed += 1
                print(f"  ! {m['modelName']}: HTTP {status}", file=sys.stderr)
        except urllib.error.HTTPError as exc:
            failed += 1
            print(f"  ! {m['modelName']}: HTTP {exc.code} {exc.read()[:120].decode(errors='replace')}", file=sys.stderr)
        except Exception as exc:  # noqa: BLE001 - report and continue
            failed += 1
            print(f"  ! {m['modelName']}: {exc}", file=sys.stderr)

    print(f"created={created} skipped={skipped} failed={failed}")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
