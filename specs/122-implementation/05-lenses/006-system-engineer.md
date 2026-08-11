# Lens 006 — System Engineer

## Stake

Docker vs `make dev` profiles diverge. Partners on Docker (#361/#365) do not see local tenant=1 clamps — yet still feel LLM-bound latency. Operators need one runbook.

## Profile divergence

```ascii
  make local Ollama     Docker compose          make cloud
  ─────────────────     ──────────────          ──────────
  workers 2             workers 8               workers 16
  tenant  1             tenant  6               tenant  12
  extract 1             extract 4               extract 32
  vision  1             vision  ~default        vision  4
```

## Operator runbook (bulk slow)

1. `GET /api/v1/pipeline/queue-metrics` — pending, park waiters, contention.
2. Confirm provider: Ollama vs Mistral/OpenAI; model pull; rate limits.
3. Confirm env caps match intended profile.
4. For local raise: `OLLAMA_NUM_PARALLEL`, VRAM, then `EDGEQUAKE_ALLOW_LOCAL_HIGH_CONCURRENCY=1`.
5. PDF: check vision host; `EDGEQUAKE_PDF_VISION_JOBS` × concurrency RSS.
6. Logs: `/tmp/edgequake-backend.log` stage errors.

## Failure modes ranked

| Rank | Mode             | Signal                           |
| ------| ------------------| ----------------------------------|
| 1    | LLM RTT × chunks | High extract time; provider busy |
| 2    | Tenant park      | `tenant_park_waiters` > 0        |
| 3    | PDF vision       | Long PdfProcessing               |
| 4    | Provider 429/503 | Errors in log                    |
| 5    | DB contention    | store_contention; SPEC-090       |
| 6    | Transfer bound   | Rare; admit ≈ total              |

## Cross-refs

- Code matrix: [../03-code-as-is.md](../03-code-as-is.md)
- Ops: `docs/operations/performance-tuning.md`
