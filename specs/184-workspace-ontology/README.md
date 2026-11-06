# SPEC-184 — Workspace ontology

> **Product pin:** EdgeQuake v0.39.1 (fork branch `feat/workspace-ontology`).
> **Inherits:** [SPEC-114](../114-config-entity-type/) entity/relation allow-lists and typed edges ·
> [SPEC-160](../160-tev1/) decision-mode ontology · SPEC-0002 knowledge injection (distinct: content, not extraction rules).
> **Status:** implemented on the fork branch.

## Why

Industrial customers (aerospace, automotive) bring their own vocabulary: an ontology, a reference
list of parts and systems, house document types. Today a workspace can only list flat entity and
relation type names (SPEC-114). The model still invents names (`LEAP 1A`, `LEAP-1A`, `Leap1A` become
three entities) and cannot know what a company means by `SUBASSEMBLY` or that a `TURBOFAN` is an
`ENGINE`. The graph drifts from the company's reference, and answers lose precision.

## What a workspace ontology is

One optional JSON document per workspace, stored in workspace `metadata.ontology`:

| Part | Meaning | Effect at extraction |
|---|---|---|
| `entity_types[]` `{id, label, description, parent, synonyms[]}` | The company's entity kinds, with a definition, an optional parent kind and other names | Definitions and hierarchy go into the extraction prompt; a type written with a synonym is mapped to its id |
| `relation_types[]` `{id, description, source_types[], target_types[]}` | The company's relation kinds and which kinds they connect | Definitions go into the prompt; allowed pairs become SPEC-114b typed edges |
| `terms[]` `{name, entity_type, aliases[], description}` | Reference names (parts, programs, systems, sites…) | Terms found in a chunk are listed in that chunk's prompt; an extracted name equal to an alias becomes the canonical name with the term's type |
| `sources[]` | Where the ontology came from (file, document, copilot, manual) | Display only |

Saving an ontology also writes the SPEC-114 lists (`entity_types`, `relation_types`,
`relation_edges`) from it, so every existing mechanism (strict mode, colors, decision mode, UI)
sees the same vocabulary. The lists that were there before are kept in
`metadata.ontology_schema_before` and restored when the ontology is removed.

## Laws

- **LAW-184-1 No ontology, no change.** Without `metadata.ontology` every prompt, parse and graph
  write is byte-for-byte what it was (tested on the prompt text).
- **LAW-184-2 An ontology never breaks ingestion.** An unreadable stored ontology is ignored with a
  warning; extraction continues with the SPEC-114 lists.
- **LAW-184-3 One normalizer.** Term and alias keys use `normalize_entity_name`, the same function
  that makes graph node ids, so an alias matches exactly what the graph stores.
- **LAW-184-4 Ambiguity is an error.** One alias may point to one term only; a type synonym may not
  be another type's id; the type hierarchy has no cycle.
- **LAW-184-5 Nothing changes the existing graph silently.** Saving an ontology affects future
  extractions only. Existing data changes only through an explicit action: *realign* (merge alias
  entities into canonical ones, retype terms) or the existing *Rebuild knowledge graph*.
- **LAW-184-6 The copilot proposes, the user approves.** The copilot can draft an ontology, compare it
  with the graph and propose a save, merges, retypes or a rebuild; each goes through the existing
  approval card.
- **LAW-184-7 Same authority as workspace settings.** Every write route sits under `/workspaces/{id}`,
  so it needs owner/admin membership, as `PUT /workspaces/{id}` does.

## Surfaces

### API (`/api/v1/workspaces/{workspace_id}/ontology…`)

| Route | Does |
|---|---|
| `GET …/ontology` | Current ontology (or `null`) and counts |
| `PUT …/ontology` | Validate, save, sync SPEC-114 lists. Returns the saved ontology and warnings. 400 lists every error |
| `DELETE …/ontology` | Remove it and restore the previous lists |
| `POST …/ontology/draft` | Body `{text, file_name?}` → proposed ontology (not saved). JSON and CSV/TSV are parsed directly; other text is read by the workspace's LLM in segments |
| `POST …/ontology/plan` | Body `{ontology?}` (default: the saved one) → what realignment would change in the current graph, and how much of the graph the ontology covers |
| `POST …/ontology/realign` | Apply the plan of the saved ontology: merge alias entities into canonical ones (create the canonical entity first when missing), retype terms. Uses the existing entity merge/create/update code |

PDF, DOCX and other binary sources are uploaded as normal documents first (the existing
conversion pipeline handles them); the draft is then built from that document's text.

### Web UI

Workspace page → **Ontology** card: view (types with definitions and parents, relations, terms with
aliases), import (file: `.json`, `.csv`, `.tsv`, `.txt`, `.md`; or a workspace document), review
and edit the draft as JSON before saving, export JSON, remove, see the alignment plan and run
*Realign graph*; *Rebuild knowledge graph* stays where it is.

### Copilot

Read tools `get_ontology`, `draft_ontology_from_document`, `plan_ontology_alignment`; new approval
operations `set_ontology` and `rebuild_knowledge_graph` next to the existing merge/update ones.

## Limits

50 entity types and 50 relation types (decision mode's ceiling), 5 000 terms, 20 aliases per term,
20 synonyms per type, 100 typed edges (SPEC-114b). Draft from free text: first 120 000 characters,
in segments of 12 000.

## Out of scope (next)

- A/B answer evaluation (same questions on the workspace before and after) — needs a workspace copy.
- OWL/RDF/SKOS import and export.
- Versioned ontology history beyond the one restorable snapshot.
