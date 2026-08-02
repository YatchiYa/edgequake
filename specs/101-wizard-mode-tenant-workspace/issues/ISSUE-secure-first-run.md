# ISSUE — Secure first-run

**Findings**: F-101-04, F-101-05, F-101-09  
**Laws**: LAW-101-4, LAW-101-7  

## Problem

Silent Default seed; env-only admin; dead `needsOnboarding`.

## Fix

`GET/POST /setup/*`; gate `ensure_defaults`; FirstRunWizard; wire store from status; PATCH auto Default Workspace on tenant create path.
