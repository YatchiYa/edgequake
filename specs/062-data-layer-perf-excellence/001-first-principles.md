# SPEC-062 — First principles

1. **Green ≠ excellent.** SPEC-061 proves no major is a release-blocker at small scale. Excellence requires headroom, release builds, and complete artifacts.
2. **Cost is physics.** ANN *query* is healthy (~1ms unfiltered). Binding walls: HNSW *insert* (~390ms) and AGE `agtype_to_json` on graph writes (pg16 2.3× slower).
3. **Measure before chase.** Degrees “3× on pg17” was p95≈max with n=15. Sample floor n≥30 + drop warmup before claiming regressions.
4. **Same SLO floor; different posture.** pg16 = legacy write lag until denormalized ids; pg17/18 = greenfield (halfvec, iterative_scan).
5. **Improve only with EXPLAIN + p95.** No silent halfvec flip, no blind global `ef_search` bump.
