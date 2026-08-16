# Lens 005 — Front Designer

## Display vs column

```ascii
  Chip / stage label     ←  KV current_stage / unified mapper
                            (may show "Re-embedding" / re_embedding)

  List status pill        ←  often documents.status column
                            (must be CHECK-safe: processing, …)
```

No WebUI code change required for the #381 fix if the list already prefers relational status for the coarse pill and KV/WS for stage detail. If any FE assumed column == `re_embedding`, treat that as a bug — column will be `processing`.

## FE SSOT reminder

`edgequake_webui/src/lib/documents/status-domain.ts` may list `re_embedding` for display. That does **not** mean SQL stores it.

## Visual non-goals

- No new purple gradients, no new dashboard chrome.
- Preserve existing Documents table status styling; only data freshness improves.

## Cross-refs

- UX: [004-ux-ui.md](004-ux-ui.md)
- Spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
