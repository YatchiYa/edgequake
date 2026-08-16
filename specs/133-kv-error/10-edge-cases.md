# 10 — Edge cases

| # | Case | Mitigation | Test |
|---|------|------------|------|
| EC-1 | Target contains one `->` | Index-guided unique both-resolve | zz-raw FLOW_* keys |
| EC-2 | Target contains multiple `->` | Same; unique both-resolve | zz-raw LEFT_MARGIN* keys |
| EC-3 | Source contains `->`, target clean | Rightmost / unique both-resolve (v0.24.2) | existing source-arrow unit |
| EC-4 | Source and target both contain `->` | Unique both-resolve if only one pair in index; else rightmost both-resolve | unit synthetic |
| EC-5 | Multiple both-resolve candidates (format collision) | Rightmost among both-resolve (documented); optional later fail-closed | unit |
| EC-6 | No entity index / empty exists | Naive rsplit fallback → miss if wrong | unit |
| EC-7 | SPEC-130 map hit with colliding name | Map wins; no parse needed for FK | e2e_spec130 |
| EC-8 | SPEC-130 map miss, spine present | Index parse recovers | contract_spec133 |
| EC-9 | SPEC-130 map empty (`ids.is_empty()` → None) | All rows use index parse | contract + merge path |
| EC-10 | True missing spine (`0/N`) | Still fail-closed; not “fixed” by parse | existing SPEC-098 |
| EC-11 | Rel type case drift | `normalize_relation_type_str` before resolve | SPEC-098 |
| EC-12 | `:` inside entity name | `rsplit_once(':')` for rel type — residual ambiguity; document; escape follow-up | unit note |
| EC-13 | Self-loop / empty bare name | Collect/merge already skip | relationship.rs |
| EC-14 | Concurrent mirror same legacy id | SPEC-120 absorb | existing |
| EC-15 | iw2 backfill of historical target-arrow keys | Wire same resolver parse | WP-3 + contract |
| EC-16 | Report / entity families | Unchanged (`entity:` / `community_report:`) | classify unit |
| EC-17 | Operator re-runs 139 for this class | Ops doc: do not; reprocess after SPEC-133 | ops doc |
| EC-18 | UI truncation of miss list | Keep ≥3 samples visible | UX spec |

## Out of edge scope

- Full escaped/versioned legacy key migration (LAW-133-9).
- Forbidding `->` in LLM entity names by default (AI lens optional).

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
