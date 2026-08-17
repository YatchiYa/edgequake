# SPEC-001 Acc attestation for v0.24.1 cut

- Pack: `specs/001-benchmark/e2e/artifacts/publish/latest/scorecard.json`
- `valid`: true
- Stage: medical-mid
- Profile: `P0_mistral_small_mix_chunk1200_v1_lrlike_arms_v2`
- Task: GraphRAG-Bench/EQ-vs-LR
- Artifacts present: BUSINESS_REPORT.md, EXEC_SUMMARY.txt, SUMMARY.md, scorecard.json, meta.json
- Pack mtime: 2026-08-03 (same calendar day as cut)
- Decision: attest current `valid: true` pack for patch cut (SPEC-106 storage-only; no retrieval path change)

Attested by release agent for v0.24.1 per release-and-cd.md “explicitly attested current valid:true pack”.
