# 01 — Finding Register (SPEC-099)

Severity: **P0** = honesty/DRY bug factory or unusable busy viewport · **P1** = high cognitive / error-prevention · **P2** = polish / scale.

| ID | Severity | Finding | Law | Inherits / supersedes |
|----|----------|---------|-----|------------------------|
| **F-099-01** | P0 | Dual status SSOT: `status-domain.ts` and `status-badge.tsx` both export normalize/display/terminal/processing helpers; callers split | LAW-099-1 | Enables LAW-098-10 honesty |
| **F-099-02** | P1 | Peer dual pills `Completed` + `Ready` (emerald + emerald) — fence reads as second success | LAW-099-3 | SPEC-091 IS3 semantics kept; presentation supersedes SPEC-030 noise notes |
| **F-099-03** | P0 | Triple narrative on upload: toast + Active runs + table badges for same session | LAW-099-2, LAW-099-6 | SPEC-048 quiet incomplete |
| **F-099-04** | P1 | Dropzone `quiet` densifies but does not collapse — toolbar height competes with ≤35vh zone | LAW-099-4 | SPEC-048; SPEC-030 F-DOC-07 |
| **F-099-05** | P1 | Clear All peer to Refresh in page header — destructive proximity | LAW-099-5 | **Supersedes** SPEC-030 F-DOC-01 |
| **F-099-06** | P1 | Active run cards are tall (stepper + dual progress bars) — zone fills 35vh with few items | LAW-099-2, LAW-099-4 | SPEC-086 density |
| **F-099-07** | P2 | Live table rows still show pulsing status badges while zone narrates (subtitle hidden, badge not demoted) | LAW-099-2 | SPEC-048 LAW-IS3 partial |
| **F-099-08** | P1 | `DocumentManager` ~1090 LOC god-composer; dual `resolvePipelineUiState` (manager vs toolbar); ~20 row action props | LAW-099-9 | SPEC-029 DI-02 selection bar debt |
| **F-099-09** | P1 | `VIRTUAL_PAGE_SIZE = 100` silent truncate — no “N of M” / overflow | LAW-099-7 | GH-319; SPEC-030 F-DOC-02 |
| **F-099-10** | P1 | Filter/header count parity break (e.g. Documents 17 vs All Status 11) | LAW-099-8 | evidence 02 |
| **F-099-11** | P2 | `NEW` badge on Created column adds scan noise when relative time already conveys recency | LAW-099-4 | SPEC-030 F-DOC-03 |
| **F-099-12** | P2 | Cost column always visible — secondary for primary inventory scan | progressive disclosure | SPEC-030 F-DOC-04; SPEC-029 DI-03 |
| **F-099-13** | P2 | `ux-ui-audit.spec.ts` looks for `.dropzone` / `[data-upload]` but dropzone uses `data-testid="document-dropzone"` — audit may no-op | LAW-099-10 | tooling |
| **F-099-14** | P1 | Toolbar pipeline banner can remain while feedback zone open (demote incomplete for stuck) — triple chrome risk | LAW-099-2 | SPEC-051 feedback zone |
| **F-099-15** | P2 | Failed row highlight uses raw `doc.status === 'failed'` — misses `delete_failed` / display_status | LAW-099-1 | LAW-098-11 |
| **F-099-16** | P2 | Selection mode adds a second toolbar row instead of replacing header (Gmail/Linear pattern) | LAW-099-9 | SPEC-029 DI-02 |

## Evidence map

| Finding | Screenshot / code |
|---------|-------------------|
| F-099-02, F-099-05, F-099-11, F-099-12 | [`evidence/01-idle-completed.png`](evidence/01-idle-completed.png) |
| F-099-03, F-099-04, F-099-06, F-099-10 | [`evidence/02-busy-active-runs.png`](evidence/02-busy-active-runs.png) |
| F-099-06 (tall card lineage) | [`evidence/03-legacy-active-card.png`](evidence/03-legacy-active-card.png) |
| F-099-01 | `status-domain.ts` L86–195 · `status-badge.tsx` L144–238 |
| F-099-03 toast | `hooks/use-file-upload.ts` ~L197 (`toast.loading`) |
| F-099-09 | `document-manager.tsx` `VIRTUAL_PAGE_SIZE = 100` |

## Closed-by-prior (not reopened as new work)

| Prior ID | Status under SPEC-099 |
|----------|------------------------|
| SPEC-098 delete dual-SSOT / pins | **Must stay green** — not weakened |
| SPEC-091 fence truth (`query_ready`) | **Semantics kept** — F-099-02 is presentation only |
| SPEC-048 Active runs ownership | **Kept** — F-099-03/04 refine disclosure |
| SPEC-030 F-DOC-05 file-type icons | Appears fixed in evidence 01 (PDF vs MD) — no F-099 |
