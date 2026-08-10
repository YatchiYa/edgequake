# 01 — Finding Register (SPEC-015V)

| ID        | Finding                                           | Severity | Gate                       |
| -----------| ---------------------------------------------------| ----------| ----------------------------|
| F-015V-1  | Vision always runs fig/page/chart writers         | P0       | Unit + e2e flags OFF       |
| F-015V-2  | No workspace metadata for extract bools           | P0       | Workspace PUT round-trip   |
| F-015V-3  | No upload multipart for extract bools             | P0       | Upload FormData + resolve  |
| F-015V-4  | Pass A prompt not overridable                     | P0       | Builder unit + convert     |
| F-015V-5  | Pass B image/chart/figure prompts not overridable | P0       | `*_analysis_messages` unit |
| F-015V-6  | Document parsing UI has no modality toggles       | P1       | Playwright wizard          |
| F-015V-7  | Upload UI has no modality toggles                 | P1       | Playwright documents       |
| F-015V-8  | `process_options=i` ≠ crop extraction             | P1       | EC9 test                   |
| F-015V-9  | No ingest snapshot of resolved vision extract     | P2       | Doc metadata assert        |
| F-015V-10 | Charts ON + Figures OFF fig-as-chart edge         | P1       | EC2 unit/e2e               |
