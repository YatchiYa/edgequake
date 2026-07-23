# `<ID>` — `<short title>`

> **Priority**: P0 | P1 | P2 | P3  
> **Audit status**: CONFIRMED | PARTIAL | FIXED | RETRACTED  
> **Cluster**: [`NN-name`](../clusters/NN-name/)  
> **Sprint**: 0–5  
> **Laws**: LAW-N  
> **Cross-refs**: related IDs

---

## 1. WHY

What breaks for users, tenants, data integrity, or cost if this stays open.

---

## 2. Audit (code is law)

| Field | Value |
|-------|-------|
| Primary locus | `path:lines` |
| Verdict | CONFIRMED / … |
| Verified against | HEAD (v0.20.2 lineage) |

```
# essential snippet or citation
```

---

## 3. Root cause (first principles)

Causal chain from invariant violation → symptom. Not a symptom list.

---

## 4. ASCII causal diagram

```
  cause --> mechanism --> symptom
```

---

## 5. Solution (SOLID + DRY)

| Principle | Application                          |
| -----------| --------------------------------------|
| S         | single responsibility owning the fix |
| O         | extension point if needed            |
| L         | subtype/contract preserved           |
| I         | narrow interface                     |
| D         | depend on abstraction / SSOT         |
| DRY       | shared primitive name                |

Concrete steps, files, and migrations.

---

## 6. Edge cases

| ID | Case | Mitigation |
|----|------|------------|
| EC-1 | … | … |

---

## 7. E2E / contract tests

| Test | Assertion |
|------|-----------|
| `test_…` | … |

---

## 8. Cross-refs

- Cluster summary, roadmap sprint, related defects, laws.
