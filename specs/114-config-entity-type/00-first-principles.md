# 00 — First Principles (SPEC-114)

## Axioms

1. **A workspace owns a KG vocabulary** — entity types and relation types are peer concerns of the same schema.  
2. **Empty means free-form for relations** — absent/empty `relation_types` preserves today's LLM freedom (backward compatible).  
3. **Strict is a policy, not a storage quirk** — same normalize → match → remap pipeline for entities and relations.  
4. **Presets are honest packaging** — selecting a domain replaces both lists; identity may be stored for UX.  
5. **UI is a view of the schema** — dual panels + preview; canvas is progressive enhancement (v2).  
6. **PUT does not rewrite history** — metadata changes apply to future ingestions; rebuild is explicit.  
7. **Evidence beats vibes** — every finding maps to a gate.

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-114-1** | Vocabulary SSOT — workspace `metadata` owns `entity_types` and `relation_types` (+ strict flags, optional `kg_schema_preset`). |
| **LAW-114-2** | Symmetric policy — entity and relation share normalize / max-50 / dedupe / strict semantics via one helper. |
| **LAW-114-3** | Empty relations = free-form — no relation prompt section and no enforce when list empty/absent. |
| **LAW-114-4** | Preset honesty — FE General ≡ Rust `default_entity_types()`; domain presets ship entity+relation; `kg_schema_preset` optional. |
| **LAW-114-5** | One schema module — UI does not fork normalize/enforce; pipeline reads metadata via factory. |
| **LAW-114-6** | Future-only apply — PUT never silently rewrites AGE; review shows rebuild/reprocess hint when schema changes. |
| **LAW-114-7** | Hybrid UX — dual panels primary; `KgSchemaPreview` secondary; full canvas deferred to v2. |
| **LAW-114-8** | CI is proof — every F-114-* has unit, Playwright, or Rust e2e gate. |

## DRY / SOLID

| Principle | Application |
|-----------|-------------|
| **DRY** | One `normalize_type_list` (Rust + mirrored FE); shared chip/bulk primitives; one preset catalog for entity+relation. |
| **SRP** | Normalize ≠ persist ≠ prompt ≠ enforce ≠ preview UI. |
| **OCP** | New domain preset = extend catalog; consumers unchanged. |
| **LSP** | Memory + Postgres workspace services share same request/response shape. |
| **ISP** | Relation selector props optional; create flow may omit strict until reconfigure. |
| **DIP** | UI depends on draft/payload helpers; pipeline depends on `ExtractionSchema`, not HTTP DTOs. |

## Inheritance (do not break)

| Prior | Constraint |
|-------|------------|
| SPEC-085 / #216 | `entity_types` normalize + persist path unchanged |
| SPEC-013 | Workspace entity_types e2e remains green |
| SPEC-096 | Language remap for entity presets; custom types stay |
| SPEC-101 | Wizard shell / step composition preserved |
| SPEC-102 | `entity_type_colors` entity-only; resolver unchanged |
| SPEC-100 | Graph CLS reserved slots remain |

## v1 vs v2 first principles

| Concern | v1 | v2 wave A (114b) | v2 canvas (later) |
|---------|----|------------------|-------------------|
| Relation model | Allow-list strings | Allow-list + `relation_edges` | same SSOT |
| Preview | Decorative pairing (removed in 114b) | Honest typed-edge list + editor | React Flow expand |
| Enforce | Label ∈ allow-list | Label + endpoint types when edges non-empty | same |

## Laws (SPEC-114b typed edges)

| Law | Statement |
|-----|-----------|
| **LAW-114-9** | Topology SSOT — optional `relation_edges[]` owns allowed `(source, relation, target)` triples in workspace metadata. |
| **LAW-114-10** | Vocabulary stays peer — `relation_types[]` remains the label allow-list; edges may only use labels ∈ that list (auto-add on create). |
| **LAW-114-11** | Empty edges = unconstrained endpoints — when `relation_edges` absent/empty, behave as v1 (label allow-list / free-form only). |
| **LAW-114-12** | Honest preview — preview renders only real edges; never invent pairings. |
| **LAW-114-13** | Wizard-fit editor first — compact triple CRUD in-step; full canvas is progressive disclosure later (no React Flow in 114b). |
