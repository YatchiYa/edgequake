"""Business-friendly publish pack for SPEC-001 Acc dual-SUT runs.

Generates stakeholder docs from scorecard.json without overclaiming SOTA.
Primary entry: ``make bench`` → publish/latest/{BUSINESS_REPORT.md, EXEC_SUMMARY.txt}.
"""

from __future__ import annotations

import json
import os
import shutil
from pathlib import Path
from typing import Any

from .paths import ARTIFACTS_DIR, REPO_ROOT

PUBLISH_DIR = ARTIFACTS_DIR / "publish"
PUBLISH_LATEST = PUBLISH_DIR / "latest"

QUESTION_TYPE_LABELS = (
    ("Fact Retrieval", "Fact lookup"),
    ("Complex Reasoning", "Multi-hop reasoning"),
    ("Contextual Summarize", "Summarization"),
    ("Creative Generation", "Creative / open-ended"),
)


def publish_latest_dir() -> Path:
    PUBLISH_LATEST.mkdir(parents=True, exist_ok=True)
    return PUBLISH_LATEST


def _f(v: Any, digits: int = 3) -> str:
    if v is None:
        return "—"
    try:
        return f"{float(v):.{digits}f}"
    except (TypeError, ValueError):
        return "—"


def _acc_ci(scorecard: dict[str, Any]) -> dict[str, Any]:
    d = (scorecard.get("metrics") or {}).get("delta_eq_minus_lr") or {}
    return d.get("overall_acc_delta_ci") or {}


def ci_includes_zero(scorecard: dict[str, Any]) -> bool | None:
    """True if Δ Acc 95% CI includes 0; None if CI missing."""
    ci = _acc_ci(scorecard)
    if "ci_low" not in ci or "ci_high" not in ci:
        return None
    try:
        lo, hi = float(ci["ci_low"]), float(ci["ci_high"])
    except (TypeError, ValueError):
        return None
    return lo <= 0.0 <= hi


def l2_gates_clear(scorecard: dict[str, Any]) -> bool:
    """EQ ctx_rel ≥ 0.50 and evidence_recall drop vs LR ≤ 0.03 (018-style)."""
    eq_ret = ((scorecard.get("metrics") or {}).get("eq") or {}).get("retrieval") or {}
    lr_ret = ((scorecard.get("metrics") or {}).get("lr") or {}).get("retrieval") or {}
    eq_cr = eq_ret.get("overall_context_relevancy")
    eq_er = eq_ret.get("overall_evidence_recall")
    lr_er = lr_ret.get("overall_evidence_recall")
    if eq_cr is None or eq_er is None or lr_er is None:
        return False
    try:
        return float(eq_cr) >= 0.50 and (float(lr_er) - float(eq_er)) <= 0.03
    except (TypeError, ValueError):
        return False


def verdict_label(scorecard: dict[str, Any]) -> str:
    """Honest Acc verdict for business language."""
    includes0 = ci_includes_zero(scorecard)
    d = (scorecard.get("metrics") or {}).get("delta_eq_minus_lr") or {}
    delta = float(d.get("overall_acc") or 0.0)
    if includes0 is True:
        return "STATISTICAL TIE on answer quality"
    if includes0 is False:
        # CI excludes 0 — only claim "ahead" if L2 gates also clear for publishable win.
        if delta > 0 and l2_gates_clear(scorecard):
            return "EdgeQuake ahead on answer quality (CI excludes 0; L2 gates clear)"
        if delta > 0:
            return "EdgeQuake Acc point lead (CI excludes 0; L2 not yet publishable win)"
        if delta < 0:
            return "LightRAG ahead on answer quality (CI excludes 0)"
    # No CI — directional only
    if abs(delta) < 1e-6:
        return "Acc point estimates match (no CI)"
    if delta > 0:
        return "EdgeQuake Acc point estimate ahead (tie not ruled out — no CI)"
    return "LightRAG Acc point estimate ahead (tie not ruled out — no CI)"


def can_claim_beats_lightrag(scorecard: dict[str, Any]) -> bool:
    includes0 = ci_includes_zero(scorecard)
    d = (scorecard.get("metrics") or {}).get("delta_eq_minus_lr") or {}
    delta = float(d.get("overall_acc") or 0.0)
    return includes0 is False and delta > 0 and l2_gates_clear(scorecard)


def _winner(eq: Any, lr: Any, *, higher_better: bool = True) -> str:
    if eq is None or lr is None:
        return "—"
    try:
        e, l = float(eq), float(lr)
    except (TypeError, ValueError):
        return "—"
    if abs(e - l) < 1e-9:
        return "Tie"
    if higher_better:
        return "EdgeQuake" if e > l else "LightRAG"
    return "EdgeQuake" if e < l else "LightRAG"


def _type_acc(block: dict[str, Any], qtype: str) -> float | None:
    row = (block.get("by_type") or {}).get(qtype) or {}
    v = row.get("answer_correctness", row.get("rouge_score"))
    if v is None:
        return None
    try:
        return float(v)
    except (TypeError, ValueError):
        return None


def build_business_report_md(
    scorecard: dict[str, Any],
    *,
    archive_rel: str | None = None,
) -> str:
    m = scorecard.get("metrics") or {}
    eq = m.get("eq") or {}
    lr = m.get("lr") or {}
    d = m.get("delta_eq_minus_lr") or {}
    ops = scorecard.get("ops") or {}
    pins = scorecard.get("pins") or {}
    eq_ret = eq.get("retrieval") or {}
    lr_ret = lr.get("retrieval") or {}
    ci = _acc_ci(scorecard)
    verdict = verdict_label(scorecard)
    n = int(ops.get("n_questions") or 0)
    eq_acc = eq.get("overall_acc")
    lr_acc = lr.get("overall_acc")
    eq_cr = eq_ret.get("overall_context_relevancy")
    lr_cr = lr_ret.get("overall_context_relevancy")
    eq_er = eq_ret.get("overall_evidence_recall")
    lr_er = lr_ret.get("overall_evidence_recall")
    eq_p50 = ops.get("eq_query_p50_ms")
    lr_p50 = ops.get("lr_query_p50_ms")
    ratio = ops.get("eq_over_lr_p50_ratio")
    if ratio is None and eq_p50 and lr_p50 and int(lr_p50) > 0:
        ratio = round(int(eq_p50) / int(lr_p50), 3)

    ci_line = "not available"
    if "ci_low" in ci and "ci_high" in ci:
        ci_line = (
            f"[{float(ci['ci_low']):+.3f}, {float(ci['ci_high']):+.3f}] "
            f"(n={int(ci.get('n') or n)})"
        )
        if ci_includes_zero(scorecard):
            ci_line += " — includes 0 ⇒ tie"

    type_rows = []
    for key, label in QUESTION_TYPE_LABELS:
        ea = _type_acc(eq, key)
        la = _type_acc(lr, key)
        type_rows.append(
            f"| {label} | {_f(ea)} | {_f(la)} | {_winner(ea, la)} |"
        )

    allowed_beats = can_claim_beats_lightrag(scorecard)
    claim_allowed = (
        "“EdgeQuake beats LightRAG on Acc under these pins”"
        if allowed_beats
        else "“Peer / statistical tie with LightRAG on Acc under fair pins” "
        "(or “point estimate ahead” only if CI excludes 0)"
    )
    claim_forbidden = (
        "“Beats LightRAG” / “wins Acc” / “#1 GraphRAG-Bench” / “SOTA RAG” "
        "without CI excluding 0 and L2 gates"
        if not allowed_beats
        else "Silent promotion of labeled CE/protect as Acc default; UltraDomain / paper Table-2 cosplay"
    )

    archive_line = archive_rel or "(see history/ after archive)"
    llm = f"{pins.get('llm_provider', 'mistral')}/{pins.get('llm_model', 'mistral-small-latest')}"
    emb = f"{pins.get('embedding_provider', 'mistral')}/{pins.get('embedding_model', 'mistral-embed')}"
    fixture_id = str(
        (pins.get("fixture_id") if isinstance(pins, dict) else None)
        or scorecard.get("fixture_id")
        or ""
    )
    n_why = (
        "n=200 medical-mid (50/type) — bootstrap Acc CI is underpowered at smoke n=40; "
        "this is the defendable publish ladder before full core"
        if n >= 200
        else f"n={n} (smoke gate — not a release publish claim; use medical-mid n=200)"
    )

    lines = [
        "# EdgeQuake vs LightRAG — Business Performance Report",
        "",
        f"**Generated:** {scorecard.get('created_at_utc', '—')}  ",
        f"**Task:** GraphRAG-Bench Acc dual-SUT (July 2026 fair pins)  ",
        f"**Profile:** `{scorecard.get('profile_id', '—')}`  ",
        f"**Fixture:** `{fixture_id or '—'}` (n={n})  ",
        f"**Valid run:** `{scorecard.get('valid')}`"
        + (
            f" ({scorecard.get('invalid_reason')})"
            if scorecard.get("invalid_reason")
            else ""
        ),
        "",
        "## One-screen first principles",
        "",
        "```text",
        "  Task     GraphRAG-Bench/EQ-vs-LR  (same corpus · questions · judge · Mix↔Mix)",
        f"  Sample   {n_why}",
        f"  Acc      EQ {_f(eq_acc)} · LR {_f(lr_acc)} · Δ {_f(d.get('overall_acc'), 3)}",
        f"  Δ Acc CI {ci_line}",
        f"  L2       evidence recall EQ {_f(eq_er)} / LR {_f(lr_er)} · "
        f"ctx_rel EQ {_f(eq_cr)} / LR {_f(lr_cr)}",
        f"  Latency  query p50 EQ {eq_p50 if eq_p50 is not None else '—'} ms / "
        f"LR {lr_p50 if lr_p50 is not None else '—'} ms",
        f"  Verdict  {verdict}",
        "```",
        "",
        "## Verdict",
        "",
        "```text",
        f"  {verdict}",
        f"  Acc   EdgeQuake {_f(eq_acc)}  ·  LightRAG {_f(lr_acc)}  ·  Δ {_f(d.get('overall_acc'), 3)}",
        f"  Δ Acc 95% CI: {ci_line}",
        "```",
        "",
        "## What we tested",
        "",
        f"- Same GraphRAG-Bench medical questions (n={n}) for both systems",
        f"- Same generator/judge stack: `{llm}` · embeddings `{emb}`",
        "- Same Mix mode, matched top-k / chunk size, official `generation_eval` Acc",
        "- Fairness: Mix arms always on, RRF fusion, chunk 1200/100, related_chunk=5",
        "- L2 required: official `retrieval_eval` (evidence recall + context relevancy)",
        "- **Not** UltraDomain win-rates · **not** paper Table-2 (GPT-4o-mini + BGE)",
        f"- **Why this n:** {n_why}",
        "",
        "## Scorecard for decisions",
        "",
        "| Layer | Plain meaning | EdgeQuake | LightRAG | Winner |",
        "|-------|---------------|-----------|----------|--------|",
        f"| Answer quality (Acc) | Are answers roughly as good? | {_f(eq_acc)} | {_f(lr_acc)} | "
        f"{'Tie (CI)' if ci_includes_zero(scorecard) else _winner(eq_acc, lr_acc)} |",
        f"| Evidence coverage | Did we find the right sources? | {_f(eq_er)} | {_f(lr_er)} | "
        f"{_winner(eq_er, lr_er)} |",
        f"| Context cleanliness | Is the prompt low-noise? | {_f(eq_cr)} | {_f(lr_cr)} | "
        f"{_winner(eq_cr, lr_cr)} |",
        f"| Speed (query p50) | Time to answer (ms) | {eq_p50 if eq_p50 is not None else '—'} | "
        f"{lr_p50 if lr_p50 is not None else '—'} | {_winner(eq_p50, lr_p50, higher_better=False)} |",
        "",
    ]
    if ratio is not None:
        lines.append(f"- **EQ/LR p50 ratio:** {ratio}× (product SLO target ≤ 1.5×)")
        lines.append("")
    lines.extend(
        [
            "## By question type (Acc)",
            "",
            "| User need | EdgeQuake | LightRAG | Who leads |",
            "|-----------|-----------|----------|-----------|",
            *type_rows,
            "",
            "## July 2026 landscape",
            "",
            "On this fair Acc head-to-head, EdgeQuake is a **LightRAG-class GraphRAG peer** "
            "when Acc is statistically tied. In the GraphRAG-Bench literature (ICLR 2026), "
            "**HippoRAG2-class** systems define the aspirational **retrieval SOTA** "
            "(high evidence recall **and** high context relevancy with compact prompts). "
            "Absolute Acc numbers from the academic paper use different models and are "
            "**not** directly comparable to these Mistral Acc pins.",
            "",
            "## Allowed / forbidden external claims",
            "",
            "| Allowed | Forbidden |",
            "|---------|-----------|",
            f"| {claim_allowed} | {claim_forbidden} |",
            "| “Peer GraphRAG with production stack (Postgres, API, PDF pipeline)” | "
            "“#1 on GraphRAG-Bench” without matching paper protocol |",
            "| “Actively closing retrieval noise / multi-hop / latency gaps” | "
            "Silent Acc headline = CE+protect without promotion gate |",
            "| “Publish Acc on medical-mid n=200 under fair pins” | "
            "Publishing smoke n=40 as the release score |",
            "",
            "## How to reproduce",
            "",
            "```bash",
            "make bench                 # medical-mid Acc (n=200) + this publish pack",
            "make bench-warm            # query-only (auto latest warm EQ workspace)",
            "make bench001-smoke-acc    # daily smoke gate only (n=40; not release)",
            "```",
            "",
            f"- Pins: Mix arms on · RRF · chunk 1200/100 · top-k {pins.get('retrieve_topk', 30)} · "
            f"`{llm}` + `{emb}`",
            "",
            "## Pointers",
            "",
            f"- **This publish pack:** `specs/001-benchmark/e2e/artifacts/publish/latest/`",
            f"- **Archive:** `{archive_line}`",
            "- **Technical SUMMARY:** same folder / archive `SUMMARY.md`",
            "- **Static business brief:** `specs/001-benchmark/019-business-eq-vs-lightrag-and-rag.md`",
            "- **Acc honesty close:** `specs/001-benchmark/001-edgquake-improvements/018-e4-acc-tie-close.md`",
            "",
        ]
    )
    return "\n".join(lines)


def build_exec_summary(scorecard: dict[str, Any]) -> str:
    m = scorecard.get("metrics") or {}
    eq = m.get("eq") or {}
    lr = m.get("lr") or {}
    ops = scorecard.get("ops") or {}
    eq_ret = eq.get("retrieval") or {}
    lr_ret = lr.get("retrieval") or {}
    verdict = verdict_label(scorecard)
    eq_p50 = ops.get("eq_query_p50_ms")
    lr_p50 = ops.get("lr_query_p50_ms")
    ratio = ops.get("eq_over_lr_p50_ratio")
    if ratio is None and eq_p50 and lr_p50 and int(lr_p50) > 0:
        ratio = round(int(eq_p50) / int(lr_p50), 3)
    includes0 = ci_includes_zero(scorecard)
    d = (scorecard.get("metrics") or {}).get("delta_eq_minus_lr") or {}
    delta = float(d.get("overall_acc") or 0.0)
    if can_claim_beats_lightrag(scorecard):
        claim = "Publishable claim: EdgeQuake beats LightRAG on Acc (CI + L2)."
    elif includes0 is False and delta < 0:
        claim = (
            "Publishable claim: LightRAG ahead on Acc (Δ Acc CI excludes 0) — "
            "do not claim EdgeQuake beats LightRAG."
        )
    elif includes0 is True:
        claim = "Publishable claim: peer / statistical tie with LightRAG on Acc — not SOTA win."
    else:
        claim = (
            "Publishable claim: Acc point estimates only (no CI or L2 incomplete) — "
            "not a beat claim."
        )
    n = int(ops.get("n_questions") or 0)
    lines = [
        "EdgeQuake vs LightRAG — GraphRAG-Bench Acc (EXEC SUMMARY)",
        f"Verdict: {verdict}",
        f"Sample: n={n} medical"
        + (" (publish mid)" if n >= 200 else " (smoke gate — not release)"),
        f"Acc: EQ {_f(eq.get('overall_acc'))} · LR {_f(lr.get('overall_acc'))}",
        f"Context cleanliness (ctx_rel): EQ {_f(eq_ret.get('overall_context_relevancy'))} · "
        f"LR {_f(lr_ret.get('overall_context_relevancy'))}",
        f"Evidence coverage (recall): EQ {_f(eq_ret.get('overall_evidence_recall'))} · "
        f"LR {_f(lr_ret.get('overall_evidence_recall'))}",
        f"Query p50 ms: EQ {eq_p50 if eq_p50 is not None else '—'} · "
        f"LR {lr_p50 if lr_p50 is not None else '—'} · ratio {ratio if ratio is not None else '—'}×",
        claim,
        "Not paper Table-2 / UltraDomain; Mistral Acc pins. HippoRAG2 = aspirational retrieval SOTA.",
        "Reproduce: make bench  (n=200 medical-mid)",
        f"Publish: specs/001-benchmark/e2e/artifacts/publish/latest/",
        f"Valid: {scorecard.get('valid')} · {scorecard.get('created_at_utc', '')}",
    ]
    return "\n".join(lines) + "\n"


def print_business_verdict(scorecard: dict[str, Any]) -> None:
    """One-screen terminal box for make bench."""
    m = scorecard.get("metrics") or {}
    eq = m.get("eq") or {}
    lr = m.get("lr") or {}
    d = m.get("delta_eq_minus_lr") or {}
    ops = scorecard.get("ops") or {}
    eq_ret = eq.get("retrieval") or {}
    lr_ret = lr.get("retrieval") or {}
    n = int(ops.get("n_questions") or 0)
    verdict = verdict_label(scorecard)
    ci = _acc_ci(scorecard)
    if ci_includes_zero(scorecard):
        ci_bit = "CI includes 0"
    elif "ci_low" in ci:
        ci_bit = f"CI [{float(ci['ci_low']):+.2f}, {float(ci['ci_high']):+.2f}]"
    else:
        ci_bit = "no CI"
    clean_w = _winner(
        eq_ret.get("overall_context_relevancy"),
        lr_ret.get("overall_context_relevancy"),
    )
    speed_w = _winner(ops.get("eq_query_p50_ms"), ops.get("lr_query_p50_ms"), higher_better=False)
    fact_w = _winner(_type_acc(eq, "Fact Retrieval"), _type_acc(lr, "Fact Retrieval"))
    width = 62
    inner = width - 2

    def row(text: str) -> str:
        t = text[:inner].ljust(inner)
        return f"║ {t} ║"

    box = [
        "╔" + "═" * width + "╗",
        row(f"EdgeQuake vs LightRAG — GraphRAG-Bench Acc (n={n})"),
        row(f"Verdict: {verdict}"),
        row(
            f"Acc  EQ {_f(eq.get('overall_acc'))}  ·  LR {_f(lr.get('overall_acc'))}  ·  {ci_bit}"
        ),
        row(f"Cleaner context: {clean_w} · Speed: {speed_w} · Fact: {fact_w}"),
        row("Publish: specs/001-benchmark/e2e/artifacts/publish/latest/"),
        "╚" + "═" * width + "╝",
    ]
    print("\n".join(box))
    _ = d  # reserved for future Δ print


def publish_peer_dir(peer_id: str) -> Path:
    """Labeled peer pack path: ``publish/peers/<peer_id>/`` (never Acc ``latest``)."""
    safe = "".join(c if c.isalnum() or c in "-_." else "_" for c in peer_id.strip())
    if not safe:
        raise ValueError("BENCH001_PUBLISH_PEER must be a non-empty peer id")
    return PUBLISH_DIR / "peers" / safe


def write_publish_pack(
    scorecard: dict[str, Any],
    *,
    stage_dir: Path,
    archive_dir: Path | None = None,
    publish_dir: Path | None = None,
) -> Path:
    """Write BUSINESS_REPORT + EXEC_SUMMARY to stage, archive, and publish/latest.

    ``publish_dir`` defaults to the real stakeholder pack path. Tests should pass
    a temp directory so they never clobber ``publish/latest``.

    Env:
    - ``BENCH001_SKIP_PUBLISH_LATEST=1`` — do not write Acc ``publish/latest``
    - ``BENCH001_PUBLISH_PEER=<id>`` — also write ``publish/peers/<id>/`` (labeled peer)
    """
    archive_rel = None
    if archive_dir is not None:
        try:
            archive_rel = str(archive_dir.relative_to(REPO_ROOT))
        except ValueError:
            archive_rel = str(archive_dir)

    peer_id = (os.environ.get("BENCH001_PUBLISH_PEER") or "").strip()
    report_md = build_business_report_md(scorecard, archive_rel=archive_rel)
    exec_txt = build_exec_summary(scorecard)
    meta = {
        "created_at_utc": scorecard.get("created_at_utc"),
        "profile_id": scorecard.get("profile_id"),
        "valid": scorecard.get("valid"),
        "verdict": verdict_label(scorecard),
        "git_sha": (scorecard.get("pins") or {}).get("edgequake_git_sha"),
        "archive": archive_rel,
        "task_name": scorecard.get("task_name"),
        "can_claim_beats_lightrag": can_claim_beats_lightrag(scorecard),
        "n_questions": (scorecard.get("ops") or {}).get("n_questions"),
        "fixture_id": (scorecard.get("pins") or {}).get("fixture_id"),
    }
    if peer_id:
        meta["publish_peer"] = peer_id
        meta["labeled_peer"] = True
        meta["not_acc_headline"] = True

    targets: list[Path] = [stage_dir]
    if archive_dir is not None:
        targets.append(archive_dir)
    skip_latest = (os.environ.get("BENCH001_SKIP_PUBLISH_LATEST") or "").strip().lower() in {
        "1",
        "true",
        "yes",
        "on",
    }
    latest = publish_dir if publish_dir is not None else publish_latest_dir()
    if not skip_latest:
        targets.append(latest)

    peer_dir: Path | None = None
    if peer_id:
        peer_dir = publish_peer_dir(peer_id)
        targets.append(peer_dir)

    for dest in targets:
        dest.mkdir(parents=True, exist_ok=True)
        (dest / "BUSINESS_REPORT.md").write_text(report_md, encoding="utf-8")
        (dest / "EXEC_SUMMARY.txt").write_text(exec_txt, encoding="utf-8")
        (dest / "meta.json").write_text(json.dumps(meta, indent=2), encoding="utf-8")

    def _copy_tech(dest: Path) -> None:
        for name in ("scorecard.json", "SUMMARY.md"):
            src = stage_dir / name
            if src.exists():
                shutil.copy2(src, dest / name)
            elif archive_dir is not None and (archive_dir / name).exists():
                shutil.copy2(archive_dir / name, dest / name)

    if peer_dir is not None:
        _copy_tech(peer_dir)
        (peer_dir / "README.md").write_text(
            f"# Labeled publish peer — `{peer_id}`\n\n"
            "Not Acc headline. Acc SSOT remains `publish/latest/` (P0 medical-mid).\n\n"
            "- [BUSINESS_REPORT.md](./BUSINESS_REPORT.md)\n"
            "- [EXEC_SUMMARY.txt](./EXEC_SUMMARY.txt)\n"
            "- [SUMMARY.md](./SUMMARY.md)\n"
            "- [scorecard.json](./scorecard.json)\n",
            encoding="utf-8",
        )

    if skip_latest:
        return peer_dir if peer_dir is not None else (
            stage_dir if archive_dir is None else archive_dir
        )

    # Stable copies of technical artifacts into publish/latest
    _copy_tech(latest)

    # Pointer file for humans
    (latest / "README.md").write_text(
        "# Latest publishable Acc pack\n\n"
        "Generated by `make bench`.\n\n"
        "- [BUSINESS_REPORT.md](./BUSINESS_REPORT.md) — stakeholder one-pager\n"
        "- [EXEC_SUMMARY.txt](./EXEC_SUMMARY.txt) — email / PR blurb\n"
        "- [SUMMARY.md](./SUMMARY.md) — technical Acc summary\n"
        "- [scorecard.json](./scorecard.json) — machine-readable metrics\n",
        encoding="utf-8",
    )
    return latest
