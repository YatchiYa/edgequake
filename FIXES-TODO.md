# EdgeQuake — Backlog de correctifs (sprint mode)

> Dérivé de [ARCHITECTURE-DEEP-DIVE.md](ARCHITECTURE-DEEP-DIVE.md) §14 et [INCIDENT-PROD-DIAGNOSIS.md](INCIDENT-PROD-DIAGNOSIS.md).
> **État au HEAD v0.21.0 (`7d1d44c9`).** La majorité de ce backlog a été traitée par l'équipe EdgeQuake sous la **vague SPEC-083** (commit `569defc4`). Cases cochées = corrigé **vérifié dans le code** ; ☑ = revendiqué SPEC-083 non re-vérifié ; `[ ]` = **toujours ouvert**.
> Légende effort : **S** ≤ ½ j · **M** 1-3 j · **L** > 3 j.

---

## 🔥 Sprint 0 — Incident prod SPEC-062 — RÉSOLU en substance, 1 réserve

### Code — corrigé en 0.21.0
- [x] **Fallback `pg_node_degrees_batch`** — `nodes_ops/read.rs:149-159` : probe de schéma + `COALESCE(eq_*, agtype)` / prop-only. **Le chat ne casse plus.** ✅
- [x] **Fallback `pg_get_incident_edges_batch`** — `edges_ops.rs:361-389` : idem + log `eq_id_fallback_used`. ✅
- [x] **DDL hors hot-path** — `graph_lifecycle.rs:417-475`, `session.rs:98-137` : `lock_timeout` 5 s, `statement_timeout=0`, probe O(1) + single-flight. ✅
- [x] **Arbiter de repli upsert natif** — gate fail-closed + arbitre `eq_*` (D-30). ✅

### ⚠️ Réserve — TOUJOURS OUVERT sur très gros graphe
- [ ] **P0** Backfill DDL `eq_*` **batché** — `graph_lifecycle.rs:445-454` : reste un `UPDATE` global non batché — M
- [ ] **P0** Index `eq_*` en **`CONCURRENTLY`** — `graph_lifecycle.rs:455-474` : `CREATE INDEX IF NOT EXISTS` sans CONCURRENTLY → verrou possible — S
- [ ] **P0 ops** Sur un graphe existant 178k+ : appliquer d'abord la **procédure manuelle de `INCIDENT-PROD-DIAGNOSIS.md`** (kill scans → backfill batché → index CONCURRENTLY) avant de laisser le boot faire — S

### Corrigé mais à vérifier
- [x] **Front** : frame SSE `{"type":"error"}` affichée (X-16/S-13 cluster) — ☑ à confirmer côté UI
- [ ] **Test cassé** `contract_spec047_p7ef_graph_upsert.rs:46-62` : asserte `eq_merge_graph_properties` (0 occurrence après D-30) — corriger ou `#[ignore]` — S
- [ ] **Réserve D-30** : vérifier que l'**accumulation de `source_ids`/`source_chunk_ids`** survit au merge natif `properties = EXCLUDED.properties` (last-write-wins) — M

---

## 🔴 Sprint 1 — Sécurité & isolation — CORRIGÉ (vague SPEC-083)

- [x] **Isolation tenant WebSocket** — `websocket.rs:69-80,293-318` : `WsSession` + filtrage par `workspace_id` + test `e2e_ws_tenant_a_never_sees_tenant_b` ✅
- [x] **Ownership `track_id`** (WS/PDF) — dérivé de la session authentifiée ✅
- [x] **RLS fail-closed + FORCE** — **migration `096_rls_fail_closed_force.sql`** : `ENABLE`+`FORCE`, policies `IS NOT NULL AND =`, branches fail-open supprimées ✅ (couvre RLS inerte + fail-open + `document_originals`)
- [x] **JWT `iss`/`aud`/`jti`** — validés + denylist jti dans `verify_token` (`jwt.rs:173-179,271-274`) ✅
- [x] **`Role::parse` fail-open** — chemin JWT bascule sur `try_parse` fail-closed (`jwt.rs:134,268`) ✅
- [x] **Rate limit** — clé = identité authentifiée (`middleware.rs:670-687`) + `start_cleanup_task` spawné + test anti-spoof ✅
- [x] ☑ **JWT_SECRET bloque le boot** (S-09) — revendiqué, à confirmer
- [x] ☑ **CORS restrictif** (S-10) — revendiqué, à confirmer
- [x] ☑ **Filename/MIME** (S-12) — revendiqué, à confirmer
- [x] ☑ **`eval()` → `literal_eval`** (S-13) — revendiqué, à confirmer
- [x] ☑ **Namespaces RLS unifiés** (S-05) — revendiqué, à confirmer

---

## 🟠 Sprint 2 — Bugs de correction — CORRIGÉ

- [x] **Normalisation entités (3 bugs)** — `entity_id.rs:191-223` réécrit : NFC → casefold → strip → possessif ASCII+U+2019 → title → upper. **Vérifié** (`.nfc()`, `.to_lowercase()`, `’`) ✅
- [x] **Gleaning avec `CompletionOptions`** — `gleaning.rs:203-207` ✅
- [x] **`cosine_similarity`** → `Result` (plus de panic) — `embedding.rs:105-128` ✅
- [x] **Cache d'extraction inerte** → **fichier `pipeline/cache.rs` supprimé** ✅
- [x] **Multigraphe (D-30)** — `eq_rel_type` + arbitre 3-col + migration 097 ✅
- [x] **Poids relation** → `WeightPolicy::Max` associatif (`weight_policy.rs`) ✅
- [x] ☑ **Offsets Pdf/Markdown** (C-15) · **Blocs atomiques** (C-16) · **KV upsert transactionnel** (C-22) · **N+1 chunk contents** (C-21) · **Dédup documents** (C-23) · **`matches_track_id` Deletion*** (C-24) · **Contrat PDF 100 MiB** (D-44) · **Multipart** (D-51) — revendiqués SPEC-083, à confirmer

---

## 🟢 Sprint 3 — Dette & qualité

### Toujours ouvert
- [ ] **Backoff sans jitter** — `worker.rs:260-268` : ajouter `rand` + jitter ±20 % — S
- [ ] **X-35 « accuracy vs corpus »** — marqué FIXED mais le benchmark montre la décroissance (0.770→0.724, §12.5). **Ne pas communiquer « résolu ».** Chantier de fond réel — L

### Corrigé (vérifié)
- [x] **Shutdown drain timeout** — `tasks/shutdown.rs` + `worker.rs:912-949` ✅
- [x] **`query_vec`** embarque la question seule — `query_pipeline.rs:483-508` ✅
- [x] **Doc « weighted blend » vs max** aligné — `modes/mix.rs:315-343` ✅
- [x] **`min_score`** toujours appliqué — `chunk_retrieval.rs:234-243` ✅

### Corrigé (revendiqué SPEC-083 — à confirmer)
- [x] ☑ Partitions `audit_logs` (D-45) · OTEL/env_filter (D-46) · `last_accessed` LRU (C-27) · progression pondérée (D-41) · ETA (D-42) · tokenizer unifié (D-53) · QueryStats/StreamStats (D-40) · `chunk.score` (D-37) · conflit de type loggé (D-32) · cap/lignée (D-33) · double-gate (D-34)
- [x] ☑ Nettoyage code mort : `validation.rs`/`sanitizer.rs`, `SOTAExtractor`, migration 002 (X-01), `crates/edgequake-llm/` fantôme
- [x] ☑ Build/CI/SDK : workflows SDK (D-48) · `sed -i` (D-49) · `make postgres-start` (D-47) · `schema.d.ts` (X-26) · gates CI (X-32) · versions SDK (X-33) · config unifiée (X-36)

---

## ⛔ Rétractés (n'étaient pas des défauts)
- [x] ~~`drop_workspace_table` préfixe manquant~~ — le préfixe est correct (aussi rétracté par SPEC-083, C-19)
- [x] ~~Table `unk_ids`~~ — relation **interne à Apache AGE**, pas EdgeQuake

---

## Suivi 0.21.0

| Sprint | Fait (vérifié) | Revendiqué | Ouvert |
|---|---:|---:|---:|
| 🔥 S0 incident | 4 | 1 | **5** (backfill/index/ops + test + réserve source_ids) |
| 🔴 S1 sécurité | 6 | 5 | 0 |
| 🟠 S2 correction | 6 | 8 | 0 |
| 🟢 S3 dette | 4 | ~20 | **2** (jitter, accuracy) |

> **Priorité restante :** (1) le backfill DDL `eq_*` batché + CONCURRENTLY sur gros graphe (§0.2) ; (2) confirmer la réserve `source_ids` du merge natif (D-30) ; (3) le test cassé ; (4) le jitter. Le reste est corrigé ou revendiqué.
> **Ne pas communiquer** « accuracy degradation résolue » (X-35) ni « beats LightRAG » — le benchmark dit égalité/derrière (§12.5).
