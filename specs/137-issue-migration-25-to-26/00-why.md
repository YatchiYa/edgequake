# 00 — Why SPEC-137

## Trigger

A field fleet serving **v0.25.0** upgrades the **binary/image to v0.26.0** and
runs the migrate CLI. SAFE SCHEMA lands (migration **149** when pending). DROP
OLD migrations **125 / 126 / 131** (and deferred **142**) remain. The operator
then tries to finish the human-gated drop with a confirm flag, then `migrate
guard`, then the same confirm command again.

The 0.26 upgrade note documents only:

```text
edgequake migrate dry-run
edgequake migrate
```

That is correct for a fleet that **already** applied 125–131. It is incomplete
for a mid-cutover fleet that has been legally serving with DROP OLD pending
since SPEC-091.

## User impact

| Layer | Impact |
|-------|--------|
| Ops | Cannot tell whether confirm was ignored or SQL aborted; retries do nothing useful |
| Product | 0.26 looks like a migrate regression; 149 is not the blocker |
| Serving | API may already be legal to start (SAFE SCHEMA complete) |
| Trust | Confirm token spelling / silent ignore looks like a broken CLI |

## Why this is a product defect (not “operator error” alone)

- `--drop-confirm` is a reasonable permutation of `--confirm-drop`. Pre-fix, it
  was **not** consent and was **not** rejected.
- Advisor action tags said `[requires --confirm]` — a third name.
- Abort stderr pointed at `public.tasks` locks for Wave D / IW2 failures.
- The 0.26 runbook hid the 091 ladder that this fleet is still on.

SQL abort on uncovered rows is **not** a defect. Weakening DROP guards would be.

## Non-goals

- Auto-applying DROP OLD.
- Dropping Apache AGE graph schemas.
- Re-scoring Acc / PDF geometry.
- Inventing a second migrate CLI.

## Success condition

1. `--drop-confirm` is accepted as consent (same as `--confirm-drop`).
2. Unknown apply flags exit non-zero with a hint.
3. SQL abort stderr names the **class** (KV / chunk vectors / fleet provenance /
   142 / checksum / lock).
4. Upgrade-to-0.26.0 documents leftover 125/126/131.
5. E2E-137-01..09 green via `make spec137-migrate-025-026-proof`.
