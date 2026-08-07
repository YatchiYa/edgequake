# issue-360 — Clear All leaves documents

**GH:** https://github.com/raphaelmansuy/edgequake/issues/360  
**Duplicate / clarification:** [#366](https://github.com/raphaelmansuy/edgequake/issues/366) (same reporter; env corrected to **v0.24.1**)  
**Reported on:** form said **v0.12.11**; partner follow-up: **v0.24.1**  
**Status on HEAD:** **Confirmed on v0.24.1** — see [issue-366-clear-all.md](issue-366-clear-all.md)

## WHY

Users must trust that “delete all” empties the workspace list.

## Classification correction

Earlier SPEC-111 pass treated #360 as “mostly fixed by #309 durable wipe” because the form version was 0.12.11. Partner clarified they reproduce on **v0.24.1**. That makes the residual list/wipe SSOT gap a **live P0/P1 UX defect**, not historical only.

## Root cause

See [#366 deep dive](issue-366-clear-all.md):

- Wipe deleted typed `documents` (RM1) but left dual-write KV residue.
- List treated authoritative empty membership as “no index” and fell back to global `-metadata` suffix scan → ghosts after refresh.

## Fix

Shared with #366 (LAW-111-9 + residual KV purge + e2e). Close #360 as duplicate of #366 once the fix ships, or cross-link both.
