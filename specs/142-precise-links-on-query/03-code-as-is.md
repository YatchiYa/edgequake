# 03 — Code as-is

## Dual channel (broken product contract)

```ascii
  Retrieval ──► sources[] (document_id, page_start, title via KV)
                      │
                      ▼
              SourceCitations panel  ──click──► /documents/{id}?chunk=&page=
                                                    (mapper bug on UUID chunk ids)

  LLM answer text ──► "[N]" / invented "page 47" / markdown links
                      │
                      ▼
              StreamingMarkdownRenderer
              NO verify · NO rewrite · NO Next.js navigation
```

## Key paths

| Layer | Path | Behavior today |
|-------|------|----------------|
| Prompt | `edgequake-query/src/grounding.rs` | Mandate `[N]`; no forbid of page/name/URL |
| Headers | `edgequake-query/src/context_format.rs` | `page=P`; `doc=` UUID only if `EDGEQUAKE_CONTENT_HEADINGS` |
| Sources | `edgequake-api/.../source_reference_builder.rs` | `page_*` copied; `file_path: None` until resolve |
| Titles | `handlers/query/mod.rs` `resolve_chunk_file_paths` | KV title / file_name → `file_path` |
| Sync | `query_execute.rs` | Answer = raw LLM text |
| Stream | `query_stream.rs` | Context before tokens; no verified Done body |
| Chat | `handlers/chat/*` | Same sources; answer not rewritten |
| Mapper | `source-mapper.ts` | `document_id: extractDocumentId(s.id)` ignores `s.document_id` |
| Answer UI | `chat-message.tsx` | No `onCitationClick`; citations after stream |
| Links | `MarkdownInlineTokens.tsx` | `target="_blank"` for all links |
| Deeplink | `document-url.ts` | Correct `?page=N` schema |
| Viewer | `documents/[id]/page.tsx` | Controlled `currentPage` from `?page=` |

## Gaps that cause hallucination UX

1. Inline `[N]` not linked.
2. No post-gen catalog membership check.
3. Prompt does not forbid inventing pages/names in prose (and does not stop it).
4. Passage numbers in panel use `chunk_index`, not `reference_id`.
5. UUID-only chunk ids break `extractDocumentId`.

## Cross-refs

- Target: [04-target-architecture.md](04-target-architecture.md)
- WHY: [00-why.md](00-why.md)
