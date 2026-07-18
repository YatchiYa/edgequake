#!/usr/bin/env python3
"""SPEC-065–082 — honesty gate for product-limits SSOT vs FAQ/envelope."""
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SSOT = ROOT / "docs" / "product-limits.md"
FAQ = ROOT / "docs" / "faq.md"
ENVELOPE = ROOT / "specs" / "063-architecture-capacity-assessment" / "003-operating-envelope.md"
DATA_LAYER = ROOT / "docs" / "deep-dives" / "data-layer.md"
CEILING_NOTES = ROOT / "specs" / "066-ceiling-proof" / "e2e" / "artifacts" / "RUN_NOTES.md"
PARETO_NOTES = ROOT / "specs" / "068-recall-quality-scale" / "e2e" / "artifacts" / "RUN_NOTES.md"
DEDICATED_NOTES = ROOT / "specs" / "069-dedicated-midscale" / "e2e" / "artifacts" / "RUN_NOTES.md"
WAVE2_PACK = ROOT / "specs" / "071-wave2-greenfield" / "000-index.md"
DISKANN_PACK = ROOT / "specs" / "070-diskann-study" / "000-index.md"
DISKANN_NOTES = ROOT / "specs" / "070-diskann-study" / "e2e" / "artifacts" / "RUN_NOTES.md"
DISKANN_PARETO_NOTES = (
    ROOT / "specs" / "072-diskann-recall-pareto" / "e2e" / "artifacts" / "RUN_NOTES.md"
)
DISKANN_RESCORE_NOTES = (
    ROOT / "specs" / "074-storage-p0-hardening" / "e2e" / "artifacts" / "RUN_NOTES.md"
)
FILTERED_RECALL_NOTES = (
    ROOT / "specs" / "075-filtered-recall-gates" / "e2e" / "artifacts" / "RUN_NOTES.md"
)
FILTERED_RECALL_PACK = ROOT / "specs" / "075-filtered-recall-gates" / "000-index.md"
PRECISION_PACK = ROOT / "specs" / "076-precision-reorder-rrf" / "000-index.md"
PRECISION_NOTES = (
    ROOT / "specs" / "076-precision-reorder-rrf" / "e2e" / "artifacts" / "RUN_NOTES.md"
)
BINARY_PACK = ROOT / "specs" / "077-binary-quantize-bakeoff" / "000-index.md"
BINARY_NOTES = (
    ROOT / "specs" / "077-binary-quantize-bakeoff" / "e2e" / "artifacts" / "RUN_NOTES.md"
)
FDL_PACK = ROOT / "specs" / "078-filtered-diskann-labels" / "000-index.md"
FDL_NOTES = (
    ROOT / "specs" / "078-filtered-diskann-labels" / "e2e" / "artifacts" / "RUN_NOTES.md"
)
MIDSCALE_PACK = ROOT / "specs" / "079-midscale-quantize-labels" / "000-index.md"
MIDSCALE_NOTES = (
    ROOT / "specs" / "079-midscale-quantize-labels" / "e2e" / "artifacts" / "RUN_NOTES.md"
)
TINY_PACK = ROOT / "specs" / "080-tiny-slice-exact" / "000-index.md"
TINY_NOTES = ROOT / "specs" / "080-tiny-slice-exact" / "e2e" / "artifacts" / "RUN_NOTES.md"
SERVING_PACK = ROOT / "specs" / "081-serving-view-dual-ssot" / "000-index.md"
SERVING_NOTES = (
    ROOT / "specs" / "081-serving-view-dual-ssot" / "e2e" / "artifacts" / "RUN_NOTES.md"
)
PUSH_PACK = ROOT / "specs" / "082-push-scale-floors" / "000-index.md"
PUSH_NOTES = ROOT / "specs" / "082-push-scale-floors" / "e2e" / "artifacts" / "RUN_NOTES.md"


def fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    sys.exit(1)


def main() -> None:
    for p in (SSOT, FAQ, ENVELOPE, DATA_LAYER):
        if not p.is_file():
            fail(f"missing required file: {p}")

    ssot = SSOT.read_text()
    faq = FAQ.read_text()
    envelope = ENVELOPE.read_text()
    data_layer = DATA_LAYER.read_text()

    if "product-limits" not in faq:
        fail("docs/faq.md must link to product-limits SSOT")

    if "Wave-2" not in faq and "partial HNSW" not in faq and "PARTIAL_BY_WORKSPACE" not in faq:
        fail("docs/faq.md must mention Wave-2 / partial HNSW for 100k Q1-d")

    # FAQ must not treat bare 8+ GB as sufficient for proven floors
    if "8+ GB" in faq or "8+ GB" in faq:
        if "Pick your size" not in faq and "16" not in faq and "shared_buffers" not in faq:
            fail("FAQ mentions 8+ GB without host-class / Pick your size guidance")

    if "shared_buffers" not in faq and "16" not in faq:
        fail("docs/faq.md must mention host-class sizing (≥16GB / shared_buffers) for proven floors")

    if "Wave-2" not in faq.split("How can I speed up queries?")[-1][:800]:
        # speed-up section should mention Wave-2 / residency early
        speed = faq.lower()
        if "wave-2" not in speed or "shared_buffers" not in speed:
            fail("FAQ speed-up section must mention Wave-2 and residency")

    for line in faq.splitlines():
        if "500k" in line.lower() and "supported" in line.lower():
            if "unproven" not in line.lower() and "wave-2" not in line.lower() and "caveat" not in line.lower() and "only" not in line.lower() and "not promoted" not in line.lower():
                if "aspirational" in line.lower():
                    continue
                fail(f"FAQ line overclaims 500k without caveat: {line.strip()}")

    if "product-limits" not in envelope and "064-filtered" not in envelope:
        fail("SPEC-063 envelope must cross-link product-limits or SPEC-064")

    if "product-limits.md" not in data_layer and "../product-limits.md" not in data_layer:
        fail("data-layer.md Capacity section must link docs/product-limits.md")

    required_ssot = [
        "TL;DR",
        "Pick your size",
        "What to set",
        "Turnkey greenfield",
        "wave2-greenfield",
        "wave2_warmup",
        "Hard caps",
        "Do not",
        "EDGEQUAKE_HNSW_PARTIAL_BY_WORKSPACE",
        "EDGEQUAKE_HNSW_EF_SEARCH=240",
        "Mid-scale wall",
        "Shared+partial vs dedicated",
        "Mix / hybrid honesty",
        "Dedicated WS table",
        "50k",
        "100k",
        "highest_green_N",
        "first_fail_N",
        "max_documents",
        "shared_buffers",
        "Glossary",
        "SPEC-071",
        "SPEC-072",
        "DiskANN",
        "Opt-in DiskANN",
        "pg18-vectorscale",
        "diskann-recall-pareto",
        "diskann-rescore-smoke",
        "query_search_list_size",
        "query_rescore",
        "150 000",
        "SPEC-074",
        "SPEC-075",
        "filtered-recall-gate",
        "EDGEQUAKE_HNSW_ITERATIVE_SCAN",
        "EDGEQUAKE_HNSW_MAX_SCAN_TUPLES",
        "EDGEQUAKE_HNSW_SCAN_MEM_MULTIPLIER",
        "iterative_scan",
        "max_scan_tuples",
        "filtered recall",
        "SPEC-076",
        "precision-layers-gate",
        "EDGEQUAKE_ANN_EXACT_REORDER",
        "EDGEQUAKE_ANN_REORDER_CANDIDATE_K",
        "EDGEQUAKE_SPARSE_FUSION",
        "exact reorder",
        "SPEC-077",
        "binary-quantize-bakeoff",
        "EDGEQUAKE_BINARY_QUANTIZE",
        "binary_quantize",
        "bit_hamming",
        "SPEC-078",
        "filtered-diskann-labels",
        "EDGEQUAKE_FILTERED_DISKANN_LABELS",
        "labels &&",
        "Filtered-DiskANN",
        "SPEC-079",
        "midscale-quantize-labels",
        "SPEC-080",
        "tiny-slice-exact",
        "EDGEQUAKE_ANN_EXACT_MAX_ROWS",
        "SPEC-081",
        "serving-view",
        "eq_serving_chunk_presence",
        "SPEC-082",
        "push-scale-ladder",
    ]
    for token in required_ssot:
        if token not in ssot:
            fail(f"docs/product-limits.md missing required token: {token}")

    if "How do I enable the supported 100k shape" not in faq and "turnkey greenfield" not in faq.lower():
        fail("docs/faq.md must answer how to enable the supported 100k / turnkey greenfield shape")

    if (
        "opt-in DiskANN" not in faq.lower()
        and "DiskANN @150k" not in faq
        and "DiskANN @250k" not in faq
    ):
        fail("docs/faq.md must mention opt-in DiskANN @150k / @250k")

    if not WAVE2_PACK.is_file():
        fail(f"missing SPEC-071 pack: {WAVE2_PACK}")
    if "wave2-greenfield" not in WAVE2_PACK.read_text() and "Turnkey" not in WAVE2_PACK.read_text():
        fail("SPEC-071 pack must mention wave2-greenfield / turnkey")

    if not DISKANN_PACK.is_file():
        fail(f"missing SPEC-070 pack: {DISKANN_PACK}")
    if not DISKANN_NOTES.is_file():
        fail(f"missing SPEC-070 RUN_NOTES: {DISKANN_NOTES}")

    if not DISKANN_PARETO_NOTES.is_file():
        fail(f"missing SPEC-072 Pareto RUN_NOTES: {DISKANN_PARETO_NOTES}")
    p072 = DISKANN_PARETO_NOTES.read_text()
    if "Promote SSOT: **YES**" not in p072 and "promote_ssot=true" not in p072:
        fail("SPEC-072 RUN_NOTES must record promote SSOT YES (opt-in DiskANN @150k)")
    if "query_search_list_size" not in p072 and "q_list" not in p072:
        fail("SPEC-072 RUN_NOTES must document query_search_list / q_list tuning")
    if "150" not in p072:
        fail("SPEC-072 RUN_NOTES must reference 150k")

    if not DISKANN_RESCORE_NOTES.is_file():
        fail(f"missing SPEC-074 rescore RUN_NOTES: {DISKANN_RESCORE_NOTES}")
    p074 = DISKANN_RESCORE_NOTES.read_text()
    if "query_rescore" not in p074:
        fail("SPEC-074 RUN_NOTES must document query_rescore")
    if "400" not in p074:
        fail("SPEC-074 RUN_NOTES must document list=400 recipe")

    if "query_rescore" not in faq.lower():
        fail("docs/faq.md must mention query_rescore for opt-in DiskANN")

    if "filtered-recall-gate" not in faq and "filtered recall" not in faq.lower():
        fail("docs/faq.md must mention filtered-recall-gate / filtered recall (SPEC-075)")
    if "iterative_scan" not in faq.lower() and "max_scan_tuples" not in faq.lower():
        fail("docs/faq.md must mention iterative_scan / max_scan_tuples bounds")

    if not FILTERED_RECALL_PACK.is_file():
        fail(f"missing SPEC-075 pack: {FILTERED_RECALL_PACK}")
    if not FILTERED_RECALL_NOTES.is_file():
        fail(f"missing SPEC-075 RUN_NOTES: {FILTERED_RECALL_NOTES}")
    p075 = FILTERED_RECALL_NOTES.read_text()
    if "filtered" not in p075.lower() or "recall@20" not in p075.lower():
        fail("SPEC-075 RUN_NOTES must document filtered recall@20")
    if "unfiltered-only" not in p075.lower() and "never unfiltered" not in p075.lower():
        fail("SPEC-075 RUN_NOTES must forbid unfiltered-only promote metric")

    if "precision-layers-gate" not in faq and "exact reorder" not in faq.lower():
        fail("docs/faq.md must mention precision-layers-gate / exact reorder (SPEC-076)")
    if "EDGEQUAKE_SPARSE_FUSION" not in faq and "sparse" not in faq.lower():
        fail("docs/faq.md must mention sparse FTS+ANN RRF tip")

    if not PRECISION_PACK.is_file():
        fail(f"missing SPEC-076 pack: {PRECISION_PACK}")
    if not PRECISION_NOTES.is_file():
        fail(f"missing SPEC-076 RUN_NOTES: {PRECISION_NOTES}")
    p076 = PRECISION_NOTES.read_text()
    if "EDGEQUAKE_ANN_EXACT_REORDER" not in p076 and "exact reorder" not in p076.lower():
        fail("SPEC-076 RUN_NOTES must document exact reorder")
    if "SPARSE_FUSION" not in p076 and "rrf" not in p076.lower():
        fail("SPEC-076 RUN_NOTES must document sparse RRF tip")
    if "default OFF" not in p076 and "default 0" not in p076.lower() and "default off" not in p076.lower():
        fail("SPEC-076 RUN_NOTES must record exact reorder default OFF")

    if "binary-quantize-bakeoff" not in faq and "binary quant" not in faq.lower():
        fail("docs/faq.md must mention binary-quantize-bakeoff / binary quantization (SPEC-077)")
    if not BINARY_PACK.is_file():
        fail(f"missing SPEC-077 pack: {BINARY_PACK}")
    if not BINARY_NOTES.is_file():
        fail(f"missing SPEC-077 RUN_NOTES: {BINARY_NOTES}")
    p077 = BINARY_NOTES.read_text()
    if "filtered" not in p077.lower() or "binary" not in p077.lower():
        fail("SPEC-077 RUN_NOTES must document filtered binary bake-off")
    if "Wave-2" not in p077 and "wave-2" not in p077.lower():
        fail("SPEC-077 RUN_NOTES must keep Wave-2 as default")
    if "not" not in p077.lower() or ("silent" not in p077.lower() and "opt-in" not in p077.lower()):
        fail("SPEC-077 RUN_NOTES must forbid silent promote / record opt-in study")

    if "filtered-diskann-labels" not in faq and "Filtered-DiskANN" not in faq:
        fail("docs/faq.md must mention filtered-diskann-labels / Filtered-DiskANN (SPEC-078)")
    if not FDL_PACK.is_file():
        fail(f"missing SPEC-078 pack: {FDL_PACK}")
    if not FDL_NOTES.is_file():
        fail(f"missing SPEC-078 RUN_NOTES: {FDL_NOTES}")
    p078 = FDL_NOTES.read_text()
    if "filtered" not in p078.lower() or "labels" not in p078.lower():
        fail("SPEC-078 RUN_NOTES must document filtered labels bake-off")
    if "Wave-2" not in p078 and "wave-2" not in p078.lower():
        fail("SPEC-078 RUN_NOTES must keep Wave-2 as default")
    if "silent" not in p078.lower() and "opt-in" not in p078.lower():
        fail("SPEC-078 RUN_NOTES must forbid silent promote / record opt-in study")

    if "midscale-quantize-labels" not in faq and "mid-scale" not in faq.lower():
        fail("docs/faq.md must mention midscale-quantize-labels / mid-scale (SPEC-079)")
    if not MIDSCALE_PACK.is_file():
        fail(f"missing SPEC-079 pack: {MIDSCALE_PACK}")
    if not MIDSCALE_NOTES.is_file():
        fail(f"missing SPEC-079 RUN_NOTES: {MIDSCALE_NOTES}")
    p079 = MIDSCALE_NOTES.read_text()
    if "Decision" not in p079 and "decision" not in p079.lower():
        fail("SPEC-079 RUN_NOTES must record Decision")
    if "Not promoted" not in p079 and "promote candidate" not in p079.lower():
        fail("SPEC-079 RUN_NOTES must record Not promoted or promote candidate")
    if "silent" not in p079.lower():
        fail("SPEC-079 RUN_NOTES must forbid silent flip")

    if "tiny-slice" not in faq.lower() and "ANN_EXACT_MAX_ROWS" not in faq:
        fail("docs/faq.md must mention tiny-slice / ANN_EXACT_MAX_ROWS (SPEC-080)")
    if not TINY_PACK.is_file():
        fail(f"missing SPEC-080 pack: {TINY_PACK}")
    if not TINY_NOTES.is_file():
        fail(f"missing SPEC-080 RUN_NOTES: {TINY_NOTES}")
    p080 = TINY_NOTES.read_text()
    if "ANN_EXACT_MAX_ROWS" not in p080 and "2000" not in p080:
        fail("SPEC-080 RUN_NOTES must document EDGEQUAKE_ANN_EXACT_MAX_ROWS / 2000")

    if "serving-view" not in faq and "eq_serving_chunk_presence" not in faq:
        fail("docs/faq.md must mention serving-view / eq_serving_chunk_presence (SPEC-081)")
    if not SERVING_PACK.is_file():
        fail(f"missing SPEC-081 pack: {SERVING_PACK}")
    if not SERVING_NOTES.is_file():
        fail(f"missing SPEC-081 RUN_NOTES: {SERVING_NOTES}")
    p081 = SERVING_NOTES.read_text()
    if "eq_serving_chunk_presence" not in p081:
        fail("SPEC-081 RUN_NOTES must document eq_serving_chunk_presence")
    if "ANN SSOT" not in p081 and "not" not in p081.lower():
        fail("SPEC-081 RUN_NOTES must forbid treating serving view as ANN SSOT")

    if "push-scale-ladder" not in faq and "SPEC-082" not in faq:
        fail("docs/faq.md must mention push-scale-ladder / SPEC-082")
    if not PUSH_PACK.is_file():
        fail(f"missing SPEC-082 pack: {PUSH_PACK}")
    if not PUSH_NOTES.is_file():
        fail(f"missing SPEC-082 RUN_NOTES: {PUSH_NOTES}")
    p082 = PUSH_NOTES.read_text()
    if "Decision" not in p082 and "decision" not in p082.lower():
        fail("SPEC-082 RUN_NOTES must record Decision")
    if "silent" not in p082.lower():
        fail("SPEC-082 RUN_NOTES must forbid silent flip")
    # DiskANN floor may stay 150k or rise to 250k after full-gate; both honest.
    if "highest_green_N" in ssot:
        if "150 000" not in ssot and "250 000" not in ssot and "150000" not in ssot:
            fail("SSOT DiskANN/Wave-2 ceiling fields must cite 150k or 250k class floors")

    notes = CEILING_NOTES.read_text() if CEILING_NOTES.is_file() else ""
    if "highest_green_N" in ssot and "100" in ssot:
        if "highest_green_N" not in notes and "highest_green_n" not in notes.lower():
            fail("SSOT cites highest_green_N but SPEC-066 RUN_NOTES missing it")

    if not PARETO_NOTES.is_file():
        fail(f"missing SPEC-068 Pareto RUN_NOTES: {PARETO_NOTES}")
    pareto = PARETO_NOTES.read_text()
    if "No cell with" not in pareto and "not promoted" not in pareto.lower() and "unchanged" not in pareto.lower():
        if "100" not in pareto:
            fail("SPEC-068 RUN_NOTES must record 100k floor / mid-scale wall")

    if not DEDICATED_NOTES.is_file():
        fail(f"missing SPEC-069 dedicated RUN_NOTES: {DEDICATED_NOTES}")
    ded = DEDICATED_NOTES.read_text()
    if "SPEC-070" not in ded and "open" not in ded.lower():
        fail("SPEC-069 RUN_NOTES must record SPEC-070 open/closed decision")
    if "Promote 150k" not in ded and "150k" not in ded:
        fail("SPEC-069 RUN_NOTES must record 150k promotion decision")

    print("OK product-limits-check: SSOT ↔ FAQ ↔ envelope ↔ data-layer consistent")


if __name__ == "__main__":
    main()
