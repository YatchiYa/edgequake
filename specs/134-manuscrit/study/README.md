# SPEC-134 Slice E — page-as-unit study

Private ablation: **same VLM, only the input image / prompt changes**.
Never commit trigger PDFs, renders, or transcriptions (LAW-134-10).

## Run

```bash
export SPEC134_STUDY_PDF="/path/to/private-manuscript.pdf"
# Optional: comma-separated recall tokens in a local file (not committed)
# export SPEC134_STUDY_GOLD="$PWD/gold.local.json"
# Vision endpoint (defaults match EdgeQuake OpenAI-compatible env).
# The harness also loads gitignored `study/.env` and repo `.env` for unset keys.
# First OpenAI run in this pack was HTTP 401 (invalid project key); the live
# go/no-go used Mistral: OPENAI_BASE_URL=https://api.mistral.ai/v1 and
# EDGEQUAKE_VISION_MODEL=mistral-small-latest (compiled Mistral vision default).
python3 specs/134-manuscrit/study/page_as_unit.py
```

Outputs: `specs/134-manuscrit/study/out/` (gitignored).

## Ablations

| Condition | Why |
|-----------|-----|
| long-edge 1024 / 2000 / 3600 | ImageGuard floor vs today’s Pass-A vs intended MS |
| PNG vs JPEG q85 | guard re-encode |
| print Acc prompt vs MS prompt | DISCO task-aware prompting |
| whole page vs crop gallery | LAW-134-20 |

## Go / no-go

Port to Rust when whole-page ≥2000px + MS prompt recovers a readable body
on the private pages. Fill the table in
[`11-honest-assessment.md`](../11-honest-assessment.md) from `out/summary.json`
without quoting page content. Live 2026-08-22 run (`mistral-small-latest`):
whole-page empty_rate 0.0 at 1024/2000/3600; crop gallery empty_rate 0.0 but
`frenchish_pages=0` vs `1` for whole-page — still **go**.
