# 10 — Reproduction

## Environment

- Issue context: EdgeQuake **v0.24.2**, Docker, PostgreSQL ([#370](https://github.com/raphaelmansuy/edgequake/issues/370))
- Local verification target: **HEAD** / current workspace pin
- Database: `DATABASE_URL` required (no memory mode)

## Hypothesis under test

```ascii
  H1  JSON POST /documents succeeds
  H2  DOCX is rejected by FE Accept and/or API whitelist (by design)
  H3  PDF succeeds only on POST /documents/pdf (not /documents/upload)
  H4  PDF may fail after admit if pdfium/vision/proxy misconfigured
```

## Procedure

### A. Control — JSON text admit

```bash
# Requires running API + workspace UUID header as deployed
curl -sS -X POST "$API/api/v1/documents" \
  -H "Content-Type: application/json" \
  -H "X-Workspace-Id: $WORKSPACE_ID" \
  -d '{"title":"repro370.json","content":"{\"ok\":true}","source_type":"text"}'
```

**Expect:** 2xx + document id.

### B. DOCX — product reject

1. WebUI: drop `sample.docx` on documents dropzone.  
   **Expect:** toast unsupported; formats list without DOCX; **no** successful upload row.
2. API:

```bash
curl -sS -X POST "$API/api/v1/documents/upload" \
  -H "X-Workspace-Id: $WORKSPACE_ID" \
  -F "file=@sample.docx"
```

**Expect:** 400 `Unsupported file type: .docx`.

### C. PDF — wrong endpoint (trap)

```bash
curl -sS -X POST "$API/api/v1/documents/upload" \
  -H "X-Workspace-Id: $WORKSPACE_ID" \
  -F "file=@sample.pdf"
```

**Expect:** 400 unsupported `.pdf` (whitelist). Confirms docs that use this curl are wrong.

### D. PDF — correct endpoint

```bash
curl -sS -X POST "$API/api/v1/documents/pdf" \
  -H "X-Workspace-Id: $WORKSPACE_ID" \
  -F "file=@sample.pdf"
```

**Expect:** 2xx admit (`pdf_id` / `task_id`). Then poll status: Converting → Completed **or** Failed with convert error (not unsupported).

### E. Docker ops checks (if D fails after admit or on startup)

```bash
docker exec "$CTR" sh -c 'echo PDFIUM_AUTO_CACHE_DIR=$PDFIUM_AUTO_CACHE_DIR; ls -ld $PDFIUM_AUTO_CACHE_DIR'
curl -sS "$API/health" | jq .
# Confirm vision/Ollama reachable from container network
```

## Evidence log

| Date | Env | Step | Result | Notes |
|------|-----|------|--------|-------|
| 2026-08-11 | code review HEAD | H2 FE Accept | PASS (by design) | `use-document-dropzone.ts` has no docx |
| 2026-08-11 | code review HEAD | H2 BE whitelist | PASS | `ALLOWED_EXTENSIONS` has no docx/pdf |
| 2026-08-11 | code review HEAD | H3 routing | PASS | `perform-file-upload.ts` → `/documents/pdf` |
| 2026-08-11 | unit | `file_validation` lib tests | **19/19 PASS** | includes invalid extension + PDF magic mismatch |
| 2026-08-11 | unit | `test_validate_extension_invalid` | PASS | non-whitelist rejected |
| 2026-08-11 | matrix script | `.json/.md/.txt` allow; `.pdf/.docx/.xlsx` reject on text upload | PASS | mirrors `ALLOWED_EXTENSIONS` |
| 2026-08-11 | FE vitest | `file-kind.test.ts` | **6/6 PASS** | PDF vs image vs text classify |
| 2026-08-11 | e2e_file_upload | full binary e2e | FAIL env | 12 failed / 3 passed — local harness (auth/DB), not matrix disproof |
| 2026-08-11 | localhost:8080 | `/health` | 401 JWT | OrbStack-fronted port; live curl A–D deferred to reporter/docker with auth |

## Code-level reproduction (no server)

Confirmed on HEAD that #370’s DOCX half is **not a regression**:

- FE: `ACCEPTED_FILE_TYPES` excludes Office  
- BE: `validate_extension` rejects non-whitelist  
- Injection e2e already expects `.docx` → 400  

PDF half requires runtime (multipart + pdfium + vision). Until live curl evidence is pasted above, treat PDF failures as **environment-class** until proven otherwise.

## Cross-refs

- System lens: [05-lenses/007-system-engineer.md](05-lenses/007-system-engineer.md)
- Honest assessment: [11-honest-assessment.md](11-honest-assessment.md)
