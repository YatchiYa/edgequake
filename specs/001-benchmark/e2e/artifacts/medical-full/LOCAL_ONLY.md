# Local-only raw artifacts

`predictions_*.json` and `eval_*.json` for medical-full (n≈2062) exceed GitHub’s 100MB limit and are gitignored.

Keep committed: `scorecard.json`, `SUMMARY.md`, `BUSINESS_REPORT.md`, `EXEC_SUMMARY.txt`, `meta.json`, `progress.json`.

Regenerate locally: `make bench001-medical-full-lr-occ-fact-l2` / `make bench001-medical-full-p0`.
