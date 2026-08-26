# EdgeQuake v0.26.1 — Démarrage local avec Langfuse

> Validé le 2026-08-26 sur cette machine. Chaîne de traçage prouvée de bout en bout.

## Démarrage

```bash
make dev-bg-langfuse      # stack complète + Langfuse (clés locales injectées)
make status               # état des services
make stop                 # arrêt
```

| Service | URL | Notes |
|---|---|---|
| Web UI | http://localhost:3010 | |
| API | http://localhost:8090 | Swagger : `/swagger-ui` |
| Langfuse | http://localhost:3310 | `dev@example.com` / `edgequake-local-dev` |
| PostgreSQL | conteneur `edgequake-postgres` | pg18 + pgvector + AGE |

Les ports viennent de `scripts/select_edgequake_port.py` (8090/3010), **pas** de
`EDGEQUAKE_PORT` du `.env`.

## Points de vigilance découverts

### 1. Le Makefile lit le `.env` RACINE, pas `edgequake/.env`
`Makefile:228` → `-include $(ROOT_DIR)/.env`. Toute config dans `edgequake/.env`
est **ignorée** par `make dev*`. Elle ne sert qu'aux exécutions conteneur.

### 2. `EDGEQUAKE_MODELS_CONFIG` doit être un chemin HÔTE
`/app/models.toml` est un chemin conteneur : en local le fichier n'existe pas et le
binaire retombe **silencieusement** sur le catalogue compilé
(`bundled_models.rs`) — les éditions de `models.toml` seraient sans effet.
Valeur correcte : chemin absolu vers `edgequake/models.toml`.

### 3. Course au démarrage de Langfuse (migrations)
Au tout premier `langfuse-up`, le worker peut démarrer avant la fin des migrations
PostgreSQL :
```
relation "monitors" does not exist · public.batch_actions does not exist
```
Les jobs OTLP sont alors acceptés (200) mais jamais transformés en traces.
**Correctif :**
```bash
cd edgequake/docker && LANGFUSE_PORT=3310 NEXTAUTH_URL=http://localhost:3310 \
  docker compose -f docker-compose.langfuse.yml --project-name edgequake-langfuse \
  restart langfuse-web langfuse-worker
```
Vérifier ensuite : `docker logs edgequake-langfuse-langfuse-worker-1 | grep -c "does not exist"` → **0**.

### 4. Deux faux positifs à ne PAS diagnostiquer comme des pannes
- **`HttpTraceClient.ResponseParseError: invalid wire type value: 6`** dans les logs
  backend : cosmétique. Langfuse répond en JSON là où le client Rust attend du
  protobuf. La livraison réussit (HTTP 200).
- **`GET /api/public/traces` renvoie 0** : API *legacy*. Langfuse v4 stocke dans
  ClickHouse `events_core`. Utiliser l'UI ou :
  ```bash
  docker exec edgequake-langfuse-clickhouse-1 clickhouse-client \
    --user clickhouse --password clickhouse \
    -q "SELECT name, count() FROM default.events_core GROUP BY name ORDER BY 2 DESC"
  ```

## Vérification du traçage

```bash
curl -s http://localhost:8090/api/v1/settings/langfuse | jq .export_active   # true
make langfuse-smoke                                                          # ✓ passed
```

Spans attendus après une ingestion + une requête : `ingest.document`,
`ingest.chunking`, `pipeline_chunk_extraction`, `extract-entities-glean`,
`embed-chunks`, `query_pipeline`, `query.embed`, `retrieval edgequake`,
`query.fuse`, `query.rerank`, `generate-answer`.
Les spans LLM portent `type=GENERATION`, le modèle et les tokens.

## Fournisseur LLM

État au 2026-08-26 : la clé **OpenAI du `.env` est invalide** (401 `invalid_api_key`,
vérifié directement auprès d'OpenAI). Le `.env` racine est donc configuré sur
**Mistral** (clé valide).

Sauvegardes : `.env.openai-original` et `.env.backup-<horodatage>`.

**Retour sur OpenAI** (après avoir généré une clé valide) :
```bash
cp .env.openai-original .env
# remplacer OPENAI_API_KEY par la nouvelle clé, puis corriger EDGEQUAKE_MODELS_CONFIG
make stop && make dev-bg-langfuse
```

⚠️ Changer de fournisseur d'embeddings change la **dimension** des vecteurs
(mistral-embed 1024 ↔ text-embedding-3-small 1536). Sur un workspace contenant déjà
des documents, prévoir `POST /api/v1/workspaces/{ws}/rebuild-embeddings`.
