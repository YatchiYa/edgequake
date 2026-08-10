# 03 — Implementation Roadmap (SPEC-015V)

```ascii
W0 Spec pack + cross-ref
W1 VisionExtractConfig + PageDrawingAssetsConfig flags
W2 API/workspace metadata + upload multipart + OpenAPI
W3 Pipeline gates + Pass A/B prompts + doc snapshot
W4 WebUI shared controls (wizard + upload)
W5 E2E Rust + Playwright + EC matrix
W6 Brutal review / optional Acc smoke note
```

## DoD

- [ ] All docs in pack present and cross-linked  
- [ ] Defaults preserve pre-015V Vision behavior (all ON)  
- [ ] Upload overrides workspace per field  
- [ ] Flags OFF skip corresponding assets/markdown/Pass B  
- [ ] Prompt override replaces SSOT at builder boundary  
- [ ] Playwright + Rust e2e green for happy path + key ECs  

## Verification commands

```bash
cargo test -p edgequake-pdf --lib vision_extract
cargo test -p edgequake-api --lib document_assets
cargo test -p edgequake-api --test e2e_spec015v_vision_extract
cargo test -p edgequake-api --test spec027_api_contract
cd edgequake_webui && bun test
pnpm exec playwright test e2e/spec015v-vision-extract.spec.ts
```
