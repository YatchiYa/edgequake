# 06 — Edge cases

| ID | Case | Risk | Mitigation | Test |
|----|------|------|------------|------|
| EC-01 | Rolling deploy: old + new pods both hold pools | Transient 2× slot use → tip shared DB | Stagger rollouts; size budget with `INSTANCE_COUNT` reflecting max overlap; Wave A close on drain | T-112-07 |
| EC-02 | `min_connections=1` × 4 roles | 4 warm backends at idle always | Document; shared-DB profile may set min 0 if sqlx allows / lower role counts | T-112-09 |
| EC-03 | SIGKILL / OOM kill | No `pool.close()`; slots until TCP timeout | Prefer SIGTERM; PG `tcp_keepalives_*`; ops terminate stale backends | Ops runbook |
| EC-04 | PgBouncer `pool_mode=transaction` | Breaks session features: `LISTEN/NOTIFY`, some prepared stmt reuse, AGE search_path tricks | Session mode for EQ or keep direct PG; document tradeoff | Manual / partner |
| EC-05 | `RESET ALL` on `after_release` | Clears session GUCs; prepared statements may be discarded (cost) | Accept correctness > micro-opt (SPEC-090 EC-06); do not remove reset | Existing 090 + C |
| EC-06 | Empty `application_name` regression | Ops blind again | Contract T-112-01/03; fail CI if SET removed | T-112-01, 03 |
| EC-07 | Migrate CLI / one-off tools open extra pools | Budget math undercounts | Include CLI in formula; use admin-sized pool; don’t run migrate concurrent with full replica storm | Ops |
| EC-08 | `DATABASE_READ_URL` replica | Query pool on different host — primary still holds ingest/queue/admin | Budget check should split primary vs read when read URL set | Wave B detail |
| EC-09 | DBeaver / psql on same instance | Compete for slots (seen in CSV) | Reserve headroom in formula; discourage GUI on prod primary | Ops |
| EC-10 | QL `LISTEN` holds idle backends | Co-tenant idle is also capacity | Partner QL investigation; shared monitoring by `application_name` | Partner |
| EC-11 | Acquire timeout under saturation | Requests fail while idle slots exist on other roles | Role split already isolates; tune sizes; ready blocker on util | SPEC-090 multi-pool e2e |
| EC-12 | `BUDGET_MODE=fail` on upgrade | Sudden boot refuse in PPD | Default `warn`; document fail as opt-in hardening | T-112-08 |
| EC-13 | Very short `idle_timeout` | Connection churn / latency spikes | Floor timeouts; don’t set sub-second in prod | U + docs |
| EC-14 | `idle_in_transaction_session_timeout` kills long txn | Rare long admin txn aborted | Admin role longer timeout or SET LOCAL override inside migrate | Wave C docs |
| EC-15 | Multiple workspaces, one process | Still one bundle — not N pools per workspace | Keep single bundle; tenancy is data-plane | — |

## ASCII — deploy overlap

```text
  t0  pod-A holds ≤34
  t1  rollout starts → pod-B starts → holds ≤34
      ──────────────────────────────
      concurrent ≤ 68 (+ QL + tools)
  t2  pod-A SIGTERM → drain → close → slots free
  t3  steady: pod-B ≤34
```

Size `INSTANCE_COUNT` for **peak overlap**, not steady state alone.
