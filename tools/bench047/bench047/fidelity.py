"""W1 representation fidelity — is the gold answer in ingested page markdown?

First principle (FP1): if vision/chunking never wrote the answer onto the
evidence page(s), retrieval cannot honestly recover it.

Lawful metric (no Acc inflation):
  answer_in_evidence_pages = gold short answer appears in markdown for gold
  evidence_pages (page-marker split). Offline / diagnostic only.

Uses GET /api/v1/query/context/artifacts/markdown/{document_id}.
"""

from __future__ import annotations

import re
from typing import Any, Optional

from .diagnostics import as_int_pages
from .subset import parse_list_field

PAGE_MARKER_RE = re.compile(r"<!--\s*edgequake-page:(\d+)\s*-->", re.IGNORECASE)


def split_markdown_by_page(markdown: str) -> dict[int, str]:
    """Split page-marked markdown into {page_num: text} (1-indexed)."""
    if not markdown:
        return {}
    parts = PAGE_MARKER_RE.split(markdown)
    # parts: [preamble, page1, text1, page2, text2, ...]
    pages: dict[int, str] = {}
    if len(parts) == 1:
        pages[1] = parts[0]
        return pages
    # preamble before first marker is ignored for page fidelity
    i = 1
    while i + 1 < len(parts):
        try:
            pnum = int(parts[i])
        except ValueError:
            i += 2
            continue
        pages[pnum] = parts[i + 1]
        i += 2
    return pages


def normalize_for_containment(text: str) -> str:
    """Aggressive normalize for number/string containment (not Acc scoring)."""
    t = (text or "").lower()
    t = t.replace("%", "")
    t = re.sub(r"[\s,$_€£¥]+", "", t)
    return t


def answer_in_text(answer: str, text: str) -> bool:
    """True if gold answer is contained in text after normalization.

    Skips unanswerable / empty. Floats: also try without trailing zeros.
    """
    ans = str(answer or "").strip()
    if not ans or ans == "Not answerable":
        return False
    hay = normalize_for_containment(text)
    needle = normalize_for_containment(ans)
    if not needle:
        return False
    if needle in hay:
        return True
    # numeric variants: 18.29 vs 18.290 / 0.1829 handled loosely for %
    try:
        f = float(ans.replace("%", "").replace(",", ""))
        # look for significant digit string
        for cand in (f"{f:g}", f"{f:.2f}", f"{f:.1f}", f"{f:.0f}"):
            if normalize_for_containment(cand) in hay:
                return True
    except ValueError:
        pass
    return False


def pages_containing_answer(answer: str, page_map: dict[int, str]) -> list[int]:
    return sorted(p for p, body in page_map.items() if answer_in_text(answer, body))


def fidelity_for_sample(
    *,
    answer: str,
    evidence_pages: Any,
    markdown: str,
    evidence_sources: Any = None,
) -> dict[str, Any]:
    """Compute representation fidelity for one QA row given full doc markdown."""
    gold_pages = as_int_pages(evidence_pages)
    page_map = split_markdown_by_page(markdown)
    evidence_text = "\n".join(page_map.get(p, "") for p in gold_pages)
    in_evidence = answer_in_text(answer, evidence_text) if gold_pages else False
    in_doc = answer_in_text(answer, markdown)
    found_pages = pages_containing_answer(answer, page_map)
    srcs = evidence_sources
    if not isinstance(srcs, list):
        srcs = parse_list_field(str(srcs or "[]"))
    return {
        "gold_evidence_pages": gold_pages,
        "answer_in_evidence_pages": in_evidence,
        "answer_in_document": in_doc,
        "pages_with_answer": found_pages,
        "evidence_pages_present_in_markdown": [p for p in gold_pages if p in page_map],
        "n_pages_in_markdown": len(page_map),
        "evidence_sources": [str(s) for s in srcs],
    }


def aggregate_fidelity(rows: list[dict[str, Any]]) -> dict[str, Any]:
    """Aggregate W1 fidelity rows (answerable only expected)."""
    if not rows:
        return {"n": 0}
    n = len(rows)
    in_ev = sum(1 for r in rows if r.get("answer_in_evidence_pages"))
    in_doc = sum(1 for r in rows if r.get("answer_in_document"))
    by_src: dict[str, list[bool]] = {}
    for r in rows:
        hit = bool(r.get("answer_in_evidence_pages"))
        for s in r.get("evidence_sources") or ["?"]:
            by_src.setdefault(str(s), []).append(hit)
    return {
        "n": n,
        "answer_in_evidence_rate": in_ev / n,
        "answer_in_document_rate": in_doc / n,
        "by_evidence_source": {
            k: {"rate": sum(v) / len(v), "n": len(v)} for k, v in sorted(by_src.items())
        },
    }
