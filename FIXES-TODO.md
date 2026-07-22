# EdgeQuake — Backlog de correctifs (sprint mode)

> Dérivé de [ARCHITECTURE-DEEP-DIVE.md](ARCHITECTURE-DEEP-DIVE.md) §14 et [INCIDENT-PROD-DIAGNOSIS.md](INCIDENT-PROD-DIAGNOSIS.md).
> Version cible : **v0.20.2** (`d96f0725`). Chaque item = fichier:ligne + réf §.
> Légende effort : **S** ≤ ½ j · **M** 1-3 j · **L** > 3 j. Priorité : 🔥 P0 (feu prod) · 🔴 P1 (sécu/bloquant) · 🟠 P2 (correction) · 🟢 P3 (dette).

---

## 🔥 Sprint 0 — Hotfix prod (l'incident SPEC-062)

> Objectif : chat + ingestion + CPU DB reviennent à la normale. Détail SQL dans `INCIDENT-PROD-DIAGNOSIS.md`.

### Ops (sans redéploiement) — à faire MAINTENANT
- [ ] **Tuer les requêtes graphe > 1 min** qui tiennent les verrous (`pg_terminate_backend` sur les scans `agtype … LIKE`) — §0.1
- [ ] **Poser le schéma `eq_*` à la main** : `ALTER TABLE … ADD COLUMN eq_node_id/eq_source_id/eq_target_id` avec `lock_timeout='5s'` — S
- [ ] **Backfill batché** des colonnes eq_* (par `ctid`, LIMIT 20000) pour éviter la transaction géante — S
- [ ] **Créer les index en `CONCURRENTLY`** : `idx_node_eq_node_id`, `idx_edge_eq_source_target`, `idx_edge_eq_source_id/target_id` — S
- [ ] **Créer les GIN `CONCURRENTLY`** : `idx_node_source_ids_gin`, `idx_edge_source_ids_gin` (sinon la requête 30 min persiste) — S
- [ ] **Restart edgequake** → vérifier `eq_id_schema_ready()=true` (plus de DDL au boot) — S
- [ ] **Vérifier** : `EXPLAIN ANALYZE` de la degree query < 100 ms + `pg_stat_activity` sans scan `agtype…LIKE` — S
- [ ] **Vérifier l'env prod** : `EDGEQUAKE_SOURCE_PREFIX_LEGACY` (doit être off) et `SHOW statement_timeout` — S

### Code (prochaine release) — empêcher la récidive sur tout gros graphe
- [ ] **P0** Fallback dans `pg_node_degrees_batch` — `nodes_ops/read.rs:148` : si `eq_id_schema_ready=false`, utiliser l'expression `agtype_to_json(properties)->>'source_id'` au lieu de `e.eq_source_id` en dur (= chat down aujourd'hui) — M
- [ ] **P0** Fallback dans `pg_get_incident_edges_batch` — `edges_ops.rs:362` (traversée, même bug, aucun gate) — M
- [ ] **P0** DDL SPEC-062 robuste — `graph_lifecycle.rs:409` : `lock_timeout` court sur les `ALTER`, backfill batché, index `CONCURRENTLY` (pas d'`UPDATE` monolithique sous verrou) — M
- [ ] **P0** Arbiter de repli pour l'upsert natif quand l'unique `eq_*` n'existe pas encore (sinon ingestion en boucle) — `mutate.rs:400`, `edges_ops.rs:566` — M
- [ ] **P1** Corriger le nouveau test contractuel cassé `contract_spec058_native_upsert_uses_eq_merge_graph_properties` (exige une chaîne que SPEC-060 a supprimée) — §14 #20 — S
- [ ] **P2** Front : afficher la frame SSE `{"type":"error"}` (chat muet actuellement sur erreur) — S

---

## 🔴 Sprint 1 — Sécurité & isolation

- [ ] **Isolation tenant WebSocket** — `websocket.rs:53-66,167` : filtrer le broadcast par tenant (identité jetée aujourd'hui, tout client voit tous les tenants) — §14 #1 — M
- [ ] **Ownership du `track_id`** — `cancel_facade.rs:18`, `pdf_upload/status.rs:262` : vérifier que le track appartient à l'appelant (WS cancel + SSE PDF) — §14 #2 — M
- [ ] **RLS de facto inerte** — `rls.rs:220` : trancher — soit vraies transactions autour de `set_tenant_context`, soit assumer explicitement le `WHERE` applicatif et documenter (le chemin actuel est `#[deprecated]` mais toujours actif) — §14 #3 — L
- [ ] **RLS fail-open** — `001:510` : retirer `tenant_id IS NULL OR …` et/ou rendre `documents.tenant_id NOT NULL` — §14 #4 — M
- [ ] **`document_originals` sans RLS** — M082 : ajouter `ENABLE ROW LEVEL SECURITY` + policy (comme `pdf_documents`) — §14 #6 — S
- [ ] **JWT `iss`/`aud`/`jti`** — `jwt.rs:162` : valider iss/aud au decode ; stocker jti pour permettre la révocation d'access token (logout ne révoque que le refresh) — §14 #7 — M
- [ ] **`Role::parse` fail-open** — `types.rs:28` : fail-closed (rôle inconnu → refus, pas `User`) — §14 #8 — S
- [ ] **JWT_SECRET** — `startup_security.rs:39` : bloquer le boot (pas juste warn) si secret par défaut ; vérifier ≥ 32 bytes — §14 #9 — S
- [ ] **CORS `Any/Any/Any`** — `server.rs:84` : défaut restrictif ; `ws_validate_origin` fail-closed si Origin absent — §14 #10 — S
- [ ] **Rate limit** — `middleware.rs:631` : dériver la clé du tenant **authentifié** (l'auth tourne avant), pas du header brut ; appeler `cleanup_stale_buckets` (fuite mémoire) — §14 #11 — M
- [ ] **Upload** — `pdf_upload/upload.rs:112`, `file_validation.rs:111` : assainir le filename ; confronter le MIME au contenu, pas à l'extension — §14 #12 — M
- [ ] **`.env.example` vision** — `.env.example:36` : ne pas fixer `VISION_PROVIDER=openai` en dur (un utilisateur Ollama enverrait ses PDF à OpenAI) — §14 #50 — S
- [ ] **`eval()` bench** — `mmlongbench_eval_score.py:138` : remplacer par `ast.literal_eval` — §14 #13 — S

---

## 🟠 Sprint 2 — Bugs de correction

- [ ] **Normalisation entités (3 bugs)** — `entity_id.rs:198-213` : (a) normaliser la casse AVANT le strip d'article (`THE COMPANY`→`THE_COMPANY` aujourd'hui) ; (b) gérer l'apostrophe typographique U+2019 (`’s`) — la branche actuelle est morte (littéraux ASCII) ; (c) possessif case-insensitive. **Cause des nœuds dupliqués** — §14 #14 — M
- [ ] **Offsets Pdf/Markdown** — `page_aware.rs:171`, `markdown_chunking.rs:49` : rebaser les spans du segment sur le document (`base_offset`) — casse le lineage page multi-page — §14 #15 — M
- [ ] **Blocs atomiques sans garde de taille** — `recursive.rs:385` : borner/splitter une région atomique géante (un gros tableau = un chunk qui casse l'embedder) — §14 #16 — S
- [ ] **Gleaning sans `CompletionOptions`** — `gleaning.rs:204` : passer `extraction_completion_options` (sinon pas de `max_tokens`/`temperature:0`/`reasoning_effort:none`) — §14 #17 — S
- [ ] **`CHUNK_MAX_RETRIES=0`** — `extraction.rs:302` : `1..=0` = zéro tentative ; passer à `0..=max` ou clamper le min à 1 — §14 #18 — S
- [ ] **`kv.rs::upsert` non transactionnel** — `kv.rs:257` : envelopper les chunks dans une transaction (échec à mi-parcours laisse des lignes) — §14 #22 — S
- [ ] **`batch_fetch_chunk_contents` N+1** — `chunk_content.rs:30` : utiliser `get_by_ids` (existe déjà) — §14 #21 — S
- [ ] **Dédup `documents` cassée** — M023 vs M032 : le prédicat `WHERE status='indexed'` rate le statut moderne `'completed'` — §14 #23 — S
- [ ] **`cosine_similarity` panique** — `embedding.rs:84` : retourner `Result` au lieu de `assert_eq!` sur mismatch de dimension — §14 #28 — S
- [ ] **`matches_track_id` ignore `Deletion*`** — `websocket.rs:573` : ajouter les variantes qui portent un `track_id` — §14 #24 — S
- [ ] **Contrat PDF 100 MiB** — `pdf_storage.rs:565`, `injection_file.rs:82` : aligner le contrat (50 MiB réel) ; corriger le message « 10 MB » — §14 #44 — S
- [ ] **Multipart en RAM** — `upload/file_upload.rs:69` : streamer l'upload ; caper le nombre de fichiers en batch — §14 #51 — M

---

## 🟢 Sprint 3 — Dette & qualité

### Fiabilité / ops
- [ ] **Backoff sans jitter** — `worker.rs` : ajouter du jitter (troupeau tonnant) — §14 (trou #7) — S
- [ ] **Shutdown drain timeout** — `worker.rs` : timeout de drain (un PDF à 2 h bloque l'arrêt) — §10.6 — M
- [ ] **`audit_logs` partitions** — planifier `create_next_audit_log_partition` (pg_cron/scheduler) sinon les INSERT casseront après la dernière partition — §14 #45 — M
- [ ] **Audit channel** — `logger.rs:36` : borner le channel + flush au shutdown (unbounded + drop-on-error aujourd'hui) — §9 — M
- [ ] **Layer OTEL avant `env_filter`** — `subscriber.rs:121` : monter après le filtre (échappe à `RUST_LOG`) — §14 #46 — S
- [ ] **`last_accessed` FIFO** — `tenant_manager.rs:255` : rafraîchir sur hit (l'éviction « LRU » est FIFO) — §14 #27 — S

### Cohérence retrieval / merge
- [ ] **Doc « weighted blend » vs code max** — `modes/mix.rs:216` : aligner doc et code (le code fait un max, pas une somme) — §14 #35 — S
- [ ] **`query_vec` = embed(historique+question)** réutilisé comme question seule — `query_pipeline.rs:465` : embedder la question nue pour le slot query — §14 #38 — M
- [ ] **`chunk.score` 3 échelles** (cosinus/RRF/rerank) — normaliser ou documenter par mode — §14 #37 — M
- [ ] **`min_score` sauté** quand `preserve_order` — `chunk_retrieval.rs:159` : décider si voulu, sinon garder le filtre — §14 #39 — S
- [ ] **Poids relation `(a+b)/2`** non associatif — `relationship.rs` : choisir somme ou vraie moyenne — §14 #31 — S
- [ ] **Types d'entité : premier gagne sans log** — `update_entity_node` : logguer/compter les conflits de type — §14 #32 — S
- [ ] **`QueryStreamStats`** sans diagnostics de bras — `query_types.rs:330` : aligner sur `QueryStats` — §14 #40 — S
- [ ] **Progression = moyenne non pondérée** — `progress.rs:548` : pondérer les 6 phases ; `avg_item_time_ms` non `#[serde(skip)]` (ETA repart de zéro) — §14 #41/#42 — M

### Tokens / pipeline
- [ ] **Trois estimateurs de tokens** (2.5/4/4) — unifier sur un tokenizer réel (tiktoken) dans le pipeline — §14 #53 — M
- [ ] **Cache d'extraction inerte** — `pipeline/cache.rs:358` : le brancher (jamais de `set`, 100 % miss) OU le supprimer — §14 #52 — S

### Nettoyage (code mort — cf. §13.4)
- [ ] Supprimer `pipeline/validation.rs` + `sanitizer.rs` (~1000 l., zéro call site) — S
- [ ] Supprimer `SOTAExtractor` + prompts tuple (non câblés) OU les câbler — M
- [ ] Supprimer `crates/edgequake-llm/` (CHANGELOG fantôme) + les 9 CHANGELOG orphelins de `crates/` racine — S
- [ ] Supprimer `age_csv_loader.rs`, `test_docker_e2e.py`, `init.sql` (orphelins) — S
- [ ] `MergerConfig.description_decay`/`min_importance`, `MAX_SOURCE_IDS` (300), `default_recursive_separators`, `append_description_history` : morts — supprimer — S

### Build / CI / SDK
- [ ] **Workflows SDK imbriqués** — `sdks/*/.github/workflows/` jamais exécutés : déplacer à la racine ou documenter le mirroring — §14 #48 — M
- [ ] **`sed -i ''`** (BSD) dans le Makefile — cassé sur Linux CI — §14 #49 — S
- [ ] **`make postgres-start` inexistant** mais recommandé par CONTRIBUTING/AGENTS : créer l'alias ou corriger la doc — §14 #47 — S
- [ ] **`schema.d.ts` (596 Ko) orphelin** — brancher le client webui dessus OU arrêter le codegen — §9.8 — M
- [ ] **Deux serveurs MCP** — déclarer le TS déprécié OU documenter le partage des rôles — §14 #55 — S

---

## Suivi

| Sprint | Items | Effort cumulé indicatif |
|---|---:|---|
| 🔥 S0 hotfix prod | 14 | ~1 semaine (ops + 4 correctifs P0) |
| 🔴 S1 sécurité | 13 | ~2-3 semaines |
| 🟠 S2 correction | 12 | ~2 semaines |
| 🟢 S3 dette | 24 | continu / au fil de l'eau |

> **Ne pas commencer S1+ avant que S0 (hotfix prod) soit vert.** L'incident SPEC-062 bloque le chat et l'ingestion en production.
