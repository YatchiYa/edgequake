# Upgrade to EdgeQuake v0.26.0

> **From:** v0.25.0 · **To:** v0.26.0 · **CD:** GHCR (`edgequake`, `edgequake-frontend`, `edgequake-postgres`)

Minor train: PDF pack-to-budget (SPEC-135), manuscript page-as-unit convert (SPEC-134),
Langfuse dev sibling (SPEC-124), and partner reliability (#377, #383–#386, SPEC-101).
**New migration: 149.** LD-15 still applies — the API never auto-migrates at boot.

**crates.io deps:** pin `edgequake-llm` **0.10.8**, `edgequake-pdf2md` **0.9.11**, `edgeparse-core` **0.2.5**; `edgequake-sdk` **0.4.0** (no path patches).

Prior: [upgrade-to-0.25.0.md](upgrade-to-0.25.0.md) (SPEC-124…133 + mig 148).

**SPEC-001 Acc:** this cut **attests** existing [`publish/latest`](../../specs/001-benchmark/e2e/artifacts/publish/latest/)
(`valid: true`, medical-mid, `2026-08-15T11:02:18Z`) — no fresh n=200 Acc run; **PDF geometry not re-scored**.

## Highlights

| Area | What changed |
|------|----------------|
| Mig **149** | `tasks.document_id` column + index + backfill (#384) |
| SPEC-135 | PDF pack-to-budget (default ON); page span `page_start`/`page_end`; MM index-once; citation `p.N–M` |
| SPEC-134 | Manuscript page-as-unit convert; lift extract off disabled reasoning |
| SPEC-124 | `make dev-langfuse` / `dev-bg-langfuse` + `make spec124-langfuse-e2e` |
| #377 / SPEC-136 | Absorb stamp-once `legacy_vector_id` unique violations |
| #383–#386 | Saga compensation, in-flight task honesty, reprocess metadata rollback |
| SPEC-101 | Wizard persist honesty (create/reconfigure embedding overrides) |

## Sequence

```text
1. Backup (recommended — schema train 149)
2. Deploy v0.26.0 images (or binary) but do not start API replicas yet if schema is behind
3. Run migrate against the target DB (LD-15):

   edgequake migrate dry-run
   edgequake migrate

4. Start API + frontend pinned to 0.26.0
5. Verify health version + OpenAPI info.version + PDF ingest smoke
```

Compose / quickstart pin:

```bash
EDGEQUAKE_VERSION=0.26.0 docker compose -f docker-compose.quickstart.yml up -d
```

## Operator notes

### PDF chunking (SPEC-135)

Product PDF ingest now **packs** converted markdown to the workspace tiktoken budget (default ON).

```bash
# Roll back to pre-135 Recursive inner (ops kill switch)
EDGEQUAKE_PDF_PACK=0

# Disable cross-page span packing (hard page emit only)
EDGEQUAKE_PDF_CROSS_PAGE_PACK=0
```

- Applies to **future ingestions only** — no auto-rebuild of existing workspaces.
- Historical `chunks.page_start`/`page_end` stay NULL until explicit **Rebuild KG**.
- Acc PDF geometry was **not** re-scored for this cut.

### Langfuse (SPEC-124)

Unchanged from 0.25.0 for production. Local dev:

```bash
make dev-langfuse          # isolated Langfuse v4 + EdgeQuake stack
make spec124-langfuse-e2e  # one-command Settings + sessions proof
```

### Manuscript PDF (SPEC-134)

Manuscript-class pages use page-as-unit convert. See [specs/134-manuscrit/](../../specs/134-manuscrit/).

## Verify

```bash
curl -s http://localhost:8080/health | jq -r '.version'   # expect 0.26.0
curl -s http://localhost:8080/api-docs/openapi.json | jq -r '.info.version'  # 0.26.0
```

## Out of scope

- crates.io publish of EdgeQuake **workspace** crates (GHCR-only CD)
- Auto-rebuild KG on upgrade
- Fresh Acc n=200 medical-mid run
