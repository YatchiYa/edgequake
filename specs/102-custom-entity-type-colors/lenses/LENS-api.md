# LENS — API (SPEC-102)

## Question

Do colors follow the same metadata contract as `entity_types`?

## Verdict

Yes: optional request field → normalize/validate → metadata JSONB → top-level response field. Omit = leave unchanged; `{}` = clear.

## Anti-patterns

- New SQL column/migration for a map  
- Storing colors on AGE nodes  
- Accepting `rgb()` / named CSS colors  

## Laws cited

LAW-102-2, LAW-102-3  
