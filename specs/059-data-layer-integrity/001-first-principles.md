# SPEC-059 — First principles

1. **Same Postgres ≠ one transaction.** Integrity = atomic insert detection + retract-on-every-terminal-path + crash janitor — not 2PC.
2. **Insert detection must be atomic with the write.** `RETURNING (xmax = 0) AS inserted` (or memory write-lock). Preflight `get_by_ids` is TOCTOU.
3. **Cancel/fail is a retract signal.** HTTP/WS/PDF/pipeline/orphan/stuck must unindex; do not rely on a live worker checkpoint alone.
4. **Isolation and property merge stay in SQL** (M090/M091). Concurrent races must be proven under READ COMMITTED.
5. **July 2026 pgvector:** `iterative_scan=relaxed_order`; new indexes `ef_construction=64` (prod tip 128 + REINDEX); halfvec greenfield recommended after measured recall ≥99% of full — never silent prod DROP.
