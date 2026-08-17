# ISSUE — Wizard shell DRY

**Findings**: F-101-01, F-101-06, F-101-07  
**Laws**: LAW-101-1  

## Problem

Three create dialogs diverge; orphan selector unused; colon vs slash model IDs.

## Fix

Single `WizardShell` + shared steps; header/guard thin; delete `tenant-workspace-selector.tsx`; slash-only `model-payload.ts`.
