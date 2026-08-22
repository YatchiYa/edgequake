# Lens 008 — PDF Expert

## Stake

MFD manuscript PDFs are **image-primary containers**, not text documents with
optional images.

## Anatomy (class)

```ascii
  Page
   ├─ MediaBox A4 (sometimes rotated landscape)
   ├─ Large DCTDecode / Flate RGB Image XObject  ≈ full page scan
   ├─ Many small CCITTFaxDecode tiles             ≈ scanner OCR crumbs
   └─ Sparse ToUnicode / text operators           ≈ unreliable
```

## Implications

| Fact | Action |
|------|--------|
| Text extract length > 0 | Does **not** mean born-digital; classifier must use image area frac |
| Mixed page rotation | Render respects page rot; classifier soft prior |
| Adaptive DPI for huge files | Must not crush MS floor without operator opt-out |
| max_rendered_pixels | Must rise with DPI or effective resolution stalls |
| pdfium bundled | Keep; no PDFIUM_DYNAMIC_LIB_PATH regress |

## Render math

```ascii
  effective_long_edge = min( mediabox_pt * dpi/72 , max_rendered_pixels )
  For A4 long edge 842pt:
    150 DPI → ~1754 px
    300 DPI → ~3508 px  → needs max_pixels ≥ 3508 (floor 3600)
```

## Auto / EdgeParse

Text-density fast-path is unsafe when large Image XObjects dominate. Classifier
must veto (LAW-134-12).

## Cross-refs

- As-is: [../03-code-as-is.md](../03-code-as-is.md)
- SPEC-095 pdfium: [../../095-pdfium/](../../095-pdfium/)
- SPEC-038: [../../038-ingestion-large-pdf/](../../038-ingestion-large-pdf/)
- SOTA: [../12-sota-assessment.md](../12-sota-assessment.md)
