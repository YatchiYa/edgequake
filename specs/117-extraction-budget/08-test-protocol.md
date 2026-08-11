# 08 — Test Protocol

## Unit

- `ExtractionCaps::resolve` precedence matrix  
- `validate` rejects bad pairs  
- Ranked wording in `prompt_quantity_limits_section`  
- Truncate metadata before/after  

## API contract

- Create/update/get workspace extract fields  
- 400 on invalid pairs  
- Document upload override round-trip into pipeline options (or metadata)  

## Pipeline e2e

- Mock LLM returns >K ents → hard truncate  
- With gleaning ≥1, second call includes continue / prior names  

## Playwright

- Card visible  
- LightRAG preset sets 40/100  
- Future-only hint  
- Wizard save round-trip (smoke)
