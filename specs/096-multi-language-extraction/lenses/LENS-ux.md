# LENS — UX (SPEC-096)

> **Laws**: L1, L5 · **Findings**: F-352-12, F-352-15 · **UI lens**: [LENS-ui.md](LENS-ui.md)

## Job to be done

When configuring a workspace for a non-English corpus, the operator needs a **single, obvious control** that answers: “In what language should the knowledge graph be written?”

Today the only discoverable lever is Entity Types (localized labels). That **implies** language but does not **promise** it — causing false confidence (F-352-15).

## UX principles

1. **Colocate with Entity Types** — Language is extraction ontology configuration, not LLM vendor settings. Place the card adjacent to `WorkspaceEntityTypesCard`.  
2. **One job per control** — A single select of allowlisted languages; no free-text in v1 (prevents EC-17 injection / typos).  
3. **Honest future-only semantics** — Same pattern as entity types: “Applies to future ingestions. Reprocess / Rebuild Knowledge Graph to refresh existing documents.” (LAW-L5)  
4. **Default transparency** — When unset, show “Using server default (English)” or resolved env label if we expose effective language. Prefer showing **configured** vs **effective**:
   - View: configured value OR “Server default”.
   - Optional subtitle: effective = `Chinese` when env set (nice-to-have; not required for v1 if costly).  
5. **Change consequences** — On save when language changes, toast analogous to LLM change: existing graph keeps old language until rebuild.  
6. **No hero clutter** — Do not add banners, stat chips, or marketing callouts on the workspace page. One card, one select.

## User flows

### Flow A — Configure existing workspace

1. Open `/w/{slug}/workspace` (or dashboard workspace detail).  
2. Click Edit.  
3. Set Extraction Language → Chinese.  
4. Save.  
5. See toast success + reprocess hint.  
6. Reload: card shows Chinese.

### Flow B — Create workspace

1. Create workspace dialog/form.  
2. Optional language select (default English / server default).  
3. Continue with models + entity types.

### Flow C — Clear override

1. Edit → choose “Server default” (maps to clear metadata).  
2. Save → inherits env/English.

## Copy (source strings)

| Key | English default |
|-----|-----------------|
| `workspace.extractionLanguage.title` | Extraction Language |
| `workspace.extractionLanguage.description` | Language used for entity names, descriptions, and relationship text during extraction. |
| `workspace.extractionLanguage.futureOnlyHint` | Applies to future document ingestions. Use Rebuild Knowledge Graph to re-extract existing documents. |
| `workspace.extractionLanguage.serverDefault` | Server default |
| `workspace.extractionLanguage.changedToast` | Extraction language updated. Reprocess documents to refresh the graph. |

## Accessibility

- Native `<select>` or Radix Select with label association.
- Keyboard operable; announce selected value.
- Error state on unsupported (should not happen with select, but API errors must surface).

## Anti-patterns to avoid

- Hiding language under “Advanced” only.
- Auto-detect toggle in v1.
- Dual controls (env UI + workspace) that fight — env is ops-only; UI edits workspace.

## Laws

L1 (explicit), L5 (future-only messaging).
