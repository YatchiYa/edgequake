#!/usr/bin/env python3
"""
SPEC-049  Two-Pass LLM Figure Pipeline — Mistral Large
=======================================================

First-principles rationale
--------------------------
Geometry (pdfium L0/L1) is a **conservative proposer**: it intentionally
keeps more crops than needed so it never misses a real figure.  Geometry alone
cannot distinguish a vector bar-chart from a decorated text box — both are
rectangular path fills.

The LLM is the **semantic oracle**.  Two passes are cheaper and more accurate
than any geometry heuristic:

  Pass 1 — FILTER  (cheap, fast model)
    Question: "Is this crop a real visual figure?"
    Drop: logo, text_block, icon_logo, empty, decorative_rule
    Keep: chart, diagram, photo, illustration, system_demo, table_visual

  Pass 2 — SPECIALIZE  (capable model, kind-aware prompt)
    Question: "What does this figure say?"  — prompt is tuned per kind:
      chart        → extract axes, series, key data values
      diagram      → list components, relationships, flow
      flowchart    → enumerate steps and decision branches
      system_demo  → describe pipeline stages and data shown
      illustration → write a descriptive caption
      table_visual → reconstruct as Markdown table

Output
------
  judge_results.json   machine-readable per-crop JSON
  judge_report.md      human-readable quality report
  filtered/            symlinks to kept PNGs only (for RAG ingestion)
  filtered_report.json filtered assets with descriptions (RAG-ready)

Usage
-----
  python3 specs/049-improve-figure-extraction/scripts/llm_judge_two_pass.py

Environment
-----------
  MISTRAL_API_KEY  — required
  SPEC049_E2E_DIR  — override e2e output directory
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import sys
import time
from collections import Counter
from dataclasses import asdict, dataclass, field
from itertools import groupby
from pathlib import Path
from typing import Optional

from mistralai.client import Mistral

# ── Config ────────────────────────────────────────────────────────────────────

PASS1_MODEL = "mistral-large-latest"    # filter pass: cheap is enough
PASS2_MODEL = "mistral-large-latest"    # specialize pass: need reasoning quality
RETRY_ATTEMPTS = 3
RETRY_DELAY_S  = 1.5

# Kinds that are real figures → proceed to Pass 2
FIGURE_KINDS = {
    "chart", "bar_chart", "line_chart", "scatter_plot", "pie_chart",
    "heatmap", "histogram", "radar_chart",
    "architecture_diagram", "flowchart", "diagram",
    "illustration", "photograph",
    "system_demo",        # e.g. RAG pipeline output demo screenshots
    "table_visual",       # raster/visual table (not text-native)
}

# Kinds that are NOT real figures → discard after Pass 1
NOISE_KINDS = {
    "logo", "icon_logo", "icon",
    "text_block",
    "decorative_rule", "empty", "whitespace",
    "other",            # when model is uncertain, keep (conservative)
}

DEFAULT_E2E_DIR = (
    Path(__file__).parent.parent / "e2e" / "markdown"
)

# ── Pass 1: Filter prompt (first-principles) ─────────────────────────────────
#
# Axiom (SPEC-049/001 modality-split): A visual crop delivers unique value only
# when its content CANNOT be expressed by plain text or a Markdown table alone.
# The LLM is the authoritative judge of this — geometry cannot decide it.

PASS1_SYSTEM = """\
You are a visual-content filter for a scientific PDF figure extraction pipeline.

Your task: decide whether a cropped image from a research paper is a REAL FIGURE
with independent visual signal, or a FALSE POSITIVE (logo, text box, icon,
decorative element).

First-principles axiom: a visual crop has value only when text/Markdown CANNOT
carry the same meaning (e.g. a bar chart, architecture diagram, or photograph).

Classify into exactly one kind:

REAL FIGURES (keep):
  "bar_chart"          — bars showing comparative data
  "line_chart"         — lines over time / continuous variable
  "scatter_plot"       — individual data points on axes
  "heatmap"            — colour-encoded 2D matrix of values
  "histogram"          — frequency distribution bars
  "pie_chart"          — proportional slices
  "radar_chart"        — multi-axis spider/radar chart
  "architecture_diagram" — boxes/arrows showing system structure
  "flowchart"          — sequential steps with decision branches
  "diagram"            — generic technical diagram
  "illustration"       — drawing, photo, or custom graphic
  "photograph"         — real-world photo
  "system_demo"        — screenshot of a running system (coloured sections
                         showing pipeline stages, inputs/outputs)
  "table_visual"       — a TABLE rendered as an image (scanned/screenshot;
                         NOT a text-native LaTeX table)

FALSE POSITIVES (discard):
  "logo"               — organisation/product logo
  "icon_logo"          — small icon or favicon
  "text_block"         — paragraph, abstract, heading, or caption text
  "decorative_rule"    — horizontal/vertical rule, border
  "empty"              — blank or near-blank area

Respond in JSON ONLY — no markdown fences, no explanation:
{"kind": "<one of the above>", "is_figure": <true|false>, "confidence": <0.0-1.0>}"""

PASS1_USER = "Classify this PDF crop — is it a real figure or a false positive?"

# ── Pass 2: Specialised extraction prompts ────────────────────────────────────

PASS2_SYSTEM_BASE = """\
You are an expert at extracting structured information from scientific figures.
You will receive a figure crop from a research paper.
Extract the requested information accurately and concisely.
Respond in Markdown."""

PASS2_PROMPTS: dict[str, str] = {
    "bar_chart": """\
Extract from this bar chart:
1. **Chart title** (if visible)
2. **X-axis label** and tick values
3. **Y-axis label** and range
4. **Series/groups** (legend entries)
5. **Key observations** (2–3 bullet points about the data)
6. **Data table** (Markdown table with the approximate bar values)""",

    "line_chart": """\
Extract from this line chart:
1. **Chart title** (if visible)
2. **X-axis** label and range
3. **Y-axis** label and range
4. **Series** names and trend direction
5. **Key observations** (2–3 bullet points)
6. **Approximate data** (Markdown table if readable)""",

    "scatter_plot": """\
Extract from this scatter plot:
1. **Chart title** (if visible)
2. **X-axis** and **Y-axis** labels
3. **Groups/clusters** visible (colour/shape coding)
4. **Key observations** (2–3 bullet points about patterns/outliers)""",

    "heatmap": """\
Extract from this heatmap:
1. **Title** (if visible)
2. **Row labels** and **column labels**
3. **Colour scale** meaning (high/low values)
4. **Hotspot regions** — which cells stand out?
5. Reconstruct as a **Markdown table** if axes are readable.""",

    "histogram": """\
Extract from this histogram:
1. **X-axis** (variable, range, bin size)
2. **Y-axis** (frequency / count / density)
3. **Distribution shape** (skewed, bimodal, normal, …)
4. **Key peaks or outlier bins**""",

    "architecture_diagram": """\
Describe this architecture diagram:
1. **Top-level components** (list each named box/module)
2. **Data flow** — what flows between components and in what direction?
3. **External interfaces** (APIs, databases, users)
4. **Key design decisions** visible in the diagram""",

    "flowchart": """\
Describe this flowchart:
1. **Start and end** conditions
2. **Main steps** in order (numbered list)
3. **Decision branches** (condition → outcome)
4. **Loops or back-edges** (if any)""",

    "diagram": """\
Describe this technical diagram:
1. **Main elements** and their roles
2. **Relationships and connections** between elements
3. **Directional flow** (if present)
4. **Key takeaway** in one sentence""",

    "system_demo": """\
Describe this system demonstration screenshot:
1. **Pipeline stages** shown (list each labelled section)
2. **Input** to the system (if visible)
3. **Output / response** (if visible)
4. **Key observations** about the system behaviour shown""",

    "table_visual": """\
Reconstruct this visual table:
1. **Headers** (column names)
2. **Rows** (as a Markdown table)
3. **Notable values** (maxima, minima, highlighted cells)""",

    "illustration": """\
Write a descriptive caption for this illustration:
1. **What is depicted** (subject matter)
2. **Key visual elements** labelled (if any)
3. **Purpose in the paper context** (if inferrable)""",

    "photograph": """\
Describe this photograph:
1. **Subject** (what is shown)
2. **Key details** relevant to a scientific paper
3. **One-sentence caption** suitable for a figure caption""",

    "pie_chart": """\
Extract from this pie chart:
1. **Title** (if visible)
2. **Slices**: label and approximate percentage for each
3. **Key observation** (largest/smallest slice, notable groupings)""",

    "radar_chart": """\
Extract from this radar / spider chart:
1. **Axes** (dimension names around the spider)
2. **Series** compared (legend entries)
3. **Notable strengths/weaknesses** per dimension""",
}

DEFAULT_PASS2_PROMPT = """\
Describe the key content of this figure in 3–5 sentences.
Include any visible labels, data, or structural elements."""


# ── Mistral helpers ───────────────────────────────────────────────────────────

def build_client() -> Mistral:
    key = os.environ.get("MISTRAL_API_KEY", "")
    if not key:
        sys.exit("ERROR: MISTRAL_API_KEY not set")
    return Mistral(api_key=key)


def encode_image(path: Path) -> str:
    return base64.standard_b64encode(path.read_bytes()).decode()


def call_vision(
    client: Mistral,
    model: str,
    system: str,
    user_text: str,
    img_path: Path,
    max_tokens: int = 600,
) -> str:
    b64 = encode_image(img_path)
    for attempt in range(RETRY_ATTEMPTS):
        try:
            resp = client.chat.complete(
                model=model,
                messages=[
                    {"role": "system", "content": system},
                    {
                        "role": "user",
                        "content": [
                            {
                                "type": "image_url",
                                "image_url": {"url": f"data:image/png;base64,{b64}"},
                            },
                            {"type": "text", "text": user_text},
                        ],
                    },
                ],
                max_tokens=max_tokens,
                temperature=0.0,
            )
            return resp.choices[0].message.content.strip()
        except Exception as exc:
            if attempt < RETRY_ATTEMPTS - 1:
                time.sleep(RETRY_DELAY_S * (attempt + 1))
            else:
                raise
    raise RuntimeError("exhausted retries")


# ── Pass 1 ────────────────────────────────────────────────────────────────────

@dataclass
class Pass1Result:
    kind: str
    is_figure: bool
    confidence: float
    raw: str
    error: Optional[str] = None


def run_pass1(client: Mistral, img_path: Path) -> Pass1Result:
    try:
        raw = call_vision(client, PASS1_MODEL, PASS1_SYSTEM, PASS1_USER, img_path,
                          max_tokens=80)
        # Strip accidental fences
        if raw.startswith("```"):
            raw = raw.split("```")[1].lstrip("json").strip()
        data = json.loads(raw)
        kind = str(data.get("kind", "other")).lower().replace(" ", "_")
        # Normalise aliases
        kind = _normalise_kind(kind)
        is_fig = bool(data.get("is_figure", kind in FIGURE_KINDS))
        conf = float(data.get("confidence", 0.5))
        return Pass1Result(kind=kind, is_figure=is_fig, confidence=conf, raw=raw)
    except Exception as exc:
        return Pass1Result(kind="other", is_figure=True, confidence=0.0,
                           raw="", error=str(exc))


def _normalise_kind(kind: str) -> str:
    aliases = {
        "architecture": "architecture_diagram",
        "flow_chart":   "flowchart",
        "photo":        "photograph",
        "icon":         "icon_logo",
        "text":         "text_block",
        "empty_page":   "empty",
    }
    return aliases.get(kind, kind)


# ── Pass 2 ────────────────────────────────────────────────────────────────────

@dataclass
class Pass2Result:
    description: str
    prompt_used: str
    error: Optional[str] = None


def run_pass2(client: Mistral, img_path: Path, kind: str) -> Pass2Result:
    prompt = PASS2_PROMPTS.get(kind, DEFAULT_PASS2_PROMPT)
    try:
        desc = call_vision(client, PASS2_MODEL, PASS2_SYSTEM_BASE, prompt,
                           img_path, max_tokens=600)
        return Pass2Result(description=desc, prompt_used=prompt[:80])
    except Exception as exc:
        return Pass2Result(description="", prompt_used=prompt[:80], error=str(exc))


# ── Composite result ──────────────────────────────────────────────────────────

@dataclass
class FigureJudgement:
    pdf: str
    asset_rel: str      # relative to document dir, e.g. "assets/p03-fig-01.png"
    page: int
    index: int
    label: str
    width: int
    height: int
    area_frac_px: float
    source: str
    # Pass 1
    kind: str
    is_figure: bool
    confidence: float
    p1_error: Optional[str]
    # Pass 2 (only if is_figure)
    description: str = ""
    p2_error: Optional[str] = None


# ── Main pipeline ─────────────────────────────────────────────────────────────

def run_two_pass(e2e_dir: Path) -> list[FigureJudgement]:
    client = build_client()
    results: list[FigureJudgement] = []

    doc_dirs = sorted(
        [d for d in e2e_dir.iterdir() if d.is_dir() and not d.name.startswith(".")],
        key=lambda d: d.name,
    )
    if not doc_dirs:
        sys.exit(f"ERROR: no document directories found under {e2e_dir}")

    total_crops = sum(
        len(json.loads((d / "report.json").read_text())["regions"])
        for d in doc_dirs
        if (d / "report.json").exists()
    )
    print(f"Two-pass LLM pipeline: {len(doc_dirs)} docs, {total_crops} crops")
    print(f"  Pass 1 (filter): {PASS1_MODEL}")
    print(f"  Pass 2 (specialize): {PASS2_MODEL}\n")

    done = kept = 0
    for doc_dir in doc_dirs:
        rp = doc_dir / "report.json"
        if not rp.exists():
            continue
        report = json.loads(rp.read_text())
        regions = report["regions"]
        print(f"  {report['pdf']} — {len(regions)} crops")

        for region in regions:
            asset_rel = region["asset_path"]
            asset_full = doc_dir / asset_rel
            if not asset_full.exists():
                print(f"    MISSING {asset_rel}")
                continue

            done += 1
            print(f"    [{done}/{total_crops}] {asset_rel} ({region['width']}×{region['height']}) ",
                  end="", flush=True)

            # ── Pass 1: filter ────────────────────────────────────────────────
            p1 = run_pass1(client, asset_full)
            if p1.error:
                print(f"P1-ERROR:{p1.error}")
            else:
                print(f"P1:{p1.kind}({'✓' if p1.is_figure else '✗'}) ", end="", flush=True)

            # ── Pass 2: specialize (only if kept) ─────────────────────────────
            description = ""
            p2_error = None
            if p1.is_figure:
                p2 = run_pass2(client, asset_full, p1.kind)
                if p2.error:
                    p2_error = p2.error
                    print(f"P2-ERROR:{p2.error}")
                else:
                    description = p2.description
                    kept += 1
                    print(f"P2:ok ✓")
            else:
                print("→ discarded")

            results.append(FigureJudgement(
                pdf=report["pdf"],
                asset_rel=asset_rel,
                page=region["page"],
                index=region["index"],
                label=region["label"],
                width=region["width"],
                height=region["height"],
                area_frac_px=region["area_frac_px"],
                source=region["source"],
                kind=p1.kind,
                is_figure=p1.is_figure,
                confidence=p1.confidence,
                p1_error=p1.error,
                description=description,
                p2_error=p2_error,
            ))

    print(f"\n{'─'*60}")
    print(f"Total crops: {total_crops}  |  Kept (real figures): {kept}  |  Discarded: {total_crops - kept}")
    return results


# ── Filtered asset writer ─────────────────────────────────────────────────────

def write_filtered_assets(results: list[FigureJudgement], e2e_dir: Path) -> None:
    """Write per-document filtered/ subdirs with kept PNGs and descriptions."""
    kept = [r for r in results if r.is_figure and r.description]
    for pdf_name, grp in groupby(sorted(kept, key=lambda r: r.pdf), lambda r: r.pdf):
        group = list(grp)
        stem = Path(pdf_name).stem
        doc_dir = e2e_dir / stem
        filtered_dir = doc_dir / "filtered"
        filtered_dir.mkdir(exist_ok=True)

        rag_assets = []
        for r in group:
            src = doc_dir / r.asset_rel
            dst = filtered_dir / Path(r.asset_rel).name
            if src.exists():
                import shutil
                shutil.copy2(src, dst)
            rag_assets.append({
                "asset": Path(r.asset_rel).name,
                "page": r.page,
                "label": r.label,
                "kind": r.kind,
                "description": r.description,
            })

        # RAG-ready JSON per document
        (filtered_dir / "rag_assets.json").write_text(
            json.dumps(rag_assets, indent=2)
        )

    print(f"Filtered assets written to {e2e_dir}/<doc>/filtered/")


# ── Report writers ────────────────────────────────────────────────────────────

def write_json(results: list[FigureJudgement], out_dir: Path) -> None:
    data = [asdict(r) for r in results]
    (out_dir / "judge_results.json").write_text(json.dumps(data, indent=2))
    print(f"JSON → {out_dir}/judge_results.json")


def write_markdown(results: list[FigureJudgement], out_dir: Path) -> None:
    kept   = [r for r in results if r.is_figure]
    discarded = [r for r in results if not r.is_figure]
    errors = [r for r in results if r.p1_error]

    kind_counts = Counter(r.kind for r in results if not r.p1_error)
    useful_pct  = 100 * len(kept) // max(len(results), 1)

    lines = [
        "# SPEC-049 Two-Pass LLM Judge Report",
        "",
        f"> Pass 1 (filter): `{PASS1_MODEL}`  |  Pass 2 (specialize): `{PASS2_MODEL}`",
        f"> Total crops: {len(results)}  |  Kept: **{len(kept)}** ({useful_pct}%)  |  Discarded: {len(discarded)}",
        "",
        "## Summary",
        "",
        "| Metric | Value |",
        "|--------|-------|",
        f"| Proposed by L0/L1 geometry | {len(results)} |",
        f"| Kept (real figures, Pass 1) | **{len(kept)}** ({useful_pct}%) |",
        f"| Discarded (noise, Pass 1)   | {len(discarded)} |",
        f"| Pass-2 descriptions written | {sum(1 for r in kept if r.description)} |",
        f"| Errors | {len(errors)} |",
        "",
        "## Figure Type Distribution (post-filter)",
        "",
        "| Kind | Count |",
        "|------|-------|",
    ]
    for kind, cnt in kind_counts.most_common():
        marker = "✓" if kind in FIGURE_KINDS else "✗"
        lines.append(f"| {marker} {kind} | {cnt} |")
    lines.append("")

    # Discarded crops
    if discarded:
        lines += [
            "## Discarded Crops (noise eliminated by Pass 1)",
            "",
            "| Doc | Asset | Kind | Confidence |",
            "|-----|-------|------|------------|",
        ]
        for r in discarded:
            lines.append(
                f"| {r.pdf} | `{r.asset_rel}` | {r.kind} | {r.confidence:.2f} |"
            )
        lines.append("")

    # Per-document results with descriptions
    lines += ["## Per-Document Results", ""]
    results_sorted = sorted(results, key=lambda r: r.pdf)
    for pdf_name, grp in groupby(results_sorted, lambda r: r.pdf):
        group = list(grp)
        kept_n = sum(1 for r in group if r.is_figure)
        lines += [
            f"### {pdf_name}",
            "",
            f"**{kept_n}/{len(group)} real figures**",
            "",
        ]
        for r in group:
            status = "✓" if r.is_figure else "✗"
            lines += [
                f"#### {status} `{r.asset_rel}` — {r.kind} (p{r.page})",
                "",
            ]
            if r.is_figure and r.description:
                lines += [r.description, ""]
            elif not r.is_figure:
                lines += [f"*Discarded: {r.kind} (confidence={r.confidence:.2f})*", ""]
        lines.append("")

    path = out_dir / "judge_report.md"
    path.write_text("\n".join(lines))
    print(f"Markdown → {path}")


# ── Entry point ───────────────────────────────────────────────────────────────

def main() -> None:
    parser = argparse.ArgumentParser(
        description="Two-pass Mistral Large judge: filter + specialise per figure type"
    )
    parser.add_argument("--e2e-dir", type=Path, default=DEFAULT_E2E_DIR)
    args = parser.parse_args()

    e2e_dir = args.e2e_dir.expanduser().resolve()
    if not e2e_dir.exists():
        sys.exit(f"ERROR: {e2e_dir} not found — run stress tests first")

    results = run_two_pass(e2e_dir)
    write_json(results, e2e_dir)
    write_markdown(results, e2e_dir)
    write_filtered_assets(results, e2e_dir)

    kept_pct = 100 * sum(1 for r in results if r.is_figure) // max(len(results), 1)
    print(f"\nFigure retention rate: {kept_pct}%")
    if kept_pct < 50:
        print("⚠ geometry is over-proposing — review L1 thresholds")
    else:
        print("✓ retention rate healthy (≥50%)")


if __name__ == "__main__":
    main()
