# 00 — Issue data (anonymized)

> **Source:** Field ticket + reconstructed CLI (no names, hosts, or secrets).
> **From:** v0.25.0 serving · **To:** v0.26.0 migrate CLI
> **DB:** `postgresql://edgequake:***@db.example.internal:5432/edgequake`
> **Raw:** [raw-logs/](raw-logs/)

## Command sequence

```bash
edgequake migrate
edgequake migrate --drop-confirm
edgequake migrate guard
edgequake migrate --drop-confirm
```

## Facts extracted (code is law)

| Fact | Value |
|------|-------|
| Product additive in 0.26.0 | Migration **149** (`tasks.document_id`) — SAFE SCHEMA |
| Human-gated remaining | **125**, **126**, **131** (+ **142** while legacy rows exist) |
| Canonical consent | `--confirm-drop` / `EDGEQUAKE_MIGRATION_CONFIRM_DROP` |
| Token in ticket | `--drop-confirm` (three times) |
| `migrate guard` | Read-only; never applies sqlx versions |
| AGE | Not dropped by 125/126/131 (`public.eq_*` only) |

## Track pin

| Track | When | Exit | Operator sees |
|-------|------|------|----------------|
| **A** | `--drop-confirm` ignored | 0 soft-exit **or** remaining DROP OLD | Consent NOT given |
| **B** | `--confirm-drop` (or env) | Non-zero | `RAISE EXCEPTION` / checksum refuse |

Ticket wording matches **A** (literal flag). **B** remains a required product
path: production KV/vector residue must fail closed, with an honest hint.

## Preflight shape (mid-cutover 0.25 + 0.26 binary)

```text
pending 125  [DROP OLD — irreversible KV tables]
pending 126  [DROP OLD — irreversible chunk vectors]
pending 131  [DROP OLD — irreversible vector fleet]
pending 142  [ASSERT — SPEC-105]     # deferred while any_legacy_rows
pending 149  [SAFE SCHEMA]           # if not yet applied
```

149 applies on `edgequake migrate` without confirm (ExpandableOnly). Drops do
not.

## What 0.26 is not

Migration 149 cannot block 125. sqlx ExpandableOnly omits irreversible versions
so 132–149 can land while 125/126/131 stay pending ([sqlx Migrator](https://docs.rs/sqlx/latest/sqlx/migrate/struct.Migrator.html)
applies unapplied older versions later — [PR #1030](https://github.com/launchbadge/sqlx/pull/1030)).
