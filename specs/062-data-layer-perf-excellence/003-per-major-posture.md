# SPEC-062 — Per-major posture

| Major | Role | Optimize | Do not pretend |
|-------|------|----------|----------------|
| **pg16** | Legacy (AGE 1.6) | Native writes + denormalized id columns; accept write lag until Wave 1 lands | AGE 1.6 matches 1.7 write cost |
| **pg17** | Managed modern | Same code as 18; iterative_scan; halfvec after recall gate | One-sample degrees spikes are product bugs |
| **pg18** | Recommended greenfield | Default tip; halfvec + HNSW GUCs | Unfiltered ANN proves filtered RAG |

**Install guidance:** new → pg18; managed PG17 → pg17 image; pg16 → stay until Wave 1, then reassess upgrade.
