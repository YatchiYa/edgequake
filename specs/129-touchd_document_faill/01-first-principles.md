# 01 — First Principles (LAW-129)

## Domain

Document processing uses **two status planes**:

```ascii
  Plane A — KV / current_stage / UI chips
            Rich vocabulary (re_embedding, queued, merging, …)

  Plane B — public.documents.status
            CHECK allowlist (migration 141): 13 values
```

Dual-write is intentional (SPEC-047 P1): list freshness must not wait for finalize stats. Dual-write must **project** A→B, never copy A blindly.

## Laws

| ID | Law | Rationale |
|----|-----|-----------|
| LAW-129-1 | **CHECK ⊂ KV vocabulary** — column values are a strict subset; never expand CHECK to host every UI stage | SPEC-098 / shell law |
| LAW-129-2 | **One mapper SSOT** — all relational status writers use `relational_documents_status_for_write` (normalize + `completed`→`indexed`) | DRY / SOLID Single Responsibility |
| LAW-129-3 | **KV honesty preserved** — slim-resume may still set KV `re_embedding` (SPEC-057 P2) | Display stage ≠ column |
| LAW-129-4 | **Non-fatal ≠ correct** — best-effort touch must succeed for valid stages; WARN is a bug signal | Operator trust |
| LAW-129-5 | **Lifecycle passthrough** — `deleting` / `delete_failed` never collapse (LAW-098-11) | Delete admit |
| LAW-129-6 | **No DDL for this fix** — application-layer projection only | Safer than CHECK widen |
| LAW-129-7 | **#377 is a trigger, not the root** — crash checkpoints expose #381; collision fix is separate | Scope |
| LAW-129-8 | **Unfakable proof** — unit matrix + postgres e2e raw-fail / touch-ok | Honest acceptance |

## CHECK allowlist (migration 141)

```ascii
  pending | processing | chunking | extracting | embedding | indexing
  completed | indexed | failed | partial_failure | cancelled
  deleting | delete_failed
```

## Projection examples

```ascii
  re_embedding      → processing
  queued            → pending
  merging / storing → processing
  partial_success   → partial_failure
  completed         → indexed   (touch/list preference after normalize)
  deleting          → deleting
```

## Cross-refs

- Why: [00-why.md](00-why.md)
- Architecture: [04-target-architecture.md](04-target-architecture.md)
- Parent CHECK: [../098-data-access-hardening/](../098-data-access-hardening/)
