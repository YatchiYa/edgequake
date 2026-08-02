# 05 — Edge cases (SPEC-102)

| ID | Scenario | Expected | Laws | Test |
|----|----------|----------|------|------|
| EC-102-01 | Invalid hex `#gg0000` | API 400; UI blocks apply | LAW-102-3 | unit + Rust |
| EC-102-02 | Empty `{}` colors on PUT | Remove metadata key | LAW-102-3 | Rust |
| EC-102-03 | Key `person` / `Person` | Normalize to `PERSON` | LAW-102-3 | unit + Rust |
| EC-102-04 | >50 color entries | Cap/reject beyond 50 | LAW-102-3 | Rust unit |
| EC-102-05 | Unknown type, no override | `DEFAULT` `#94a3b8` | LAW-102-1 | unit |
| EC-102-06 | Community color mode | Node fill from community palette | LAW-102-4 | Playwright |
| EC-102-07 | Reset to default | Key removed; UI shows default hex | LAW-102-3 | Playwright |
| EC-102-08 | Shorthand `#0f0` | Accept; store canonical `#00ff00` preferred | LAW-102-3 | unit |
| EC-102-09 | Color equals default | May strip from metadata (`stripDefaultOverrides`) | LAW-102-3 | unit |
| EC-102-10 | PUT without permission | Existing authz (403/401) unchanged | LAW-102-2 | inherit auth |
| EC-102-11 | Dark theme | Hex fills unchanged; swatch visible | LAW-102-7 | visual/legend |
| EC-102-12 | Concurrent edit last-write-wins | Same as other workspace metadata | LAW-102-2 | inherit |
| EC-102-13 | Type removed from entity_types | Orphan color key tolerated (harmless) | LAW-102-2 | unit |
| EC-102-14 | Omit field on update | Leave colors unchanged | LAW-102-2 | Rust |

## ASCII vignette — resolve order

```ascii
type="machine", overrides={MACHINE:"#aabbcc"}
  → normalize MACHINE
  → override hit → #aabbcc

type="CREATURE", overrides={}
  → default ENTITY_TYPE_COLORS.CREATURE

type="ZZZ_UNKNOWN", overrides={}
  → DEFAULT #94a3b8
```
