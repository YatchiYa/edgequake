"""W1 representation fidelity — is the gold answer in ingested page markdown?

First principle (FP1): if vision/chunking never wrote the answer onto the
evidence page(s), retrieval cannot honestly recover it.

Lawful metric (no Acc inflation):
  answer_in_evidence_pages = gold short answer appears in markdown for gold
  evidence_pages (page-marker split). Offline / diagnostic only.

Protocol hardening (026):
  - Raw a_in_e is diagnostic only (short needles like "6"/"7" inflate hits).
  - Gate on answer_in_evidence_rate_long (needle_len ≥ MIN_NEEDLE_LEN_GATE).
  - Flag short_needle_fp_suspect when hit ∧ page_spread ≥ FP_SPREAD_THRESHOLD.
  - Cross-run compares require full answerable n (no max_samples for gates).

Uses GET /api/v1/query/context/documents/markdown/{document_id}.
"""

from __future__ import annotations

import re
from typing import Any, Optional

from .diagnostics import as_int_pages
from .protocol import (
    FP_SPREAD_THRESHOLD,
    MIN_NEEDLE_LEN_GATE,
    PROTOCOL_VERSION,
    GATE_CHART_A_IN_E_LONG,
    GATE_TABLE_A_IN_E_LONG,
)
from .subset import parse_list_field

PAGE_MARKER_RE = re.compile(r"<!--\s*edgequake-page:(\d+)\s*-->", re.IGNORECASE)
CROP_COVERAGE_RE = re.compile(
    r"<!--\s*edgequake-crop-coverage:\s*(.*?)\s*-->",
    re.IGNORECASE | re.DOTALL,
)


def parse_crop_coverage_comment(markdown: str) -> Optional[dict[str, int]]:
    """Parse W1-crop-telemetry HTML comment into int fields (026 instrumentation)."""
    m = CROP_COVERAGE_RE.search(markdown or "")
    if not m:
        return None
    out: dict[str, int] = {}
    for part in m.group(1).split():
        if "=" not in part:
            continue
        k, v = part.split("=", 1)
        try:
            out[k.strip()] = int(v.strip())
        except ValueError:
            continue
    return out or None


def split_markdown_by_page(markdown: str) -> dict[int, str]:
    """Split page-marked markdown into {page_num: text} (1-indexed).

    Duplicate page markers: later segment wins (LAST-wins). CONCAT vs LAST
    was audited equal on chart-8 a_in_e; LAST keeps SSOT simple.
    """
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
    """Aggressive normalize for number/string containment (not Acc scoring).

    W1-measure-listmem: drop ASCII/curly quote marks so gold `\"MMMU\"` matches
    page text `MMMU` (MMLongBench string golds often wrap labels in quotes).

    Year-span expand (032): `1981-82` / `2001-02` also index full years so list
    golds `['1981','1982']` hit when the page only prints the abbreviated span.
    """
    t = (text or "").lower()
    t = t.replace("%", "")
    t = t.replace("'", "").replace('"', "").replace("“", "").replace("”", "")
    t = t.replace("‘", "").replace("’", "")
    # Expand YYYY-YY / YYYY–YY spans before stripping separators.
    def _expand_span(m: re.Match[str]) -> str:
        y1 = int(m.group(1))
        yy = int(m.group(2))
        century = y1 - (y1 % 100)
        y2 = century + yy
        if y2 < y1:
            y2 += 100
        return f"{y1} {y2} {m.group(0)}"

    t = re.sub(r"\b((?:19|20)\d{2})[-–](\d{2})\b", _expand_span, t)
    t = re.sub(r"[\s,$_€£¥]+", "", t)
    return t


def list_gold_members(answer: str) -> list[str] | None:
    """Return list members when gold is a List answer; else None.

    Mirrors MMLongBench List scoring (per-element). A single-element list is
    still treated as list gold so containment stays member-wise.
    """
    ans = str(answer or "").strip()
    if not ans.startswith("["):
        return None
    members = parse_list_field(ans)
    if not members:
        return None
    out = [str(m).strip() for m in members if str(m).strip()]
    return out or None


def answer_needle(answer: str) -> str:
    """Normalized needle used for containment / length gating.

    List golds: concatenation of member needles (stable length for long gate).
    """
    members = list_gold_members(answer)
    if members:
        return "".join(normalize_for_containment(m) for m in members)
    return normalize_for_containment(str(answer or "").strip())


def answer_in_text(answer: str, text: str) -> bool:
    """True if gold answer is contained in text after normalization.

    Skips unanswerable / empty. Floats: also try without trailing zeros.

    List golds (W1-measure-listmem / MMLongBench List physics): hit iff *every*
    member is contained in text. Serializing `['a','b']` as one needle was a
    false-negative when both members already appear on the evidence page.
    """
    ans = str(answer or "").strip()
    if not ans or ans == "Not answerable":
        return False
    members = list_gold_members(ans)
    if members is not None:
        return all(answer_in_text(m, text) for m in members)
    hay = normalize_for_containment(text)
    needle = normalize_for_containment(ans)
    if not needle:
        return False
    if needle in hay:
        return True
    # numeric variants: 18.29 vs 18.290 / 0.1829 handled loosely for %
    try:
        f = float(ans.replace("%", "").replace(",", "").replace('"', "").replace("'", ""))
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

    needle = answer_needle(answer)
    needle_len = len(needle)
    n_pages = len(page_map)
    page_spread = (len(found_pages) / n_pages) if n_pages else 0.0
    short_needle = needle_len < MIN_NEEDLE_LEN_GATE
    long_eligible = needle_len >= MIN_NEEDLE_LEN_GATE and bool(needle)
    # Long hit: same containment, but only scored for gate-eligible needles
    in_evidence_long: bool | None
    if not long_eligible:
        in_evidence_long = None
    else:
        in_evidence_long = in_evidence
    fp_suspect = bool(
        in_evidence and short_needle and page_spread >= FP_SPREAD_THRESHOLD
    )

    return {
        "gold_evidence_pages": gold_pages,
        "answer_in_evidence_pages": in_evidence,
        "answer_in_evidence_pages_long": in_evidence_long,
        "answer_in_document": in_doc,
        "pages_with_answer": found_pages,
        "evidence_pages_present_in_markdown": [p for p in gold_pages if p in page_map],
        "n_pages_in_markdown": n_pages,
        "evidence_sources": [str(s) for s in srcs],
        "needle": needle,
        "needle_len": needle_len,
        "short_needle": short_needle,
        "page_spread": page_spread,
        "short_needle_fp_suspect": fp_suspect,
        "long_eligible": long_eligible,
    }


def _rate(flags: list[bool]) -> float:
    return sum(1 for x in flags if x) / len(flags) if flags else 0.0


def aggregate_fidelity(rows: list[dict[str, Any]]) -> dict[str, Any]:
    """Aggregate W1 fidelity rows (answerable only expected).

    Includes raw rates (legacy) and long-needle gate rates (protocol SSOT).
    """
    if not rows:
        return {"n": 0, "protocol_version": PROTOCOL_VERSION}
    n = len(rows)
    in_ev = sum(1 for r in rows if r.get("answer_in_evidence_pages"))
    in_doc = sum(1 for r in rows if r.get("answer_in_document"))
    # Prefer explicit long_eligible; fall back for older samples lacking fields
    if any("long_eligible" in r for r in rows):
        long_rows = [r for r in rows if r.get("long_eligible")]
    else:
        long_rows = [
            r
            for r in rows
            if len(answer_needle(str(r.get("needle") or ""))) >= MIN_NEEDLE_LEN_GATE
            or r.get("answer_in_evidence_pages_long") is not None
        ]

    def _long_hit(r: dict[str, Any]) -> bool:
        v = r.get("answer_in_evidence_pages_long")
        if v is not None:
            return bool(v)
        return bool(r.get("answer_in_evidence_pages"))

    long_hit_n = sum(1 for r in long_rows if _long_hit(r))
    fp_n = sum(1 for r in rows if r.get("short_needle_fp_suspect"))
    short_n = sum(1 for r in rows if r.get("short_needle"))

    by_src: dict[str, list[bool]] = {}
    by_src_long: dict[str, list[bool]] = {}
    by_src_excl: dict[str, list[bool]] = {}
    by_src_excl_long: dict[str, list[bool]] = {}
    for r in rows:
        hit = bool(r.get("answer_in_evidence_pages"))
        srcs = [str(s) for s in (r.get("evidence_sources") or ["?"])]
        for s in srcs:
            by_src.setdefault(s, []).append(hit)
        if "long_eligible" in r:
            eligible = bool(r.get("long_eligible"))
        elif r.get("needle_len") is not None:
            eligible = int(r["needle_len"]) >= MIN_NEEDLE_LEN_GATE
        else:
            # Legacy rows without needle fields: treat as eligible (= raw rate).
            eligible = True
        if eligible:
            lh = _long_hit(r)
            for s in srcs:
                by_src_long.setdefault(s, []).append(lh)
        if len(srcs) == 1:
            by_src_excl.setdefault(srcs[0], []).append(hit)
            if eligible:
                by_src_excl_long.setdefault(srcs[0], []).append(_long_hit(r))

    def _src_agg(d: dict[str, list[bool]]) -> dict[str, dict[str, Any]]:
        return {k: {"rate": _rate(v), "n": len(v)} for k, v in sorted(d.items())}

    def _gate(src: str, threshold: float) -> dict[str, Any]:
        flags = by_src_long.get(src) or []
        rate = _rate(flags) if flags else None
        return {
            "rate": rate,
            "n": len(flags),
            "threshold": threshold,
            "pass": bool(flags) and rate is not None and rate >= threshold,
        }

    n_long = len(long_rows)
    return {
        "n": n,
        "protocol_version": PROTOCOL_VERSION,
        "answer_in_evidence_rate": in_ev / n,
        "answer_in_document_rate": in_doc / n,
        "n_long_eligible": n_long,
        "answer_in_evidence_rate_long": (long_hit_n / n_long) if n_long else None,
        "n_short_needle": short_n,
        "n_short_needle_fp_suspect": fp_n,
        "min_needle_len_gate": MIN_NEEDLE_LEN_GATE,
        "fp_spread_threshold": FP_SPREAD_THRESHOLD,
        "by_evidence_source": _src_agg(by_src),
        "by_evidence_source_long": _src_agg(by_src_long),
        "by_evidence_source_exclusive": _src_agg(by_src_excl),
        "by_evidence_source_exclusive_long": _src_agg(by_src_excl_long),
        "gates": {
            "chart_a_in_e_long": _gate("Chart", GATE_CHART_A_IN_E_LONG),
            "table_a_in_e_long": _gate("Table", GATE_TABLE_A_IN_E_LONG),
        },
        "note": (
            "Gate Wave 1 on answer_in_evidence_rate_long / by_evidence_source_long, "
            "not raw answer_in_evidence_rate (short needles inflate). "
            "Require full answerable audit (n = all answerable)."
        ),
    }
