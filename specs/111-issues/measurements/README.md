# measurements/

Gate logs and honesty artifacts for SPEC-111 (mirror SPEC-110 layout).

| Artifact | Role |
|----------|------|
| [`BRUTAL-HONESTY.md`](BRUTAL-HONESTY.md) | Living release-safety verdict |
| [`e2e111-release-safety-gates.txt`](e2e111-release-safety-gates.txt) | **Current** full gate transcript (fmt/clippy + closeout + clear-all) |
| [`e2e111-honesty-closeout-gates.txt`](e2e111-honesty-closeout-gates.txt) | Prior honesty closeout |
| [`e2e111-final-gates.txt`](e2e111-final-gates.txt) | Earlier Cluster A + clear-all snapshot |
| [`SUMMARY.md`](SUMMARY.md) | Gate table + ops notes |

Expected / present:

- [x] `SUMMARY.md` — gate table + ops notes
- [x] `e2e111-release-safety-gates.txt` — release-safety audit (2026-08-07)
- [x] `e2e111-honesty-closeout-gates.txt` / `e2e111-final-gates.txt`
- `e2e111-residue-cast-explain.txt` — optional EXPLAIN (Index Cond) on large fleets
