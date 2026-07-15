"""SPEC-047 measurement protocol constants + Acc attribution helpers.

Hardened 2026-07-15 after First-Principles audit:
  - Acc/F1 soft-score remains official MMLongBench law (vendored eval_score).
  - W1 fidelity gates use *long-needle* a_in_e (short golds inflate raw rates).
  - Evidence-source Acc: report official multi-label *and* exclusive (len==1).
  - Acc storytelling requires Chart Acc ↑ + fidelity long gate — Acc alone is not W1 win.
  - Cross-run fidelity compares require same n (all answerable); max_samples is debug-only.

List-member honesty (2026-07-15, 029 / W1-measure-listmem):
  - MMLongBench scores List golds per-element (greedy avg). Fidelity mirrors that:
    list gold is in evidence iff *every* member is contained (not the serialized list string).
  - Strip wrapping quote chars from scalar needles (`"MMMU"` → MMMU).
"""

from __future__ import annotations

from typing import Any

from . import mmlongbench_eval_score as ev
from .subset import parse_list_field

PROTOCOL_VERSION = "026-listmem-2026-07-15"

# ---------------------------------------------------------------------------
# SSOT: best chart-8 score stack (Acc #5 W3-arith-v2, 2026-07-15).
# Retain these levers in code for Phase B+ benches — do not regress without FP note.
# Acc/F1 reference: Acc #2 F1 0.480 still beats Acc #5 F1 0.457 on chart-8.
# ---------------------------------------------------------------------------
DEFAULT_BENCH_PROFILE = "P0_mm_ite"

BEST_SCORE_STACK: dict[str, Any] = {
    "profile_id": DEFAULT_BENCH_PROFILE,
    "protocol_version": PROTOCOL_VERSION,
    "query_mode": "hybrid",
    "process_options": "ite",
    "document_scope": True,
    "chart8_smoke_acc": 0.562,
    "chart8_smoke_f1": 0.457,
    "chart8_acc_f1_sota_f1": 0.480,
    "artifact_ref": "smoke-chart8-026-w3-arith-v2-20260715-2126",
    "levers": [
        "W1 fig-as-chart + coexist + crop-expand (ingest)",
        "W1-measure-listmem + year-span fidelity (measure)",
        "W3-arith-v2 MUST compute + worked example 36%×1503→541 (Gen)",
        "W1-dense-scalar Acc#3 callouts REVERTED (negative)",
    ],
    "honesty": (
        "Acc↑ from W3-arith = Gen composition, not W1 Chart representation. "
        "Chart a_in_e_long gate uses long-needle fidelity, not Acc alone."
    ),
}

# Containment gate: needles shorter than this are excluded from a_in_e_long.
MIN_NEEDLE_LEN_GATE = 3
# Hit + answer appears on this fraction of pages → short-needle FP suspect.
FP_SPREAD_THRESHOLD = 0.30

# Wave 1 exit (chart-8): gate on long rates, not raw short-needle rates.
GATE_CHART_A_IN_E_LONG = 0.50
GATE_TABLE_A_IN_E_LONG = 0.55


def _srcs(sample: dict[str, Any]) -> list[str]:
    raw = sample.get("evidence_sources")
    if isinstance(raw, list):
        return [str(s) for s in raw]
    return [str(s) for s in (parse_list_field(str(raw or "[]")) or [])]


def exclusive_source(sample: dict[str, Any]) -> str | None:
    """Primary evidence source when exactly one label; else None (multi-source)."""
    srcs = _srcs(sample)
    if len(srcs) == 1:
        return srcs[0]
    return None


def bucket_for_attribution(sample: dict[str, Any]) -> str:
    """Partition for Acc Δ storytelling (not official slices)."""
    ans = str(sample.get("answer") or "").strip()
    if ans == "Not answerable":
        return "unanswerable"
    if ans.startswith("["):
        return "list_gold"
    return "other_answerable"


def mean_acc(rows: list[dict[str, Any]]) -> float | None:
    scored = [r for r in rows if "score" in r]
    if not scored:
        return None
    return float(sum(float(r["score"]) for r in scored) / len(scored))


def attribution_slices(samples: list[dict[str, Any]]) -> dict[str, Any]:
    """Single-run Acc mass by attribution bucket (list / unans / other)."""
    buckets: dict[str, list[dict[str, Any]]] = {
        "list_gold": [],
        "unanswerable": [],
        "other_answerable": [],
    }
    for s in samples:
        if "score" not in s:
            continue
        buckets[bucket_for_attribution(s)].append(s)
    out: dict[str, Any] = {"protocol_version": PROTOCOL_VERSION}
    for name, rows in buckets.items():
        out[name] = {
            "n": len(rows),
            "accuracy": mean_acc(rows),
            "score_sum": float(sum(float(r["score"]) for r in rows)) if rows else 0.0,
        }
    return out


def paired_acc_delta(
    base: list[dict[str, Any]],
    now: list[dict[str, Any]],
) -> dict[str, Any]:
    """Paired Acc Δ mass by attribution bucket (same doc_id+question keys)."""
    bm = {(r["doc_id"], r["question"]): r for r in base if "score" in r}
    nm = {(r["doc_id"], r["question"]): r for r in now if "score" in r}
    common = set(bm) & set(nm)
    mass: dict[str, float] = {
        "list_gold": 0.0,
        "unanswerable": 0.0,
        "other_answerable": 0.0,
    }
    n_improved = n_worsened = n_flat = 0
    for k in common:
        b, n = bm[k], nm[k]
        d = float(n["score"]) - float(b["score"])
        mass[bucket_for_attribution(n)] += d
        if d > 1e-9:
            n_improved += 1
        elif d < -1e-9:
            n_worsened += 1
        else:
            n_flat += 1
    n = len(common) or 1
    return {
        "protocol_version": PROTOCOL_VERSION,
        "n_paired": len(common),
        "n_improved": n_improved,
        "n_worsened": n_worsened,
        "n_flat": n_flat,
        "delta_acc": sum(mass.values()) / n if common else 0.0,
        "mass": mass,
        "acc_points": {k: v / n for k, v in mass.items()},
        "note": (
            "list_gold mass often = extract normalize (W4), not W1 Chart representation. "
            "Do not claim W1 win from Acc alone."
        ),
    }


def exclusive_source_accuracy(
    samples: list[dict[str, Any]],
) -> dict[str, dict[str, Any]]:
    """Acc for questions with exactly one evidence_sources label (honest Chart-only)."""
    by: dict[str, list[dict[str, Any]]] = {}
    for s in samples:
        if "score" not in s:
            continue
        ex = exclusive_source(s)
        if ex is None:
            continue
        by.setdefault(ex, []).append(s)
    return {
        k: {
            "accuracy": float(ev.eval_acc_and_f1(v)[0]),
            "n": len(v),
        }
        for k, v in sorted(by.items())
    }


def gate_notes() -> dict[str, Any]:
    return {
        "protocol_version": PROTOCOL_VERSION,
        "fidelity_gate_metric": "answer_in_evidence_rate_long",
        "min_needle_len_gate": MIN_NEEDLE_LEN_GATE,
        "fp_spread_threshold": FP_SPREAD_THRESHOLD,
        "chart_a_in_e_long_min": GATE_CHART_A_IN_E_LONG,
        "table_a_in_e_long_min": GATE_TABLE_A_IN_E_LONG,
        "require_full_answerable_audit": True,
        "acc_claim_requires": [
            "Chart exclusive Acc ↑ or multi-label Chart Acc ↑",
            "Chart a_in_e_long ≥ gate",
            "paired attribution shows non-list_gold contribution if claiming W1",
        ],
    }
