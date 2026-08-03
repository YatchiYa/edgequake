# 03 — Implementation Plan

| Wave | Work |
|------|------|
| A | `legacy_store_census` + typed unknown env + cutover refuse on empty census |
| B | Migration 142 empty DROP / abort-if-rows / `legacy_stores_forbidden` |
| C | Era-aware INV/FTS (keep dual when census>0); workspace typed count |
| D | `contract_spec105_legacy` + fix e2e_spec024; soak adjacency |
| E | CHANGELOG + upgrade docs + assessment |
