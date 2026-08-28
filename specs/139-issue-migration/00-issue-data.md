# 00 — Issue data (anonymized)

> **Source:** Field ticket + operator logs. No production credentials in this file.
> **From:** mid-cutover serving · **Image:** `ghcr.io/raphaelmansuy/edgequake:0.26.1`
> **Raw:** [logs-folder/](logs-folder/) · redacted copies: [raw-logs/](raw-logs/)

## Command sequence (operator)

```text
1. edgequake migrate
2. edgequake guard          # RED
3. start edgequake (engine)
4. edgequake guard          # still RED
5. edgequake migrate --confirm-drop   # Wave D ABORT
```

## Facts extracted (code is law)

| Fact | Value |
|------|-------|
| Image | 0.26.1 — SPEC-137 CLI already present (`--drop-confirm` alias in drop log) |
| Expandable apply | 106–124, 128–130, 132–141, 143–149 (DROP 125/126/131 + 142 deferred) |
| Latest applied after migrate | 149 |
| `uncovered_fleet` | **521076** unchanged across every post-migrate guard |
| `uncovered_chunk` | 41114 → 21177 then stuck |
| W3 verify | `expected: 44580, actual: 18503, sampled: 2416, mismatches: 1370` |
| iw2 | `ON CONFLICT DO UPDATE command cannot affect row a second time` on first claimed batch |
| KV residue last guard | total 2596: chunk_text=11, doc_shells=4, lineage=1232, multimodal=58, doc_hash=1246, staging_hash=41, wsdoc=4 |
| Wave D abort | 2592 un-migrated durable KV rows |
| First guard (pre-migrate) | `document_artifacts` 42P01 — guard before SAFE SCHEMA |

## Track pin

| Track | Symptom | Root |
|-------|---------|------|
| **A** | Engine terminated; fleet uncovered frozen | iw2 within-batch arbiter dups → 21000 |
| **B** | W3 verify FAIL then job `failed` | Wrong `actual` aggregate + no reclaim |
| **C** | lineage/MM/hash never move | 119 one-shot before 122; no remainder job |
| **D** | Stamp never runs after iw2 crash | `run_engine` `?` on first job Err |

## What 0.26.1 already is

SPEC-137 consent/abort-class is working (drop log names Wave D, not `pg_locks`).
This pack does not re-litigate `--drop-confirm`.
