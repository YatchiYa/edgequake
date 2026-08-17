# 05 — Issue #3: INV-03 indexed documents without chunks

**Crit:** High · **Volume:** 24 (hourly CRITICAL) · **Law:** LAW-I2 + SPEC-091 LAW-D6 · **E2E:** E2E-104-03

## Symptom (prod, 0.22.0)

20 documents with `status = 'indexed'` and no KV keys `{doc_id}-chunk-%`. Inspector logs CRITICAL drift each hour.

## Why V22 has it (real drift)

```ascii
 Ingest SAGA (KV era)
   write index / status='indexed'
        │
        ├─ chunk KV writes succeed ──▶ healthy
        │
        └─ chunk KV fail / delete residue / partial compensate
                 │
                 ▼
           INV-03 finds ≥10 docs ──▶ CRITICAL (correct alarm)
```

Possible causes (ops to confirm per doc id): partial SAGA without rollback of `status`, delete of chunk keys without status revert, legacy import.

## Why V23 makes it worse (silent)

```ascii
 SPEC-091 mig 125 drops eq_*_kv
        │
        ▼
 INV-03 still queries {kv_table}
        │
        ▼
 if !kv_exists { return; }   -- SILENT SUCCESS
        │
        ▼
 public.chunks is SSOT but never checked
        │
        ▼
 False green health after "successful" upgrade
```

## Remediation

1. INV-03 dual presence (harden):

```sql
-- fire only when NEITHER store proves chunks
indexed AND NOT EXISTS (chunks)
  AND (NOT kv_exists OR NOT EXISTS (kv '{id}-chunk-%'))
```

2. Critical if `public.chunks` relation missing.
3. EC-16 closed — safe on KV-era and post-125.

## Fix status (2026-08-03 harden)

**Closed.** Grade A− ([13](13-fix-assessment.md) v2). Migration: no new mig; dual-read removes wrong-era false CRITICAL.
