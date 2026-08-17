# LENS — Front End (SPEC-096)

> **Laws**: L1, L3, L5 · **Findings**: F-352-12 · **UI**: [LENS-ui.md](LENS-ui.md)

## Type changes

```ts
// Workspace type (edgequake_webui/src/types or generated)
extraction_language?: string | null;
```

Create/Update payloads:

```ts
extraction_language?: string | null; // omit | value | "" / "none" to clear
```

## Constants

```ts
// e.g. edgequake_webui/src/constants/extraction-languages.ts
export const EXTRACTION_LANGUAGES = [
  "English",
  "Chinese",
  "Japanese",
  "Korean",
  "Spanish",
  "French",
  "German",
  "Portuguese",
  "Italian",
  "Russian",
] as const;
```

Comment: keep in sync with `edgequake_pipeline::prompts::SUPPORTED_LANGUAGES` (OCP / EC-22).

## API client

Update `updateWorkspace` / `createWorkspace` in `lib/api/edgequake` to pass `extraction_language`. No new endpoint.

## State wiring

| Page | Work |
|------|------|
| `/w/[slug]/workspace` | Local state `selectedExtractionLanguage`; seed from `workspace.extraction_language` when entering edit; include in `updateMutation` |
| Dashboard workspace detail (if parallel) | Same card component |
| Create workspace | Optional select; omit when server default |

React Query: invalidate `['workspace', tenantId, workspaceId]` on success (existing pattern).

## Component API

```tsx
export interface WorkspaceExtractionLanguageCardProps {
  isEditing: boolean;
  workspace: Workspace;
  selectedLanguage: string | null; // null = server default
  onLanguageChange: (language: string | null) => void;
}
```

## Toast on change

If `previous !== next` after success, show `workspace.extractionLanguage.changedToast` (LAW-L5). Do not force rebuild modal; soft info toast is enough (entity-types pattern).

When language change remaps a preset-backed type list, also toast that entity types were updated to match extraction language (future ingestions only — LAW-L5/L6).

## Entity type catalog (LAW-L6)

```ts
// edgequake_webui/src/constants/entity-type-catalog.ts
localizeType(token, language)
localizeTypes(types, language)
remapPresetTypes(types, fromLang, toLang) // null if not a known preset
detectCanonicalPreset(types) // PresetKey | 'custom'
getPresetTypes(key, language) // from entity-presets
```

Wire `onLanguageChange` in workspace pages to call `remapPresetTypes` and update `selectedEntityTypes` when non-null. Pass `extractionLanguage` into `EntityTypeSelector`.

## Tests

| Kind | File / ID |
|------|-----------|
| Component unit (optional) | render view/edit states |
| Catalog unit | `spec096_entity_type_catalog_*` |
| Playwright | `spec096_ui_workspace_language_select`, `spec096_ui_create_workspace_language`, `spec096_ui_future_only_hint`, `spec096_ui_entity_types_follow_language` |

## i18n

Add keys from [LENS-ux.md](LENS-ux.md); English fallbacks in `t()` second arg for consistency with entity types card.

## Non-goals for FE

- Calling detect-language APIs.
- Per-document language UI.
- Editing env vars from the browser.
