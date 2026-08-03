# SPEC-094 Parse Scorecard

HTTP harness that sweeps a PDF golden set against `POST /api/v1/parse` and
writes a per-backend scorecard JSON.

## Quick start (CI / EdgeParse)

```bash
# Against a running server (make dev-bg)
cargo run -p edgequake --example parse_scorecard -- \
  http://127.0.0.1:8080 \
  ../legacy/edgequake-pdf/test-data \
  edgeparse \
  10 \
  /tmp/parse-scorecard.json
```

Args: `<base_url> <golden_dir> [backend=edgeparse] [limit=50] [out=parse-scorecard.json]`

## Golden set

For local CI-friendly runs, use the small fixtures under:

- `legacy/edgequake-pdf/test-data/001_simple_text.pdf`
- `legacy/edgequake-pdf/test-data/008_multi_page_5_pages.pdf`
- `edgequake/crates/edgequake-pdf/test-data/embedded_figure_sample.pdf`

Or point the golden dir at a 50-document corpus for full matrix sweeps.

## Acceptance checks

- Scorecard includes `total_ms`, `pages_per_second`, `fallback_applied`, failure rate
- Harness fails if `edgequake-parse*` temp dirs remain after the sweep
