# 005 — Expert Developer Lens

**Cross-ref:** [003](./003-fair-evaluation-protocol.md) · [009](./009-implementation-plan.md) · [010](./010-smoke-then-full-runbook.md)

---

## 1. Recommended code layout (SRP / DRY)

```text
edgequake/crates/edgequake-bench/          # OR tools/bench047/ (Python) — pick one SSOT
  OR
tools/bench047/
  ├── pyproject.toml                      # isolated tooling env
  ├── README.md
  ├── bench047/
  │   ├── download.py                     # HF/GitHub cache
  │   ├── subset.py                       # smoke/core fixtures
  │   ├── ingest.py                       # EdgeQuake HTTP client
  │   ├── query.py                        # hybrid query loop
  │   ├── extract.py                      # GPT-4o / Mistral judge
  │   ├── score.py                        # vendor eval_score.py
  │   ├── report.py                       # scorecard.json + SUMMARY.md
  │   ├── profiles.py                     # locked Mistral env
  │   └── cli.py                          # `bench047 smoke|core|full`
  └── vendor/
      └── mmlongbench_eval_score.py       # pinned upstream copy + SHA
```

**Decision (spec default):** implement harness in **Python** under `tools/bench047/` because:

1. Upstream eval is Python.  
2. HF `datasets` download is trivial.  
3. No need to bloating Rust workspace for a research harness.  
4. EdgeQuake remains the SUT via HTTP — language-agnostic fairness.

Rust may later add a thin `make` wrapper only.

---

## 2. EdgeQuake API surface to use

| Step | API | Notes |
|------|-----|-------|
| Health | `GET /health` | Require `llm_provider_name=mistral`, embed dim 1024 |
| Workspace | create workspace API | Isolated slug |
| Upload | `POST /api/v1/documents` | PDF + vision flags |
| Status | document get/list | Wait `Completed` |
| Query | `POST /api/v1/query` | `mode=hybrid` |
| Optional | graph/entities | Diagnostics only |

Reuse patterns from SPEC-013 / SPEC-021 Mistral live proofs; do not invent a second upload path.

---

## 3. Critical code fixes before claiming vision Small

| File | Issue | Required change |
|------|-------|-----------------|
| `edgequake/models.toml` | `mistral-small-latest` has `supports_vision = false` | Set `true` if API supports vision (per Mistral docs) **or** document pinned dated id `mistral-small-2506` / Small 4 id with vision |
| `provider_setup.rs` | Comment: Small silently drops images | Align routing: if model supports vision, do not strip; if not, fail closed |
| Makefile Mistral defaults | Vision defaults to `pixtral-large-latest` | Bench profile overrides to Small; keep Makefile default unless product decision changes |

Developer rule: **capability flag must match runtime behavior.** Tests:

1. Unit: vision content retained for `mistral-small-latest` when flag true.  
2. Live smoke: one page image round-trip returns non-empty visual description.

---

## 4. CLI UX (easy to run)

```bash
bench047 download
bench047 smoke          # 10 docs
bench047 core           # ~40 docs
bench047 full           # 135 docs
bench047 report path/to/artifacts/smoke
bench047 doctor         # keys, health, vision flag, embed dim
```

Makefile sugar:

```makefile
bench047-smoke: ## SPEC-047 smoke (10 real PDFs, hybrid, Mistral Small+embed)
bench047-core:
bench047-full:
bench047-doctor:
```

Exit codes: `0` valid+gates pass · `1` valid but gates fail · `2` invalid run · `3` infra error.

---

## 5. Artifact layout

```text
specs/047-rag-evaluation/e2e/artifacts/
  smoke/
    meta.json                 # pins, env redacted, git sha
    ingest.jsonl
    predictions.jsonl         # long + short + score
    scorecard.json            # schema in 012
    SUMMARY.md                # human one-pager
    logs/
  core/
  full/
```

`artifacts/` gitignored except `README.md` explaining how to produce them.

---

## 6. Concurrency & idempotency

- Ingest: bounded parallelism (default 2) — vision rate limits.  
- Query: bounded parallelism (default 4).  
- Resume: skip completed `doc_id` / `question_id` if `--resume` and artifact rows exist.  
- Idempotent workspace create.

---

## 7. Testing strategy (developer)

| Layer | What |
|-------|------|
| Unit | `score.py` vs upstream fixtures (Int/Float/ANLS/List/unanswerable) |
| Contract | `doctor` against running backend |
| Smoke e2e | 1 tiny PDF fixture **plus** optional live 10-doc (nightly) |
| Golden | Commit `fixtures/expected_smoke_schema.json` (shape only, not scores) |

Never commit MMLongBench PDFs.

---

## 8. Expert developer acceptance

- [ ] One Python package, one CLI, three stages  
- [ ] Vendored scorer SHA checked in CI  
- [ ] Vision capability fix merged or run fails closed  
- [ ] Artifacts schema validated  

Next: [006 MLOps](./006-mlops-lens.md).
