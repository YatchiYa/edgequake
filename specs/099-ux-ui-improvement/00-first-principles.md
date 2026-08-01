# 00 — First Principles (SPEC-099)

## Axioms

1. **Inventory is the primary job** of `/documents` when idle; upload and live work are secondary chrome.  
2. **Narrative appears once** — live stage story has a single owner (feedback zone).  
3. **Status meaning is domain, paint is presentation** — normalize/rank/display resolution must not live in React components.  
4. **Serving fence is a queryability signal**, not a second “success” outcome.  
5. **Destructive actions require friction and distance** from benign peers (NN/g error prevention).  
6. **Honesty beats completeness theater** — never imply a full corpus when the fetch is capped.  
7. **Evidence beats vibes** — every finding maps to a unit or Playwright gate (LAW-098-6 inheritance).  
8. **Do not weaken delete honesty** — LAW-098-10 / LAW-098-11 remain binding.

## Laws

| Law | Statement |
|-----|-----------|
| **LAW-099-1** | One status domain — `normalizeStatus` / `getDocumentDisplayStatus` / terminal / processing / stage rank live only in `lib/documents/status-domain.ts`. Presentation modules import domain; they do not reimplement it. |
| **LAW-099-2** | Narrative vs inventory — live stage story (stepper, % bars, cancel) appears in the feedback zone; table shows compact status without a duplicate stepper; live rows hide stage subtitles (LAW-IS3 harden). |
| **LAW-099-3** | Serving fence is secondary — never a peer success pill; StatusCell composes pipeline status ⊕ fence (e.g. `Completed · Ready` or tooltip for Indexed-not-queryable); `query_ready` semantics unchanged (SPEC-091). |
| **LAW-099-4** | Viewport budget — idle: inventory table ≥60% of main content height; busy: feedback zone ≤35vh **and** upload slot collapsed. |
| **LAW-099-5** | Destructive friction — Clear All is not a peer of Refresh; overflow or danger zone + typed confirm retained ([NN/g proximity](https://www.nngroup.com/articles/proximity-consequential-options/)). |
| **LAW-099-6** | One upload narrative — toast XOR feedback-zone upload list for the same session (`use-file-upload` demotes toast when zone owns ids). |
| **LAW-099-7** | Scale honesty — when fetch is capped (`VIRTUAL_PAGE_SIZE` / server page), UI shows overflow affordance or “showing N of M”; never silent truncate. |
| **LAW-099-8** | Filter honesty — header count, status filter chip counts, and visible row set share one filtered view-model. |
| **LAW-099-9** | SOLID shell — `DocumentManager` is a thin shell; upload slot, feedback zone, inventory table own SRP; row actions via context/store, not 15 callback props. |
| **LAW-099-10** | CI is proof — every F-099-* has a unit or Playwright gate; inherit SPEC-048/050/086/091/098 green. |

## DRY / SOLID

| Principle | Application |
|-----------|-------------|
| **DRY** | Single `status-domain` for merge + UI + filters; single `inventoryViewModel` for counts; shared `isLiveRun` id set from zone → table. |
| **SRP** | Zone owns live narrative; table owns inventory; dropzone owns admit chrome; shell wires controllers. |
| **OCP** | New pipeline/lifecycle status extends domain maps + presentation config — not parallel helpers in badge. |
| **DIP** | Controllers expose view-models; table/zone depend on abstractions (`isLiveRun`, `displayStatus`), not on toast or dropzone internals. |
| **ISP** | Row needs status + selection + actions context — not the full Manager prop bag. |
| **LSP** | Memory/dev and postgres-backed list merges share the same domain predicates. |

## Inheritance (do not break)

| Prior law | Constraint on SPEC-099 |
|-----------|------------------------|
| LAW-098-9/10/11 | Mid-delete never paints Completed/Ready; pins + sessions remain SSOT |
| SPEC-091 IS3 | `query_ready` fence remains queryable; presentation only |
| SPEC-048 LAW-IS3 | Active View owns narrative; quiet/collapse must not remove drop capability |
| SPEC-086 | Dual-run + cancel paths remain operable in zone |

## Target StatusCell (LAW-099-3)

```ascii
BEFORE (peer pills)              AFTER (composite)
┌──────────┐ ┌───────┐           ┌─────────────────────┐
│Completed │ │ Ready │           │ Completed · Ready   │  ← one cell
└──────────┘ └───────┘           └─────────────────────┘
                                 ┌─────────────────────┐
                                 │ Indexed · not ready │  ← amber secondary
                                 └─────────────────────┘
                                   (tooltip / title explains fence)
```
