# Lens 002 — Full Stack Developer

## Stake

Wire modality → profile → `VisionConversionConfig` without forking three copy-pasted
DPI parsers. Keep DRY with existing vision env resolve.

## Touchpoints

| Layer | Change |
|-------|--------|
| `edgequake-pdf` | `PageModality`, classifier, `ManuscriptProfile`, prompt select, asset policy hooks |
| `edgequake-api` `pdf_processing.rs` | Apply profile; skip EdgeParse FP; pass env floors |
| Parse / upload options | Optional modality override for tests |
| Multimodal Pass-B | Area/ink gate when modality manuscript |
| WebUI | Chip from API fields |
| Migrations | `document_pages` columns (WP-5) |

## SOLID map

```ascii
  pdf_processing
       │ depends on
       ▼
  edgequake_pdf::ManuscriptProfile::resolve(...)
       │ uses
       ├─ PageClassifier
       └─ vision_prompts::pass_a_system_prompt_for(modality)
```

Do **not** put manuscript prompt strings in the API crate.

## Env

Document in `.env.example` + AGENTS.md after implement (WP-6).

## Failure classes

Higher DPI → longer VLM calls. Reuse SPEC-057 stall/timeout; classify honestly;
do not fall back to EdgeParse on MS (LAW-134-12).

## Cross-refs

- As-is: [../03-code-as-is.md](../03-code-as-is.md)
- Plan: [../07-implementation-plan.md](../07-implementation-plan.md)
- SOTA: [../12-sota-assessment.md](../12-sota-assessment.md)
