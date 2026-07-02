# SPEC-040 — O(N) Complexity Expert Lens

**Lens:** Algorithmic complexity, resource budgets, backpressure  
**Reference:** SPEC-006 performance guarantees, SPEC-034 graph improvements

---

## Complexity budget table

| Operation | Naive | Required | Current code | Issue |
| --------- | ----- | -------- | ------------ | ----- |
| Popular nodes (scoped) | O(V×E) nested loop | O(E_w + V_w log V_w) hash join | Filter-first CTE — `query_ops.rs:504` | #262 if plan regresses |
| Workspace node COUNT | O(V) seq scan | O(V_w) index scan | `vertex_count_sql` — `analytics_ops.rs:83` | #262 |
| Workspace edge COUNT | O(E) seq scan | O(E_w) index/filter | `edge_count_sql` — `analytics_ops.rs:107` | #262 |
| Full graph stream | O(V+E) | O(limit + E_local) | BFS capped by budget — `resource/budget.rs` | #262 timeout |
| Document list | O(all docs) | O(docs in WS) | wsdoc prefix — SPEC-027 | OK |
| KV stats aggregation | O(metadata) | O(docs in WS) | `load_workspace_metadata_values` | OK |
| Duplicate hash lookup | O(1) | O(1) | Single KV get | #253 ghost key |
| Conversation message insert | O(1) | O(1) | Single INSERT | #259 FK fail |

**Notation:** V_w / E_w = vertices/edges in workspace after filter.

---

## Issue #262 — Complexity deep dive

### Reported failure mode

- V ≈ 27,500, E ≈ 23,000
- Nested Loop Left Join → ~600M comparisons (issue #262)
- Exceeds 15s tokio timeout → **O(V×E) effectively**

### Fixed algorithm (already in code)

```
1. MATERIALIZED filtered_nodes = σ_workspace_id(V)     -- O(V_w) with index
2. edge_counts = JOIN(filtered_nodes, E, start_id)     -- O(E_w) hash join
3. JOIN filtered_nodes LEFT edge_counts                  -- O(V_w)
4. ORDER BY degree LIMIT k                               -- O(V_w log k)
```

**Why MATERIALIZED matters:** Forces PostgreSQL to compute workspace filter **once** before edge aggregation — prevents re-scanning full edge table per vertex.

Evidence: comment at `query_ops.rs:504-508`.

### Remaining O(N) risks

| Risk | Complexity | Mitigation |
| ---- | ---------- | ---------- |
| No index on workspace filter | O(V) seq scan | M078 child indexes |
| Bad join order | O(V×E) | ANALYZE + extended stats |
| `_ag_label_vertex` union | Planner scans all child tables | Acceptable if child index used |
| Multiple concurrent stats polls | k × O(V_w) | 60s cache — `stats.rs:58-74` |

---

## Timeout stack (defense in depth)

```
Layer 1: UI React Query stale-if-error     (stats.stale banner)
Layer 2: API STATS_FETCH_TIMEOUT = 4s      (stats.rs:84)
Layer 3: API graph_query_timeout = 15s     (graph_materialization.rs)
Layer 4: PostgreSQL statement_timeout      (optional ops setting)
Layer 5: Materialization semaphore = 1     (graph_materialization.rs:22-29)
```

**Principle:** Timeouts are **circuit breakers**, not fixes. #262 must fix planner, not raise layer 3 to 60s.

---

## Issue #259 — Complexity coupling

Long-running operations increase **race window** T:

```
T_race ≈ T_query + T_stream_save
```

If T_query grows from 2s → 30s (#262), probability of conversation deletion during stream rises linearly.

**Multi-workspace slowness:** Each workspace adds:

- Cold cache miss on stats (4s max)
- Graph materialization slot contention (503 if second tab)
- KV metadata load O(docs_w)

Total perceived latency ≈ **sum of sequential cold paths** — not true O(n²), but linear in workspace switches without cache warming.

---

## Issue #253 — Complexity

Duplicate check: **O(1)** KV GET on hash key — efficient.

Failure mode is **correctness**, not complexity: ghost key causes O(k) failed upload attempts (user retries k times).

Replace loop:

```
User clicks Replace k times → k × (DELETE attempt + full upload pipeline)
```

Worst case **O(k × ingest_cost)** — unbounded if UI doesn't toast on failure.

---

## Issue #251 — Complexity

`include_str!` parse at startup: **O(models)** once — fine.

`ModelsConfig::load()` file read: **O(file size)** — negligible.

Bug is **O(1) wrong branch taken** — always embed path.

---

## Issue #250 — Complexity

Version check: **O(1)** per page load if fetching `/health`.

No performance issue — observability only.

---

## Benchmark gates (CI proposal)

| Benchmark | Target | Crate |
| --------- | ------ | ----- |
| `get_popular_nodes_with_degree` 1k nodes | <100ms | `graph_sota_tests.rs` ✅ |
| `get_popular_nodes_with_degree` 27k nodes | <500ms | **Add** |
| `node_count_by_workspace` | <200ms | **Add** |
| Workspace stats endpoint | <4s p95 | Playwright + metrics |

```bash
# Local proof (seeded graph)
cargo bench -p edgequake --bench graph_performance
cargo test -p edgequake-core --test e2e_graph_performance -- --nocapture
```

---

## Backpressure interactions

| Symptom | Cause | O(N) interpretation |
| ------- | ----- | --------------------- |
| 503 Graph materialization busy | Semaphore=1 | Correct backpressure |
| Stats show 0 | Timeout + no stale cache | First request cold O(V_w) |
| Pool exhausted | Long graph + many tabs | O(concurrent) × query time |

**Recommendation:** After #262 fix, consider raising `DEFAULT_GRAPH_MATERIALIZE_CONCURRENT` to 2 **only** if p95 query <2s (measure first).

---

## Anti-patterns to reject

| Proposal | Why reject |
| -------- | ---------- |
| Full graph fallback on timeout | O(V+E) — violates SPEC-006 |
| Client-side stats from document list length | Wrong when graph entities ≠ docs |
| Disable workspace filter in SQL | Cross-tenant data leak |
| Global COUNT(*) cache without workspace key | Wrong isolation |

---

## Complexity-correct fix summary

| Issue | Fix | Asymptotic improvement |
| ----- | --- | ---------------------- |
| #262 | Child indexes + ANALYZE | O(V×E) → O(V_w + E_w) |
| #259 | Short-circuit FK + clear ID on WS switch | Reduces wasted O(query) work |
| #253 | O(1) hash delete | Stops O(k) retry loop |
| #251 | O(1) load path fix | N/A |
| #250 | O(1) version read | N/A |
