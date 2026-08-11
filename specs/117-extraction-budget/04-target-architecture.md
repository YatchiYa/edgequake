# 04 — Target Architecture

## Precedence (LAW-117-2)

```ascii
  document extract_max_*  ──┐
  workspace extract_max_* ──┼──► ExtractionCaps::resolve ──► prompts + hard truncate
  fleet env / 40/100      ──┘
```

## Metadata keys (workspace)

```json
{
  "extract_max_entities": 40,
  "extract_max_records": 100
}
```

Absent both → Inherit. Clear → remove keys.

## IngestionPipelineOptions

```rust
pub extraction_caps: Option<ExtractionCaps>, // None → from_env at resolve time
```

Factory/prepare set resolved caps **before** build_ingestion_pipeline.

## Pipeline improvements

```ascii
  1) Soft prompt: rank highest-value / relation-bearing first
  2) Hard truncate FIFO (safety net)
  3) If extract_caps_applied && gleaning_left:
        continue prompt with prior entity names
        → merge additional ents (own K budget)
```

## Validation

`entities >= 1`; `records >= entities`; else API 400.
