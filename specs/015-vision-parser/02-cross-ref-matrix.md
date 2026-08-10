# 02 — Cross-Ref Matrix (SPEC-015V)

| Finding | Law | Code | Test |
|---------|-----|------|------|
| F-015V-1 | LAW-015V-4 | `edgequake-pdf/.../vision.rs` gates | `edgequake-pdf` lib + e2e |
| F-015V-2 | LAW-015V-1/2 | `workspace_ops` + metadata keys | workspace CRUD / e2e |
| F-015V-3 | LAW-015V-2 | `PdfUploadOptions` + multipart | upload types unit + e2e |
| F-015V-4 | LAW-015V-5 | `VisionPdfConverter` `.system_prompt` | vision_extract unit |
| F-015V-5 | LAW-015V-5 | `multimodal/prompts.rs` | prompts unit |
| F-015V-6 | LAW-015V-7 | `document-parsing-step.tsx` + `VisionExtractControls` | Playwright |
| F-015V-7 | LAW-015V-7 | `document-dropzone.tsx` | Playwright |
| F-015V-8 | LAW-015V-4 | `document_assets` + analyze gates | EC9 |
| F-015V-9 | LAW-015V-6 | `pdf_processing` metadata snapshot | e2e metadata |
| F-015V-10 | LAW-015V-4 | chart residual / promote_fig_as_chart | EC2 |

## Type SSOT

| Type | Crate | Path |
|------|-------|------|
| `VisionExtractConfig` | edgequake-pdf | `vision_extract.rs` |
| `PageDrawingAssetsConfig` | edgequake-pdf | `backend/mod.rs` |
| Workspace metadata keys | edgequake-core | helpers + workspace_ops |
| Upload options | edgequake-api | `pdf_upload/types.rs` |
| UI controls | edgequake_webui | `vision-extract-controls.tsx` |
