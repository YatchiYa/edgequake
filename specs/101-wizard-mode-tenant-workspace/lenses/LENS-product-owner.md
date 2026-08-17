# LENS — Product Owner (SPEC-101)

## Job to be done

Admin lands on a fresh EdgeQuake instance and reaches a **selected workspace with resolved models** without reading ops docs or fighting chip-dense modals.

## Activation event

First workspace ready (tenant + workspace + models resolved, possibly via server defaults).

## Success metrics

| Signal | Target |
|--------|--------|
| Time to first workspace | ≤ 3 minutes guided |
| Happy-path steps without Advanced | Admin (if needed) → Tenant name → Accept defaults → Workspace name → Review |
| Support tickets “which model?” | Drop — defaults always labeled |
| Context awareness (LAW-101-11) | User can name active Tenant **and** Workspace without opening the menu |

## Context job-to-be-done

At any dashboard surface, the operator must answer “which org / which workspace am I in?” from the header alone — not from the URL.

## Non-goals

- Teaching the full model hub  
- Billing / invites on day one  
- Replacing env bootstrap for automated fleets  

## Copy principles

- Every step title answers “what am I deciding?”  
- Every subtitle answers “why does this matter?”  
- Defaults language: “Using server defaults” + monospace `provider/model`, never bare “Server default” alone.
