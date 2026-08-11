# 06 — UX / UI Spec

## v1 scope

**No new UI surfaces.** Backend identity bridge only.

## Status honesty

| State | User-visible |
|-------|--------------|
| `processing` | Existing spinner / badge |
| `completed` | Existing success state; entity_count may be >0 |
| `failed` | Existing error field; must not show `invalid uuid 'injection::…'` after fix |

## Non-goals

- Wizard steps for dual identity
- Debug panels showing both IDs by default
- New empty states

## Optional follow-up (out of v1)

If injection handlers continue to ignore path `workspace_id` (tenant default remap), product UX should eventually accept path workspace explicitly — tracked separately from SPEC-118 identity work.
