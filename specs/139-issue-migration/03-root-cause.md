# 03 — Root cause (code is law)

> Pre-fix = v0.26.1 shipped engine. Post-fix = this pack.

## Track A — iw2 21000 (primary crash)

`write_entity_batch` builds parallel arrays and:

```sql
INSERT INTO entity_embeddings (model_id, entity_id, …)
SELECT … FROM unnest($2::uuid[], …)
ON CONFLICT (model_id, entity_id) DO UPDATE
  SET legacy_vector_id = COALESCE(entity_embeddings.legacy_vector_id, EXCLUDED.legacy_vector_id)
```

`EntityNameIndex::resolve` uses `normalize_entity_name`. Two legacy ids in one
`ORDER BY id LIMIT n` batch (`entity:Acme Corp Ltd` and `entity:ACME_CORP_LTD`)
share one `entity_id`. Postgres: **cannot affect row a second time**.

The file header claimed `ON CONFLICT DO NOTHING` (which allows duplicate
proposed keys). The SQL is DO UPDATE (required for provenance COALESCE).

Serving upsert in `storage_impl.rs` already last-write-wins within the batch
(QW2). iw2 did not. Same class as SPEC-110 migration 118.

First field batch failed ~137ms after lease claim → `uncovered_fleet=521076`
never moved. `run_batch` `Err` rolls back the TX; job stays `preflight` (retryable)
but every retry hits the same 21000 until the binary changes.

Relationship batches have the same DO UPDATE on `(model_id, relationship_id)`.

## Track B — W3 verify accounting + terminal failed

`verify_chunk_embedding_backfill` sets `actual = COUNT(*) FROM chunk_embeddings`
(global) and `expected = COUNT` of `-chunk-` rows **in one** legacy table.

Fleet aggregate:

```text
agg.expected += r.expected   -- SUM across tables
agg.actual = agg.actual.max(r.actual)  -- MAX of the same global count
```

Field: expected 44580, actual 18503. Even a complete copy cannot match
`SUM(legacy)` vs `max(global typed)` once there are many `eq_*_vectors` tables.

Verify FAIL → `finish_job(..., "failed")`. `claim_lease` only claims
`pending|preflight|running|paused`. Subsequent boots **skip** W3. Remaining
uncovered chunks stay uncovered even after W1 adds spine.

`VerifyReport::passes()` also required `mismatches == 0` by default. Field
sampled 1370/2416 mismatches (halfvec / missing spine / model name). Coverage
is the 126 drop gate; equality must be opt-in.

## Track C — 119 before 122, no remainder

sqlx order: 117 → 118 → **119 artifacts** → … → **122 shells**.

119: `EXISTS (SELECT 1 FROM documents d WHERE d.id = left(kv.key, 36)::uuid)`.
Keys whose shells land only in 122 are skipped. 119 is recorded successful.
Engine jobs were only W1 / W3 / iw2 / stamp.

Field plateau: lineage=1232, multimodal=58, doc_hash=1246, staging_hash=41
(constant across guards while chunk_text 18904 → 11).

## Track D — engine isolation

`run_engine` used `run_job(...).await?`. iw2 `Err` skipped
`iw2-fleet-provenance-stamp` (and would skip remainder jobs).

## What is not the root cause

| Hypothesis | Why rejected |
|------------|----------------|
| `--drop-confirm` misspelling | 0.26.1 drop log shows consent INCLUDED |
| Migration 149 | Additive; already applied |
| Wave D / 126 / 131 abort | Correct fail-closed on uncovered rows |
| `guard` should drop | LAW-137-6 read-only |
