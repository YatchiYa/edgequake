#!/usr/bin/env python3
"""SPEC-115 geometry probe — real LightRAG F chunker on gold MD / PDF meta.

No LLM. Writes measurements/geometry_results.json + geometry_table.md
"""

from __future__ import annotations

import json
import sys
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[3]
LR_ROOT = Path("/Users/raphaelmansuy/Github/03-working/LightRAG")
OUT_DIR = Path(__file__).resolve().parents[1] / "measurements"
GOLD = REPO / "zz_test_docs/academic_papers/lighrag_2410.05779v3.pymupdf.gold.md"
PDF = REPO / "papers/light_rag_2410.05779v3.pdf"

sys.path.insert(0, str(LR_ROOT))


def eq_adaptive_size(n_bytes: int) -> tuple[int, int]:
    if n_bytes > 100_000:
        size = 600
    elif n_bytes > 50_000:
        size = 800
    else:
        size = 1200
    overlap = int(size * 0.083)
    return size, overlap


def main() -> None:
    from lightrag.chunker import chunking_by_token_size
    from lightrag.utils import TiktokenTokenizer

    text = GOLD.read_text(encoding="utf-8")
    tok = TiktokenTokenizer()
    doc_tokens = len(tok.encode(text))
    pdf_bytes = PDF.stat().st_size if PDF.exists() else None

    rows = []
    for size, ov in [(1200, 100), (800, 66), (600, 50)]:
        chunks = chunking_by_token_size(
            tok,
            text,
            chunk_token_size=size,
            chunk_overlap_token_size=ov,
        )
        sizes = [len(tok.encode(c["content"])) for c in chunks]
        rows.append(
            {
                "label": f"F@{size}/{ov}",
                "chunk_token_size": size,
                "overlap": ov,
                "N": len(chunks),
                "token_min": min(sizes),
                "token_avg": round(sum(sizes) / len(sizes), 1),
                "token_max": max(sizes),
            }
        )

    prod_size, prod_ov = eq_adaptive_size(len(text.encode("utf-8")))
    pdf_pin = eq_adaptive_size(pdf_bytes)[0] if pdf_bytes else None

    payload = {
        "utc": datetime.now(timezone.utc).isoformat(),
        "sample": str(GOLD.relative_to(REPO)),
        "chars": len(text),
        "utf8_bytes": len(text.encode("utf-8")),
        "doc_tokens_tiktoken": doc_tokens,
        "pdf_path": str(PDF.relative_to(REPO)) if PDF.exists() else None,
        "pdf_bytes": pdf_bytes,
        "eq_adaptive_pin_on_text_bytes": {"size": prod_size, "overlap": prod_ov},
        "eq_adaptive_pin_if_keyed_on_pdf_bytes": pdf_pin,
        "note": "EQ ingest uses text_content.len() not PDF file bytes (prepare.rs).",
        "lightrag_F_rows": rows,
        "verdict_H_C1": {
            "N_fair_1200": next(r["N"] for r in rows if r["chunk_token_size"] == 1200),
            "N_product_800": next(r["N"] for r in rows if r["chunk_token_size"] == 800),
            "ratio": round(
                next(r["N"] for r in rows if r["chunk_token_size"] == 800)
                / next(r["N"] for r in rows if r["chunk_token_size"] == 1200),
                3,
            ),
        },
    }

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    (OUT_DIR / "geometry_results.json").write_text(
        json.dumps(payload, indent=2) + "\n", encoding="utf-8"
    )

    lines = [
        "# Geometry results (SPEC-115)",
        "",
        f"- UTC: `{payload['utc']}`",
        f"- Sample: `{payload['sample']}`",
        f"- chars={payload['chars']} utf8_bytes={payload['utf8_bytes']} "
        f"tiktoken={payload['doc_tokens_tiktoken']}",
        f"- PDF bytes={pdf_bytes} (adaptive-if-wrong-key→{pdf_pin}); "
        f"text adaptive pin→{prod_size}/{prod_ov}",
        "",
        "| Pin | N | min | avg | max |",
        "|-----|--:|----:|----:|----:|",
    ]
    for r in rows:
        lines.append(
            f"| {r['label']} | {r['N']} | {r['token_min']} | {r['token_avg']} | {r['token_max']} |"
        )
    v = payload["verdict_H_C1"]
    lines += [
        "",
        f"**H-C1:** N_product/N_fair = {v['N_product_800']}/{v['N_fair_1200']} = **{v['ratio']}**",
        "",
    ]
    (OUT_DIR / "geometry_table.md").write_text("\n".join(lines), encoding="utf-8")
    print("\n".join(lines))


if __name__ == "__main__":
    main()
