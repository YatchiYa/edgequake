# LENS — Status Honesty (SPEC-099)

## Question

Can an operator trust StatusCell, filter chips, Active runs, and delete feedback to agree?

## Binding prior laws

| Law | Constraint |
|-----|------------|
| LAW-098-9 | Lifecycle admit dual-written; merge treats `deleting` as inflight |
| LAW-098-10 | Delete UI one SSOT — sessions + pins until absence |
| LAW-098-11 | `delete_failed` ≠ pipeline `failed`; Retry verbs match lifecycle |
| SPEC-091 IS3 | `query_ready` fence truth |
| SPEC-048 LAW-IS3 | Active View owns narrative |

## SPEC-099 additions

| Law | Honesty rule |
|-----|--------------|
| LAW-099-1 | One domain for normalize/display — merge and paint cannot diverge |
| LAW-099-3 | Fence presentation secondary — never fake a second “success” |
| LAW-099-6 | Toast cannot claim upload state the zone already owns |
| LAW-099-8 | Header / chips / rows share one filtered view-model |

## Forbidden paints

```ascii
 FORBIDDEN mid-delete:     Completed · Ready
 FORBIDDEN dual success:   [Completed] [Ready] as peer emerald pills
 FORBIDDEN toast lie:      "Uploading N..." while zone lists same N as Queued/Working
 FORBIDDEN count lie:      Header 17 · chip All Status (11) · unexplained
 FORBIDDEN silent cap:     Table shows 100 rows, implies corpus complete
```

## Data path (honest)

```ascii
 API / WS / sessions
   → status-domain (display + ranks)
     → mergeMonotonicList + pins
       → inventoryViewModel (counts + rows)
         → StatusCell (pipeline ⊕ fence)
         → FeedbackZone (narrative)
```

## Cross-ref

F-099-01 · F-099-02 · F-099-03 · F-099-10 · F-099-15 · Issues `ISSUE-status-ssot-unify`, `ISSUE-serving-fence-presentation`, `ISSUE-inventory-scale-honesty`
