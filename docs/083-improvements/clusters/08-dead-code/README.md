# Cluster 08 — Dead code & false positives

> **Sprint**: 4  
> **Laws**: LAW-3, LAW-8  
> **Defects**: C-19 (RETRACTED), D-52, register §5 inventory

---

## WHY

~2k LOC and orphan artifacts inflate maintenance cost and confuse agents. Some “defects” were wrong (C-19). Inert cache is pure overhead.

## INVENTORY (verify call sites before delete)

| Item | Path | Action |
|------|------|--------|
| `pipeline/validation.rs` + `sanitizer.rs` | edgequake-pipeline | Delete if zero prod call sites (~1000 LOC) |
| `pipeline/cache.rs` / CachedExtractor | never `set` | Delete **or** wire set (prefer delete until needed) — [D-52](../../defects/D-52.md) |
| `SOTAExtractor` + prompts tuple | unwired; prod uses JSON path | Delete or feature-gate tests-only |
| `crates/edgequake-llm/` stub CHANGELOG | root crates | Remove orphan; real crate is deps `edgequake-llm` |
| Orphan CHANGELOGs under root `crates/` | | Remove |
| `default_recursive_separators` unused in prod | | Activate via X-14 (not delete) |
| `MergerConfig.description_decay / min_importance` | never read | Remove fields |
| C-19 `drop_workspace_table` | | **RETRACTED** — keep stub study only |

## PROCESS

```
  rg call sites --> if only self-tests, quarantine PR
                 --> cargo test -p affected
                 --> delete in dedicated PR (no behavior change)
```

## E2E

`contract_cache_set_or_module_removed` is **Backlog** while [D-52](../../defects/D-52.md) remains CONFIRMED. Until then: workspace builds after deletions; `rg` call-site proof on deletion PRs; no public API break without version note.
