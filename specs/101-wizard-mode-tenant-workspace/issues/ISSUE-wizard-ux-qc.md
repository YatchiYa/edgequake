# ISSUE — Wizard UX QC (Wave 6)

**Findings**: F-101-11…17  
**Laws**: LAW-101-8, LAW-101-9, LAW-101-10  

## Problem

Core wizard shipped, but Aug 2026 gaps remain: no draft restore, first-run dismiss chrome dishonest, create cancel loses dirty work without confirm, weak a11y announcements, no inline validation, missing after-evidence multi-viewport captures.

## Fix

1. `wizard-draft-storage` + hydrate/persist/clear in all three wizards.  
2. `WizardShell` dismiss policy + dirty `AlertDialog` + live region.  
3. Inline `aria-invalid` on admin/name steps; Review Edit links; slug hint placement.  
4. `spec101Screenshot` + `spec101-ux-capture.spec.ts` → `evidence/after-*.png` at 1440/768/375.
