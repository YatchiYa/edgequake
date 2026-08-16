# 04 — Target architecture

## Principle

```ascii
                    RelVectors.id (legacy string)
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
         known map      index-guided     naive rsplit
         (SPEC-130)     parse (133)      (last resort)
              │               │               │
              └───────► relationships.id ◄────┘
                        (Plane A UUID)
```

## New SSOT

```rust
/// Prefer splits where both endpoints exist in the resolver.
pub fn parse_relationship_legacy_key_with_resolver<F>(
    id: &str,
    exists: F,
) -> Option<(String, String, String)>
where
    F: Fn(&str) -> bool;
```

- Lives next to `parse_relationship_legacy_key` in `embedding_family.rs`.
- `EntityNameIndex` gains a thin wrapper that passes `|n| self.resolve(n).is_some()`.
- Naive `parse_relationship_legacy_key` remains for classification / no-index callers.

## Call sites to wire (fallback path)

| Caller | Change |
|--------|--------|
| `fleet_embedding_index::mirror_legacy_batch` | On known-map miss, parse via index resolver |
| `fleet_embedding_backfill` / stamp / coverage scan | Parse via index when resolving legacy ids |
| Unit tests | zz-raw five keys + source-arrow + multi-both-resolve rightmost |

## What does not change

- Key **format** string shape (no forced migration of stored `legacy_vector_id`)
- Fail-closed when still unresolved
- SPEC-130 map construction / RelGraph-before-RelVectors order
- Entity name extraction vocabulary (AI may still emit `->`)

## SOLID / DRY

| Principle | Application |
|-----------|-------------|
| SRP | `embedding_family` owns composite key grammar; index owns existence; sink owns UUID write |
| OCP | New disambiguation strategies extend resolver helper without rewriting callers |
| DRY | One formatter; one naive parse; one resolver parse — no per-crate copies |
| LSP | Resolver parse returns same `Option<(src,tgt,rel)>` shape as naive parse |
| ISP / DIP | Mirror depends on `Fn(&str)->bool`, not SQL |

## Optional follow-up (LAW-133-9)

```ascii
  v2 key:  "rel:" + len-prefixed(src) + len-prefixed(tgt) + rel
           or escape "->" / ":" inside names
  dual-read: accept v1 + v2 during backfill window
```

Not required to close the manuscript PDF incident.

## Cross-refs

- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Laws: [01-first-principles.md](01-first-principles.md)
