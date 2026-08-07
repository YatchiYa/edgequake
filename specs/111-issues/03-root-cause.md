# 03 — Root cause (code is law)

> **Historical RCA (pre-fix on v0.24.1 discovery).** Current status is the README status board + [`measurements/BRUTAL-HONESTY.md`](measurements/BRUTAL-HONESTY.md). Current law is **coverage / provenance** (`uncovered_* == 0`, fleet = `legacy_vector_id`). See [`09-ops-runbook.md`](09-ops-runbook.md). Snippets below describe the **buggy HEAD at discovery**, not the v0.24.2 candidate.

## #364 — Advisor emptiness vs SQL coverage (CONFIRMED on HEAD / v0.24.1)

### What the code does

`VectorPosture::chunk_retirable` / `fleet_retirable` (`advisor/types.rs`):

```rust
&& self.legacy_chunk_rows == 0
// fleet also: && self.legacy_fleet_rows == 0
```

`count_legacy_chunk_rows` (`advisor/mod.rs`) is a live `COUNT(*)` on `eq_*_vectors` with `id LIKE '%-chunk-%'`. Backfill **copies**; it does not delete.

Migration **126** (`126_spec091_vector_drop.sql`):

1. **Guard:** abort if any legacy chunk row lacks typed `chunk_embeddings` coverage (correct).
2. **Then** `DELETE … WHERE id LIKE '%-chunk-%'` and drop empty tables.

Advisor actions (`rules.rs`) gate `drop vector-legacy` on `v.retirable()` — so dry-run shows:

```text
✗ drop vector-legacy — cannot drop … N legacy chunk rows un-migrated
```

even when every row is covered.

`--confirm-drop` still applies migrations; the **SQL** guard is the real safety. Partner correctly refused to force past a confusing RED.

### Drift-guard test gap

`contract_spec091_advisor_matches_126_guard` asserts retirable **after** full 126 apply (emptiness), not that pre-drop retirable ≡ guard pass. Comment admits: “after the drop drains legacy rows”.

### Secondary

`verify_chunk_embedding_backfill` / fleet verify require per-dim `|a-b| < 1e-3`. Regenerated embeddings (partner workaround for #363) fail verify even with 100% row coverage.

### Verdict

**Real product defect (advisor predicate + messaging).** Physical drop guard is sound. Still present on **v0.24.1** and current HEAD.

---

## #363 — Exact-name join + scan-as-processed (CONFIRMED)

### What the code does

`FleetEmbeddingBackfillJob::write_relationship_batch` (`fleet_embedding_backfill.rs`):

1. Parse legacy id via `parse_relationship_legacy_key` → `(src, tgt, rel_type)` (already uppercase/normalized key shape).
2. Lookup:

```sql
SELECT r.id FROM relationships r
JOIN entities es ON es.id = r.source_id
JOIN entities et ON et.id = r.target_id
WHERE es.name = $1 AND et.name = $2 AND r.relation_type = $3
  AND r.workspace_id = $4
```

3. On miss: `continue` — **no failed_count**.
4. Batch returns `scanned = rows.len()`, `written = inserts`.
5. Runner advances `processed_count` by **`scanned`** (`lease.rs` / `runner.rs`).

Entity path: `SELECT id FROM entities WHERE name = $1` — same exact equality; legacy ids are `entity:NORMALIZED_NAME` from `entity_name_from_legacy_id` (strip prefix only).

If `public.entities.name` holds display forms (`Acme Corp Ltd`) while legacy keys hold `ACME_CORP_LTD`, join miss rate → catastrophic; job still “completes”.

### Verdict

**Real P0.** Still present. Partner regenerate-from-spine workaround is valid ops escape hatch, not a product fix.

---

## #362 — Wrong-direction cast (CONFIRMED)

### What the code does

`residue.rs` RESIDUE_SQL / GUARD_TOTAL_SQL (and mirrored fragments in `125_spec091_kv_drop.sql`):

```sql
WHERE d.id::text = substring(k.key from '…uuid…')
-- also document_id::text = substring(...)
```

Same file already uses the correct direction for chunks / wsdoc / injection:

```sql
WHERE c.document_id = left(k.key, 36)::uuid
WHERE d.id = split_part(k.key, ':', 3)::uuid
```

Postgres cannot Index Cond on `(uuid)::text = text` → Filter + heap churn → timeout on tens of thousands of KV rows.

### Verdict

**Real P1.** Still present in advisor **and** migration 125 (must patch both for LAW-C3).

---

## #366 / #360 — Clear All incomplete (CONFIRMED on v0.24.1)

Partner clarified #360 env is **0.24.1** and opened [#366](https://github.com/raphaelmansuy/edgequake/issues/366) with the correct pin.

### What the code does (pre-fix)

1. Wipe RM1 deletes typed `documents` / chunks set-based and **skips** residual KV list-surface purge (`PurgingDocumentKv` no-op jump).
2. List membership uses relational `documents` ids (`workspace_document_index`).
3. When membership returns **empty**, `load_scoped_document_metadata_entries*` fell through to `keys_with_suffix("-metadata")` and re-hydrated dual-write KV ghosts into `GET /documents`.

Durable wipe (#309) fixed AGE N× prefix races; it did **not** close List ⊆ Wipe under dual-read.

### Verdict

**Real P1 on v0.24.1.** Fix: LAW-111-9 (authoritative empty terminal) + wipe residual KV purge via `plan_workspace_document_kv_deletion`. Deep dive: [issue-366-clear-all.md](issue-366-clear-all.md).

---

## #361 — Bulk upload slow (v0.12.11 report)

### Nature

Ingest is LLM + embed + graph write bound. Concurrency is intentionally capped (local VLM, pool, per-tenant fairness — see `pdf_processing.rs`, SPEC-090). “Multiple documents take long” is often **expected**.

### Verdict

**Capacity / product expectation**, not a confirmed logic bug. Cross-ref SPEC-090. Require timings (docs, pages, provider, worker count) before code change.

---

## Causal chain (Cluster A)

```ascii
  Long-lived DB (AGE names + legacy vectors + fat KV)
           │
           ├─► iw2 exact name join (#363) ──► written ≪ scanned
           │         │
           │         └─► processed_count≈100%, failed=0  (false GREEN)
           │
           ├─► partner regenerates embeddings ──► verify |a-b|<1e-3 fails (#364 secondary)
           │
           ├─► advisor retirable needs legacy COUNT=0 (#364) ──► dry-run RED forever pre-drop
           │         │
           │         └─► SQL 126 coverage guard would PASS if coverage real
           │
           └─► kv residue id::text (#362) ──► advisor timeout ──► “guard unavailable”
```
