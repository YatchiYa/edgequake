#!/usr/bin/env python3
"""SPEC-134 Slice E page-as-unit ablation.

Renders each PDF page at several long-edges, optionally JPEG-encodes, and
calls an OpenAI-compatible vision chat endpoint with print vs manuscript
prompts. Never prints or writes trigger-document prose into the repo:
outputs stay under study/out/ (gitignored).

Env:
  SPEC134_STUDY_PDF          path to private PDF (required)
  SPEC134_STUDY_GOLD         optional JSON {"anchors": ["..."]}
  OPENAI_API_KEY / EDGEQUAKE_LLM_API_KEY
  OPENAI_BASE_URL / EDGEQUAKE_LLM_BASE_URL  (default https://api.openai.com/v1)
  EDGEQUAKE_VISION_MODEL / OPENAI_MODEL     (default gpt-4.1-mini)
"""

from __future__ import annotations

import base64
import io
import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
OUT = ROOT / "out"


def load_dotenv_files() -> None:
    """Fill unset env from gitignored .env files (never override a live export)."""
    for path in (ROOT / ".env", REPO / ".env"):
        if not path.is_file():
            continue
        for raw in path.read_text(encoding="utf-8").splitlines():
            line = raw.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            if line.startswith("export "):
                line = line[len("export ") :]
            key, _, val = line.partition("=")
            key = key.strip()
            val = val.strip().strip("'").strip('"')
            if key and key not in os.environ:
                os.environ[key] = val

PRINT_PROMPT = (
    "You are an expert document converter for RAG. "
    "Write ALL output in English (translate labels if needed). "
    "Output ONLY Markdown."
)

MS_PROMPT = (
    "Transcribe this page faithfully in the SAME LANGUAGE as the source. "
    "Do not translate. Unreadables as [?]. Never invent. "
    "Implicit tables as GFM. Output ONLY Markdown."
)


def _env(name: str, default: str | None = None) -> str | None:
    v = os.environ.get(name, "").strip()
    return v if v else default


def render_pages(pdf_path: Path, long_edge: int) -> list[bytes]:
    """Return PNG bytes per page, scaled so max(w,h) == long_edge."""
    try:
        import fitz  # PyMuPDF
    except ImportError:
        sys.stderr.write("PyMuPDF (fitz) is required: pip install pymupdf\n")
        raise
    doc = fitz.open(pdf_path)
    pages: list[bytes] = []
    for page in doc:
        rect = page.rect
        scale = long_edge / max(rect.width, rect.height)
        mat = fitz.Matrix(scale, scale)
        pix = page.get_pixmap(matrix=mat, alpha=False)
        pages.append(pix.tobytes("png"))
    doc.close()
    return pages


def to_jpeg(png: bytes, quality: int = 85) -> bytes:
    from PIL import Image

    im = Image.open(io.BytesIO(png)).convert("RGB")
    buf = io.BytesIO()
    im.save(buf, format="JPEG", quality=quality)
    return buf.getvalue()


def crop_gallery(png: bytes, tiles: int = 4) -> list[bytes]:
    """Naive 2x2 (or tiles) crops — stands in for scan-tile theater."""
    from PIL import Image

    im = Image.open(io.BytesIO(png)).convert("RGB")
    w, h = im.size
    cols = 2
    rows = max(1, tiles // cols)
    out: list[bytes] = []
    tw, th = w // cols, h // rows
    for r in range(rows):
        for c in range(cols):
            box = (c * tw, r * th, (c + 1) * tw if c < cols - 1 else w, (r + 1) * th if r < rows - 1 else h)
            crop = im.crop(box)
            buf = io.BytesIO()
            crop.save(buf, format="PNG")
            out.append(buf.getvalue())
    return out


def chat_vision(prompt: str, images: list[tuple[bytes, str]], model: str, base: str, key: str) -> str:
    content: list[dict] = [{"type": "text", "text": prompt}]
    for data, mime in images:
        b64 = base64.b64encode(data).decode("ascii")
        content.append(
            {
                "type": "image_url",
                "image_url": {"url": f"data:{mime};base64,{b64}"},
            }
        )
    body = json.dumps(
        {
            "model": model,
            "messages": [{"role": "user", "content": content}],
            "max_tokens": 4096,
            "temperature": 0,
        }
    ).encode()
    req = urllib.request.Request(
        base.rstrip("/") + "/chat/completions",
        data=body,
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {key}",
        },
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=180) as resp:
            payload = json.loads(resp.read().decode())
    except urllib.error.HTTPError as e:
        detail = e.read()[:240].decode("utf-8", "replace").replace("\n", " ")
        return f"[http_error {e.code}] {detail}"
    except Exception as e:  # noqa: BLE001 — study harness
        return f"[error {type(e).__name__}: {e}]"
    try:
        return payload["choices"][0]["message"]["content"] or ""
    except (KeyError, IndexError, TypeError):
        return "[unparseable]"


def is_empty(text: str) -> bool:
    t = text.strip().lower()
    if not t or t.startswith("[http_error") or t.startswith("[error") or t.startswith("[unparseable"):
        return True
    if "no text extracted" in t:
        return True
    return len(t) < 40


def frenchish(text: str) -> bool:
    hits = sum(1 for w in ("le", "la", "les", "des", "une", "pas", "pour", "dans") if f" {w} " in f" {text.lower()} ")
    return hits >= 2


def recall(text: str, anchors: list[str]) -> float:
    if not anchors:
        return 0.0
    low = text.lower()
    hit = sum(1 for a in anchors if a.lower() in low)
    return hit / len(anchors)


def main() -> int:
    load_dotenv_files()
    pdf = Path(_env("SPEC134_STUDY_PDF") or "")
    if not pdf.is_file():
        sys.stderr.write("Set SPEC134_STUDY_PDF to a readable PDF.\n")
        return 2
    gold_path = Path(_env("SPEC134_STUDY_GOLD") or str(ROOT / "gold.local.json"))
    anchors: list[str] = []
    if gold_path.is_file():
        try:
            anchors = list(json.loads(gold_path.read_text()).get("anchors") or [])
        except json.JSONDecodeError:
            anchors = []
    key = _env("OPENAI_API_KEY") or _env("EDGEQUAKE_LLM_API_KEY") or ""
    dry = not key
    base = _env("OPENAI_BASE_URL") or _env("EDGEQUAKE_LLM_BASE_URL") or "https://api.openai.com/v1"
    model = (
        _env("EDGEQUAKE_VISION_MODEL")
        or _env("EDGEQUAKE_VISION_LLM_MODEL")
        or _env("OPENAI_MODEL")
        or "gpt-4.1-nano"
    )
    OUT.mkdir(parents=True, exist_ok=True)

    conditions = [
        ("px1024_png_ms", 1024, "png", "ms", "page"),
        ("px2000_png_ms", 2000, "png", "ms", "page"),
        ("px3600_png_ms", 3600, "png", "ms", "page"),
        ("px3600_jpg_ms", 3600, "jpeg", "ms", "page"),
        ("px3600_png_print", 3600, "png", "print", "page"),
        ("px2000_png_ms_crops", 2000, "png", "ms", "crops"),
    ]

    summary: list[dict] = []
    for name, long_edge, fmt, prompt_kind, mode in conditions:
        prompt = MS_PROMPT if prompt_kind == "ms" else PRINT_PROMPT
        pngs = render_pages(pdf, long_edge)
        max_pages = int(_env("SPEC134_STUDY_MAX_PAGES") or "0") or len(pngs)
        pngs = pngs[:max_pages]
        empty = 0
        rec = 0.0
        fr = 0
        n = len(pngs)
        for i, png in enumerate(pngs, start=1):
            blob = to_jpeg(png) if fmt == "jpeg" else png
            mime = "image/jpeg" if fmt == "jpeg" else "image/png"
            if mode == "crops":
                images = [(c, "image/png") for c in crop_gallery(png)]
            else:
                images = [(blob, mime)]
            if dry:
                text = "[dry-run]"
            else:
                text = chat_vision(prompt, images, model, base, key)
            page_out = OUT / f"{name}_p{i}.md"
            page_out.write_text(text, encoding="utf-8")
            if is_empty(text) or text == "[dry-run]":
                empty += 1
            else:
                rec += recall(text, anchors)
                if frenchish(text):
                    fr += 1
        row = {
            "condition": name,
            "long_edge": long_edge,
            "format": fmt,
            "prompt": prompt_kind,
            "mode": mode,
            "pages": n,
            "empty_rate": empty / n if n else 1.0,
            "anchor_recall": (rec / n) if n and anchors else None,
            "frenchish_pages": fr,
            "dry_run": dry,
        }
        summary.append(row)
        print(json.dumps(row), flush=True)

    (OUT / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")
    go = False
    if not dry:
        by = {r["condition"]: r for r in summary}
        hi = by.get("px3600_png_ms") or by.get("px2000_png_ms")
        lo = by.get("px1024_png_ms")
        crops = by.get("px2000_png_ms_crops")
        go = bool(
            hi
            and hi["empty_rate"] < 0.5
            and lo
            and lo["empty_rate"] >= hi["empty_rate"]
            and crops
            and crops["empty_rate"] >= hi["empty_rate"]
        )
    (OUT / "go_nogo.json").write_text(
        json.dumps({"go_for_rust": go, "dry_run": dry, "model": model}, indent=2),
        encoding="utf-8",
    )
    print(f"wrote {OUT / 'summary.json'} go={go} dry={dry}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
