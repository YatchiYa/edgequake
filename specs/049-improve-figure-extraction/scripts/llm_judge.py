#!/usr/bin/env python3
"""
SPEC-049 LLM Judge — Mistral Large Vision
==========================================

Scores every extracted figure PNG using **first-principles** criteria:

  1. SIGNAL      — Does the crop contain a meaningful visual element?
  2. CROP QUALITY— Is the bounding box precise (no clipped content, minimal padding waste)?
  3. COMPLETENESS— Is the full figure visible (not partially cut)?

The judge reads `report.json` files produced by the Rust stress test, sends each
figure to `pixtral-large-latest` (Mistral's vision-capable Mistral Large), and
writes:
  • `judge_results.json`   — machine-readable full results
  • `judge_report.md`      — human-readable Markdown summary per document

Usage:
    python3 specs/049-improve-figure-extraction/scripts/llm_judge.py [--e2e-dir PATH]

Environment:
    MISTRAL_API_KEY   — required
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import sys
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Optional

from mistralai.client import Mistral

# ── Config ────────────────────────────────────────────────────────────────────

MODEL = "mistral-large-latest"          # Mistral Large with vision (2512)
RETRY_ATTEMPTS = 3
RETRY_DELAY_S  = 2.0

DEFAULT_E2E_DIR = (
    Path(__file__).parent.parent / "e2e" / "markdown"
)

# ── First-Principles Judge Prompt ─────────────────────────────────────────────
#
# Axiom: A visual crop has value only when the text channel cannot carry the
# same meaning (SPEC-049/001 modality-split principle).
# Therefore a "good" figure is one that:
#   - Contains quantitative/structural/pictorial content a reader cannot get
#     from prose or formatted text alone.
#   - Is precisely bounded — not a full-page dump, not clipped.
#   - Is complete — the visual element is fully visible.
#
JUDGE_SYSTEM = """\
You are an expert PDF figure extraction quality auditor.
Your task is to evaluate a cropped image that was automatically extracted from
a research-paper PDF as a potential "figure" or "chart".

Use FIRST PRINCIPLES to score it:

AXIOM: A visual crop delivers unique value only when its content CANNOT be
expressed by plain text or structured Markdown tables alone.

Evaluate on THREE dimensions (score 1–5 each):

1. SIGNAL (1=none, 5=full)
   5 — Clear quantitative chart (bar/line/scatter/pie/heatmap), architecture
       diagram with boxes/arrows, photograph, or technical illustration.
   3 — Partial figure, small icon, or diagram mixed with text.
   1 — Pure text, equation, decorative rule, logo, whitespace, or tiny ornament.

2. CROP_QUALITY (1=bad, 5=excellent)
   5 — Crop tightly wraps just the figure content with minimal padding; no
       unrelated prose visible.
   3 — Some extra whitespace or a few lines of unrelated text included.
   1 — Crops across the middle of the figure OR includes large text blocks.

3. COMPLETENESS (1=partial, 5=complete)
   5 — The entire figure is visible (axes, legend, all panels).
   3 — Most of the figure visible; a title or legend may be cut.
   1 — Only a small portion visible; figure clearly continues beyond crop.

Also provide:
  figure_type: one of
    "line_chart" | "bar_chart" | "scatter_plot" | "pie_chart" | "heatmap" |
    "architecture_diagram" | "flowchart" | "illustration" | "photograph" |
    "table_visual" | "equation" | "text_block" | "icon_logo" |
    "small_ornament" | "other"
  is_useful_crop: true if signal≥3 AND crop_quality≥3 AND completeness≥3
  issues: [] or list of short strings describing problems
    (e.g. "clipped bottom", "mostly text", "too small <40px")

Respond with ONLY valid JSON, no markdown fences, no explanation:
{
  "signal": <int 1-5>,
  "crop_quality": <int 1-5>,
  "completeness": <int 1-5>,
  "figure_type": "<string>",
  "is_useful_crop": <bool>,
  "issues": [<string>, ...]
}"""

JUDGE_USER = "Evaluate this extracted PDF figure crop:"


# ── Data classes ──────────────────────────────────────────────────────────────

@dataclass
class JudgeScore:
    signal: int = 0
    crop_quality: int = 0
    completeness: int = 0
    figure_type: str = "other"
    is_useful_crop: bool = False
    issues: list[str] = field(default_factory=list)
    raw_response: str = ""
    error: Optional[str] = None


@dataclass
class FigureResult:
    pdf: str
    asset_path: str        # relative to document dir
    page: int
    index: int
    label: str
    width: int
    height: int
    area_frac_px: float
    source: str
    score: JudgeScore


# ── Mistral client ────────────────────────────────────────────────────────────

def build_client() -> Mistral:
    key = os.environ.get("MISTRAL_API_KEY", "")
    if not key:
        sys.exit("ERROR: MISTRAL_API_KEY not set")
    return Mistral(api_key=key)


def encode_image(path: Path) -> str:
    return base64.standard_b64encode(path.read_bytes()).decode()


def judge_figure(client: Mistral, img_path: Path) -> JudgeScore:
    """Call Mistral vision API and parse the structured score."""
    b64 = encode_image(img_path)

    for attempt in range(RETRY_ATTEMPTS):
        try:
            resp = client.chat.complete(
                model=MODEL,
                messages=[
                    {"role": "system", "content": JUDGE_SYSTEM},
                    {
                        "role": "user",
                        "content": [
                            {
                                "type": "image_url",
                                "image_url": {
                                    "url": f"data:image/png;base64,{b64}"
                                },
                            },
                            {"type": "text", "text": JUDGE_USER},
                        ],
                    },
                ],
                max_tokens=400,
                temperature=0.0,
            )
            raw = resp.choices[0].message.content.strip()
            # Strip accidental markdown fences
            if raw.startswith("```"):
                raw = raw.split("```")[1].lstrip("json").strip()
            data = json.loads(raw)
            return JudgeScore(
                signal=int(data.get("signal", 0)),
                crop_quality=int(data.get("crop_quality", 0)),
                completeness=int(data.get("completeness", 0)),
                figure_type=str(data.get("figure_type", "other")),
                is_useful_crop=bool(data.get("is_useful_crop", False)),
                issues=list(data.get("issues", [])),
                raw_response=raw,
            )
        except Exception as exc:
            if attempt < RETRY_ATTEMPTS - 1:
                time.sleep(RETRY_DELAY_S * (attempt + 1))
            else:
                return JudgeScore(error=str(exc), raw_response="")
    return JudgeScore(error="exhausted retries", raw_response="")


# ── Main pipeline ─────────────────────────────────────────────────────────────

def run_judge(e2e_dir: Path) -> list[FigureResult]:
    client = build_client()
    results: list[FigureResult] = []

    doc_dirs = sorted(
        [d for d in e2e_dir.iterdir() if d.is_dir()],
        key=lambda d: d.name,
    )
    if not doc_dirs:
        sys.exit(f"ERROR: no document directories found under {e2e_dir}")

    total_figs = sum(
        len(json.loads((d / "report.json").read_text())["regions"])
        for d in doc_dirs
        if (d / "report.json").exists()
    )
    print(
        f"Judge: {len(doc_dirs)} documents, {total_figs} figures — model={MODEL}"
    )

    done = 0
    for doc_dir in doc_dirs:
        report_path = doc_dir / "report.json"
        if not report_path.exists():
            print(f"  [skip] {doc_dir.name}: no report.json")
            continue

        report = json.loads(report_path.read_text())
        pdf_name = report["pdf"]
        regions  = report["regions"]
        print(f"\n  {pdf_name} — {len(regions)} regions")

        for region in regions:
            asset_rel = region["asset_path"]   # "assets/p02-fig-01.png"
            asset_full = doc_dir / asset_rel

            if not asset_full.exists():
                print(f"    MISSING {asset_rel}")
                continue

            done += 1
            print(
                f"    [{done}/{total_figs}] {asset_rel} "
                f"({region['width']}×{region['height']}) ...",
                end="",
                flush=True,
            )
            score = judge_figure(client, asset_full)
            marker = "✓" if score.is_useful_crop else "✗"
            if score.error:
                print(f" ERROR: {score.error}")
            else:
                print(
                    f" {marker} sig={score.signal} cq={score.crop_quality} "
                    f"cmp={score.completeness} type={score.figure_type}"
                )

            results.append(
                FigureResult(
                    pdf=pdf_name,
                    asset_path=asset_rel,
                    page=region["page"],
                    index=region["index"],
                    label=region["label"],
                    width=region["width"],
                    height=region["height"],
                    area_frac_px=region["area_frac_px"],
                    source=region["source"],
                    score=score,
                )
            )

    return results


# ── Report writers ────────────────────────────────────────────────────────────

def write_json_results(results: list[FigureResult], out_dir: Path) -> None:
    data = [
        {
            "pdf": r.pdf,
            "asset_path": r.asset_path,
            "page": r.page,
            "index": r.index,
            "label": r.label,
            "width": r.width,
            "height": r.height,
            "area_frac_px": round(r.area_frac_px, 4),
            "source": r.source,
            **asdict(r.score),
        }
        for r in results
    ]
    path = out_dir / "judge_results.json"
    path.write_text(json.dumps(data, indent=2))
    print(f"\nJSON results → {path}")


def write_markdown_report(results: list[FigureResult], out_dir: Path) -> None:
    """Write a first-principles quality report as Markdown."""
    lines = [
        "# SPEC-049 Figure Extraction — LLM Judge Report",
        "",
        f"> Model: `{MODEL}`  |  Total figures: {len(results)}",
        "",
    ]

    # Aggregate stats
    useful = [r for r in results if r.score.is_useful_crop]
    not_useful = [r for r in results if not r.score.is_useful_crop and not r.score.error]
    errors = [r for r in results if r.score.error]

    lines += [
        "## Summary",
        "",
        f"| Metric | Value |",
        f"|--------|-------|",
        f"| Total figures extracted | {len(results)} |",
        f"| Useful crops (sig≥3 ∧ cq≥3 ∧ cmp≥3) | **{len(useful)}** ({100*len(useful)//max(len(results),1)}%) |",
        f"| Not useful | {len(not_useful)} ({100*len(not_useful)//max(len(results),1)}%) |",
        f"| Judge errors | {len(errors)} |",
        "",
    ]

    # Figure type distribution
    from collections import Counter
    type_counts = Counter(r.score.figure_type for r in results if not r.score.error)
    lines += ["## Figure Type Distribution", ""]
    lines += ["| Type | Count |", "|------|-------|"]
    for ft, cnt in type_counts.most_common():
        lines.append(f"| {ft} | {cnt} |")
    lines.append("")

    # Issues summary
    all_issues: list[str] = []
    for r in results:
        all_issues.extend(r.score.issues)
    issue_counts = Counter(all_issues)
    if issue_counts:
        lines += ["## Common Issues", ""]
        lines += ["| Issue | Occurrences |", "|-------|-------------|"]
        for issue, cnt in issue_counts.most_common(10):
            lines.append(f"| {issue} | {cnt} |")
        lines.append("")

    # Per-document breakdown
    lines += ["## Per-Document Results", ""]
    from itertools import groupby
    results_sorted = sorted(results, key=lambda r: r.pdf)
    for pdf_name, grp in groupby(results_sorted, key=lambda r: r.pdf):
        group = list(grp)
        useful_n = sum(1 for r in group if r.score.is_useful_crop)
        lines += [
            f"### {pdf_name}",
            "",
            f"**{useful_n}/{len(group)} useful crops**",
            "",
            "| Asset | Page | Size | Area% | sig | cq | cmp | Type | Useful | Issues |",
            "|-------|------|------|-------|-----|----|-----|------|--------|--------|",
        ]
        for r in group:
            s = r.score
            issues_str = "; ".join(s.issues) if s.issues else "—"
            err_str = f"ERROR:{s.error}" if s.error else ""
            useful_str = "✓" if s.is_useful_crop else "✗"
            lines.append(
                f"| `{r.asset_path}` | {r.page} | {r.width}×{r.height} "
                f"| {r.area_frac_px*100:.1f}% "
                f"| {s.signal} | {s.crop_quality} | {s.completeness} "
                f"| {s.figure_type} | {useful_str} | {err_str or issues_str} |"
            )
        lines.append("")

    # Not-useful crops section (actionable)
    if not_useful:
        lines += ["## Not-Useful Crops (Actionable)", ""]
        lines += [
            "These crops were flagged by the judge as low-signal, poorly cropped, "
            "or incomplete. Review whether extraction parameters need adjustment.",
            "",
        ]
        for r in not_useful:
            s = r.score
            lines.append(
                f"- **{r.pdf}** `{r.asset_path}` "
                f"sig={s.signal} cq={s.crop_quality} cmp={s.completeness} "
                f"type={s.figure_type}"
                + (f" — {'; '.join(s.issues)}" if s.issues else "")
            )
        lines.append("")

    path = out_dir / "judge_report.md"
    path.write_text("\n".join(lines))
    print(f"Markdown report → {path}")


# ── Entry point ───────────────────────────────────────────────────────────────

def main() -> None:
    parser = argparse.ArgumentParser(description="Mistral Large vision judge for SPEC-049")
    parser.add_argument(
        "--e2e-dir",
        type=Path,
        default=DEFAULT_E2E_DIR,
        help=f"E2E output directory (default: {DEFAULT_E2E_DIR})",
    )
    args = parser.parse_args()

    e2e_dir: Path = args.e2e_dir.expanduser().resolve()
    if not e2e_dir.exists():
        sys.exit(f"ERROR: e2e dir not found: {e2e_dir}\nRun the stress tests first.")

    results = run_judge(e2e_dir)
    write_json_results(results, e2e_dir)
    write_markdown_report(results, e2e_dir)

    # Exit non-zero if >30% of crops are not useful (threshold for code review)
    useful_pct = 100 * sum(1 for r in results if r.score.is_useful_crop) // max(len(results), 1)
    print(f"\nUseful crop rate: {useful_pct}%")
    if useful_pct < 70:
        print("⚠ Below 70% — review extraction parameters.")
        sys.exit(1)
    else:
        print("✓ Useful crop rate ≥ 70% — extraction quality acceptable.")


if __name__ == "__main__":
    main()
