# LENS — Full Stack (SPEC-015V)

```ascii
UI (VisionExtractControls)
  → PUT workspace metadata / multipart upload
  → VisionExtractConfig::resolve
  → PageDrawingAssetsConfig + Pass B overrides
  → convert + analyze + doc metadata snapshot
```

**Invariants:** One resolve type; OpenAPI DTOs mirror metadata keys; FE omit-unchanged on upload.
