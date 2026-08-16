# Lens 004 — UX / UI

## Stake

Operators and document owners need **truthful failure language** and **discoverable knobs**. P0 does not require a wizard redesign; it requires honest errors and documented env.

## Today (pain)

```ascii
  Documents list: Failed
  Detail: generic / failure_class=unknown
  Operator: retries → same 400 → trust break
```

## Target UX outcomes

| Surface | Change |
|---------|--------|
| Failed document detail | Show `failure_class=llm_unsupported_param` + recommended action copy |
| Recommended action copy | “Set EDGEQUAKE_LLM_OMIT_TEMPERATURE=true (or switch model / API format) and restart” |
| Setup / ops docs | Mantle examples for omit + Responses |
| Settings (optional P2) | Read-only “Effective LLM wire” card: format + omit flags |

## Copy (normative)

**Title:** LLM rejected request parameters  
**Body:** The model rejected a parameter (often `temperature`). EdgeQuake can omit it when `EDGEQUAKE_LLM_OMIT_TEMPERATURE=true`. For Responses-only models, set `EDGEQUAKE_LLM_API_FORMAT=responses`.  
**CTA primary:** Open setup docs  
**CTA secondary:** Retry after config change (do not spin auto-retry forever — permanent class)

## What not to build in P0

- Per-workspace toggle UI for omit/format (env is fleet-level in v1)
- Auto-toast “we detected Gemma, enable omit?” without proof
- Changing query chat composer temperature slider behavior beyond respecting omit when server-side

## Accessibility / clarity

- Prefer plain language over raw OpenAI error JSON in the primary line; keep raw error in expandable “Technical details.”

## Cross-refs

- UX spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
- Front designer: [005-front-designer.md](005-front-designer.md)
- PO: [001-product-owner.md](001-product-owner.md)
