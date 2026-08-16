# 10 — Edge cases

| # | Case | Mitigation | Test |
|---|------|------------|------|
| EC-1 | N=2 small PDFs happy path | Admit both | Playwright spec132 |
| EC-2 | N=4 > concurrency 3 | 4th waits then admits | queue-visibility / unit |
| EC-3 | Wake channel full | try_send / timeout; durable row | e2e_spec132 admit-non-block |
| EC-4 | Vision host down | Admit OK; convert fails honestly | Arm D reproduction + logs |
| EC-5 | One file >50 MiB | Reject that file; siblings proceed | client maxSize + API |
| EC-6 | `/upload/batch` with PDF | Explicit per-file error | SPEC-123 + docs |
| EC-7 | Shared client `batchTrackId` | Progress still per `task_id` | SPEC-054 |
| EC-8 | Duplicate PDFs in batch | Duplicate dialog | duplicate-upload e2e |
| EC-9 | Mixed PDF+MD selection | Each route correct | perform-file-upload tests |
| EC-10 | `/pdf/batch` body sum >50 MiB | Clear 413 | API / OpenAPI note |
| EC-11 | Worker crash mid-batch | Recover-stuck / hydrate | existing recovery |
| EC-12 | Abort after admit | Cancel task, not transfer | cancel paths |
| EC-13 | XHR admit timeout mid-batch | Row error; slot freed; others continue | WP-3 + unit |
| EC-14 | Second selection while first in flight | Shared executor queues; no discard | multi-document-queue-visibility |

## Out of edge scope

- Unbounded parallel vision for wall-clock (#361).
- Office/DOCX ingest (SPEC-121).

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
