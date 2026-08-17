# LENS — UI (SPEC-096)

> **Laws**: L1, L5 · **UX**: [LENS-ux.md](LENS-ux.md) · **Pattern**: [`workspace-entity-types-card.tsx`](../../../edgequake_webui/src/components/workspace/workspace-entity-types-card.tsx)

## Component

**`WorkspaceExtractionLanguageCard`**

Reuse the Card / CardHeader / CardTitle / CardDescription / CardContent pattern from Entity Types. Icon: `Languages` (lucide) — avoid inventing new visual language.

## States

| Mode | Content |
|------|---------|
| View + configured | Show language display name as text or Badge (`data-testid="ws-extraction-language-value"`) |
| View + unset | Muted text: “Server default” |
| Edit | Select of allowlisted languages + leading option “Server default” |
| Saving | Disable select / inherit parent edit busy state |
| Error | Inline alert under select if API returns 400 |

## Layout

```
┌─ Workspace page ─────────────────────────────┐
│  Header [Edit]                               │
│  Stats …                                     │
│  Model config grid …                         │
│  ┌ Entity Types Card ─────────────────────┐  │
│  └────────────────────────────────────────┘  │
│  ┌ Extraction Language Card ──────────────┐  │  ← NEW, immediately below
│  │  [Languages icon] Extraction Language  │  │
│  │  future-only hint                      │  │
│  │  view: Chinese | edit: <Select>        │  │
│  └────────────────────────────────────────┘  │
│  Actions / status footer …                   │
└──────────────────────────────────────────────┘
```

Keep existing visual density; no new page section chrome beyond one card.

## Options (v1 allowlist)

Mirror `SUPPORTED_LANGUAGES`:

English, Chinese, Japanese, Korean, Spanish, French, German, Portuguese, Italian, Russian

Plus synthetic UI value `__server_default__` → clear on save.

## Testids (normative)

| Element | `data-testid` |
|---------|----------------|
| Card root | `workspace-extraction-language-card` |
| View value | `ws-extraction-language-value` |
| Edit select | `ws-extraction-language-select` |
| Future-only hint | `extraction-language-future-only-hint` |
| Create form select | `create-workspace-extraction-language` |

## Visual rules (align with existing workspace UI)

- Follow existing Card spacing and typography tokens.
- Do **not** introduce purple-gradient marketing blocks or new dashboard widgets.
- Select width: full width of card content on mobile; reasonable max on desktop (`max-w-sm` acceptable).
- Match Entity Types strict-status muted helper text style for the future-only hint.

## Create workspace

Add the same select to the create form near entity types / advanced section. Default selection = Server default (omit field on POST).

## Interaction with Edit mode

Language is part of the same Edit/Save transaction as models and entity types — do not invent a separate Save on the language card alone (DRY with current workspace page).

## Layout order

Render **Extraction Language** card **above** Entity Types (workspace pages and create dialogs). Language is chosen first; type presets remapped afterward (LAW-L6).

## LAW-L6 — Entity types follow language

- When the operator changes Extraction Language and current chips match a known preset (any language variant), remapped chips appear immediately (e.g. French General → `PERSONNE`, `ORGANISATION`, …).
- Custom/mixed lists stay as-is; show a short muted hint that custom types are not auto-translated.
- `EntityTypeSelector` placeholder / preset buttons use language-aware tokens via `getPresetTypes(key, language)`.
- Persist remapped `entity_types` on the same Save as `extraction_language`.
