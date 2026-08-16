# 01 — First Principles (LAW-133)

## Domain

Typed embeddings store relationship vectors keyed by a **legacy composite string**:

```ascii
  legacy_rel_id  ::=  source  "->"  target  ":"  RELATION_TYPE

  Problem: source and target are free-form entity names.
           The delimiter alphabet {"->", ":"} is a subset of the name alphabet.
```

This is the same class of failure as CSV commas in fields: **delimiter collision**.
PostgreSQL FK integrity is not the bug — the application invents parent keys that
never existed ([FK ordering notes](https://errornotes.dev/en/errors/postgresql/fix-postgresql-insert-or-update-violates-foreign-key-constraint-error) apply only after wrong names are chosen).

## Identity planes

```ascii
  Plane A — Relational spine (authoritative for typed FK)
            entities.id / relationships.id (UUID)

  Plane B — Legacy vector id (string composite, historical)
            entity:NAME | SRC->TGT:TYPE | community_report:N

  Plane C — In-session sink map (SPEC-130)
            HashMap<legacy_rel_id, relationships.id>
```

Typed mirror must prefer **A via C**, then **A via disambiguated B**, never invent A.

## Laws

| ID | Law | Rationale |
|----|-----|-----------|
| LAW-133-1 | **Composite keys need collision strategy** — raw concatenation with delimiters that appear in fields is not invertible | First principles / delimiter collision |
| LAW-133-2 | **One parse SSOT** — `format_relationship_legacy_key` + parse helpers live in `embedding_family.rs`; all mirror/backfill/stamp call them | DRY |
| LAW-133-3 | **Index-guided disambiguation before fail** — when an `EntityNameIndex` is available, choose the `->` split where **both** endpoints resolve; unique both-resolve wins | Correctness without key migration |
| LAW-133-4 | **SPEC-130 UUID map is primary in-session** — parse is fallback for map miss, iw2, stamp, advisor | SOLID: sink owns identity write; mirror owns resolve |
| LAW-133-5 | **Fail closed on ambiguity** — zero both-resolve → naive rsplit then miss; multiple both-resolve → prefer rightmost among both-resolve (documented) or miss — never pick a split that does not resolve | Operator trust / LAW-098-4 |
| LAW-133-6 | **Do not blame spine ensure for this class** — `995/1000` with arrow-in-name samples ≠ missing `entities` rows | Ops honesty (SPEC-098 runbook) |
| LAW-133-7 | **Source-arrow regression barred** — `27_->_25_STRENGTHENING->CLAIM_FRONTIER` must still resolve | CHANGELOG contract |
| LAW-133-8 | **Unfakable proof** — unit keys from zz-raw + contract/e2e with real index | Honest acceptance |
| LAW-133-9 | **Optional escape encoding is a follow-up** — versioned/escaped keys may eliminate residual multi-both-resolve; not required to close this incident | Scope |

## Algorithm (LAW-133-3)

```ascii
  parse_with_resolver(id, exists):
    pair, rel ← rsplit id on last ':'
    candidates ← []
    for each index i where pair[i..] starts with "->":
      src ← pair[..i]; tgt ← pair[i+2..]
      if src≠∅ ∧ tgt≠∅ ∧ exists(src) ∧ exists(tgt):
        candidates.push((src,tgt,rel))
    match candidates.len():
      1 → return that candidate
      >1 → return rightmost candidate   # preserves source-arrow preference
      0 → return naive rsplit(id)       # then caller FK lookup may miss
```

## What this is not

| Not | Why |
|-----|-----|
| Missing PostgresEntitySink spine (`0/N`) | That is SPEC-098 Symptom A — different causal chain |
| AGE `graphid` operator gap | SPEC-106 |
| Concurrent `legacy_vector_id` race | SPEC-120 |
| Documents status CHECK | SPEC-129 |

## Cross-refs

- Why: [00-why.md](00-why.md)
- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Parents: [../098-data-access-hardening/](../098-data-access-hardening/), SPEC-130 sink map
