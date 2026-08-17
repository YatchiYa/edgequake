# 05 — Edge cases (SPEC-100)

| ID | Scenario | Expected |
|----|----------|----------|
| EC-100-01 | Hard refresh Pipeline while busy | Chunk slot reserved before live card |
| EC-100-02 | Soft Refresh Documents with cache | No inventory Y jump (`placeholderData`) |
| EC-100-03 | Navigate `/documents` → `/documents/[id]` | Breadcrumb band height unchanged (spacer→bar) |
| EC-100-04 | Settings as non-admin | Admin slots stay collapsed/hidden without late tall mount |
| EC-100-05 | Idle Dashboard cold load | No false reservation of live-work chrome |
