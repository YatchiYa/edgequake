# Lens 001 — Product Owner

## Stake

Heading-dense markdown (meeting notes, CRM exports, wikis) is a common partner corpus. Exploding \(N\) burns extract budget and produces empty “heading” embeddings. A heading-only first chunk is a trust failure: the product looks broken even when size is “correct.”

## Outcomes (v1)

1. Markdown ingest packs small sections; heading-dense note → one chunk.
2. Operators can still pin Acc-fair Recursive/Fixed (unchanged).
3. Kill switch for rollback without a code revert.
4. Rebuild is explicit (future ingestions only).
5. Langfuse shows real chunk-token spread so support can diagnose without reading PII.

## Non-outcomes (v1)

Tenant chunking, LLM-written context prefixes, late chunking, auto-rebuild.

## Acceptance narrative

> As a partner, I upload a markdown outline with many short `###` sections. The document does not become one chunk per heading. Lineage shows packed chunks with headings still visible in the text. After I change packing, I rebuild to re-chunk old docs.

## Cross-refs

- Why: [../00-why.md](../00-why.md)
- Acceptance: [../09-acceptance.md](../09-acceptance.md)
- UX: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
