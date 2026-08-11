# 08 — Test Protocol

## Gates

| Gate | Command / action | Pass criteria |
|------|------------------|---------------|
| G0 | Spec pack review | Matrix + laws + EC table present |
| G1 | `cargo test -p edgequake-api --test e2e_file_upload` | TXT/MD/JSON admit |
| G2 | PDF e2e (`e2e_spec013_*` / pdf suite) | Valid PDF admit; bad magic rejected |
| G3 | pdfium (`e2e_spec095_pdfium` if env) | Prime path documented |
| G4 | FE unit `file-kind` + dropzone rejects | Office → invalid |
| G5 | Playwright image + PDF progress smoke | Paths green on local stack |
| G6 | Manual Docker repro ([10-reproduction.md](10-reproduction.md)) | Evidence logged |

## Test IDs (normative)

| ID | Case | Layer | Proof |
|----|------|-------|-------|
| T1 | TXT/MD/JSON admit | API e2e | `e2e_file_upload.rs` |
| T2 | Image multipart | Playwright / API | `e2e/image-upload.spec.ts` |
| T3 | PDF via `/documents/pdf` | API e2e | `e2e_spec013_*` |
| T4 | PDF via `/documents/upload` → 400 clear | API | new or extend upload e2e |
| T5 | DOCX dropzone reject | FE | unit + optional Playwright |
| T6 | DOCX API reject | API | upload + injection e2e |
| T7 | XLSX reject | FE + API | same pattern as T5/T6 |
| T8 | Oversize | FE + API | size validators / 413 |
| T9 | Bad PDF magic | API | existing PDF invalid tests |
| T10 | pdfium prime fail-closed | API/ops | `e2e_spec095_pdfium` |
| T11 | Vision down → Failed convert | e2e/mock | status + error code |
| T12 | Workspace missing on PDF | API | 400 Workspace ID required |

## Edge-case coverage map

Every EC-01..EC-20 in [07-implementation-plan.md](07-implementation-plan.md) must map to a T-id or an explicit “ops runbook only” note (EC-09).

## Negative proof (Office)

```ascii
  given product matrix excludes Office
  when user selects report.docx
  then no network call to /documents*
  and toast lists supported formats
```

## Positive proof (PDF)

```ascii
  given writable pdfium cache + workspace header
  when multipart POST /documents/pdf with %PDF- bytes
  then admit succeeds (pdf_id / task_id)
  and convert failure (if forced) ≠ UNSUPPORTED_FORMAT
```

## Cross-refs

- Implementation: [07-implementation-plan.md](07-implementation-plan.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
