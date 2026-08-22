# Fixtures (synthetic only)

## Rules

1. **Never** commit the operator trigger scan or any transcription of its content.
2. Use synthetic PDFs / PNGs with invented labels (`ITEM-A`, `12.3`, etc.).
3. Gold markdown files use the same invented strings for CER/WER.
4. Keep files small (<2 MB each) for CI.

## Planned inventory

| Path | Role |
|------|------|
| `print_simple.pdf` | Born-digital print control |
| `ms_image_primary.pdf` | Image-primary handwritten synthetic |
| `ms_implicit_table.gold.md` | Table F1 gold |
| `ms_hand_chart.gold.md` | Chart KV gold |
| `ms_noise_crop.png` | Pass-B suppression |

Placeholders may be empty until WP-8; rubric is normative now.

## Generation notes

- Prefer pdfium-renderable single-page PDFs with one embedded JPEG.
- Handwriting can be a raster of synthetic strokes (not real PII).
- Document generator script path when added: `fixtures/gen/` (optional).

## Cross-refs

- Test protocol: [../08-test-protocol.md](../08-test-protocol.md)
- LAW-134-10: [../01-first-principles.md](../01-first-principles.md)
- SOTA: [../12-sota-assessment.md](../12-sota-assessment.md)
