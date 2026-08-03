# LENS — Progressive Disclosure (SPEC-099)

## Question

What must be visible immediately vs disclosed on demand?

References: [NN/g Progressive Disclosure](https://www.nngroup.com/articles/progressive-disclosure/) · SaaS density practice (show 3–5 essentials; defer the rest).

## Split (locked)

| Always primary | Secondary (request / context) |
|----------------|-------------------------------|
| Search | PDF parser override (idle detail; icon in collapse) |
| Status filter | Column toggles (Cost, Created vs Updated) |
| Inventory rows | Full Admit→Materialize stepper (zone only when live) |
| Compact StatusCell | Fence explanation tooltip / Indexed long label |
| Upload affordance | Full dropzone copy + format list (idle expanded) |
| Refresh | Clear All (overflow + typed confirm) |
| Selection actions when selected | Batch extras in overflow |

## Staged vs progressive

- **Progressive:** Cost column, Clear All, parser override, fence long copy.  
- **Staged (linear):** Upload → Admit → Prepare → Extract → Materialize lives in the **feedback zone** only — not restated as a table stepper.

## Observed failure (evidence 02)

```ascii
Primary overload:
  header chips + search + quiet dropzone
  + tall run cards (full stepper × N)
  + table badges
  + toast
= more than 7 competing chunks → disclosure failure
```

## Laws

LAW-099-2 · LAW-099-3 · LAW-099-4 · LAW-099-5 · LAW-099-6

## DoD for this lens

- Idle: expanded upload OK; no Active runs chrome.  
- Busy: collapse upload; one narrative owner; secondary columns optional.  
- No third toast SSOT for the same session.
