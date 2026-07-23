# 083 — LightRAG Query API Law (Product Parity)

**Status:** **Shipped** product Equal LightRAG (query API) · Acc Beat / Acc Equal mid **STOP** ([085](./085-fairness-concurrency-equal-stop.md))  
**Date:** 2026-07-23  
**Parent:** [082](./082-gold-citation-compat.md) · [061 Idea G](./061-lightrag-law-first-principles-eq.md)  
**Stakeholder brief:** [019](../019-business-eq-vs-lightrag-and-rag.md)  
**LR source:** `LightRAG/lightrag/base.py` `QueryParam` · `operate.py` `get_keywords_from_query` / `kg_query`  
**Acc SSOT:** frozen P0 mid (`medical-mid-20260722T104918Z`) — **not** promoted by this pack  
**Labeled peer:** [`PRODUCT_QUERY_API_v1`](../e2e/artifacts/publish/peers/PRODUCT_QUERY_API_v1/)

---

## 1. Decision

Equal LightRAG on the **agent/product query API**, not Mix packing Acc:

| Law | LightRAG | EdgeQuake (083) |
|-----|----------|-----------------|
| Pre-supplied `hl_keywords` / `ll_keywords` | Skip keyword LLM | Same |
| Answer roles | system = prompt+context; user = query | Default `chat()` split; `EDGEQUAKE_ANSWER_COMPLETE_BLOB=1` rollback |
| `response_type` | Formats answer style | Optional request field (default Multiple Paragraphs) |
| `context_only` | `only_need_context` | Already present |

**Not claimed:** Acc Beat / mid Parity. Acc Equal mid path **STOP** after fairness rebench ([085](./085-fairness-concurrency-equal-stop.md)); Acc CI stays **labeled keep only** (E2 / fair chat-split).

---

## 2. Forbidden

080–082 packing / TOPIC / soft Mix / B7–B9 / B10 Acc promote / F4 always-on / synonym Acc fishing / silent Acc B5 overwrite / Acc `publish/latest` replace.

---

## 3. Gates (closeout)

| Gate | Result |
|------|--------|
| Unit — hl/ll skip keyword extractor | PASS (`hl_ll_override_skips_keyword_extractor`) |
| Unit — chat system/user default | PASS (`test_generate_uses_chat_system_user_by_default`) |
| Unit — gold-compat strip `[N]` | PASS (`grounding::` suite) |
| Contract — OpenAPI / `schema.d.ts` | PASS (`hl_keywords`, `ll_keywords`, `response_type`) |
| Latency — B5 WS keyword stage | PASS control **909** → override **0** ms (`T035127Z`); closeout re-smoke `T035417Z` both 0ms (warm) · Acc latest file hashes unchanged |
| Acc `publish/latest` | Untouched by 083 smoke; remains P0 mid SSOT |

**Success claim:** EQ query API matches LightRAG keyword-override + system/user generate — **not** “EQ beats LightRAG Acc.”

---

## 4. Reproduce

```bash
# Unit
cargo test -p edgequake-query --lib keyword_override
cargo test -p edgequake-query --lib test_generate_uses_chat
cargo test -p edgequake-query --lib grounding::
cargo test -p edgequake-api --lib builds_keyword_override

# Labeled latency (keyword skip) — no Acc promote
./tools/bench001/scripts/run_product_query_api_latency.sh
```

Success: `keyword_time_ms ≈ 0` with hl/ll override; peer `PRODUCT_QUERY_API_v1`; Acc latest not rewritten.
