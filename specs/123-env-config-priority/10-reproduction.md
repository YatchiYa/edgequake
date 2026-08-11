# 10 — Reproduction

## Pre-fix (bug)

1. Workspace PDF parser = **Server Default** (UI: Resolves to Vision). Confirm API `pdf_parser_backend` is null/`none`.
2. Ensure `EDGEQUAKE_AUTO_PDF_ROUTING` default (on); `EDGEQUAKE_PDF_PARSER_BACKEND` unset or `vision`.
3. Documents page: Parser for this upload = **Workspace Default (Vision)**.
4. Batch-upload ≥1 born-digital PDF (e.g. arXiv Argus) + optional text files.
5. Open document detail → Extraction shows **EdgeParse**.

## Control (today already OK)

Set workspace explicitly to **Vision** → same PDF → lineage Vision (`explicit=true`).

## Post-fix (required)

Steps 1–4 → lineage **Vision**.

Optional: set workspace to **Auto** → born-digital may EdgeParse with auto note.

## Mixed batch admission

1. Drop one large PDF (≥ threshold) + one small PDF with Workspace Default (Vision).
2. Confirm EdgeParse in admission dialog.
3. **Expect:** large → EdgeParse; small → Vision.
