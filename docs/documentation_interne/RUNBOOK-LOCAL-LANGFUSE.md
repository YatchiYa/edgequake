# EdgeQuake v0.26.1 — Démarrage local avec Langfuse

> Révision 2.0 — 2026-08-27. Chaîne de traçage **et** suivi des coûts validés de bout
> en bout sur Langfuse **3.1.1** et **4**.

## Démarrage

```bash
make dev-bg-langfuse       # stack complète + Langfuse v4 local (clés injectées)
make langfuse-sync-prices  # tarifs des modèles → sans cela, coûts à $0.00
make status                # état des services
make stop                  # arrêt
```

| Service | URL | Notes |
|---|---|---|
| Web UI | http://localhost:3010 | |
| API | http://localhost:8090 | Swagger : `/swagger-ui` |
| Langfuse | http://localhost:3310 | `dev@example.com` / `edgequake-local-dev` |
| PostgreSQL | conteneur `edgequake-postgres` | pg18 + pgvector + AGE |

Les ports viennent de `scripts/select_edgequake_port.py` (8090/3010), **pas** de
`EDGEQUAKE_PORT` du `.env`.

## Les deux transports Langfuse

EdgeQuake exporte vers **toutes les versions de Langfuse ≥ 2.x**. Le transport est
choisi au démarrage :

| Version Langfuse | Endpoint OTLP | Transport retenu |
|---|---|---|
| ≥ 3.22x, 4.x | présent | **OTLP/HTTP** |
| ≤ 3.1 | **404** | **API d'ingestion native** |

```bash
EDGEQUAKE_LANGFUSE_API=auto        # défaut : sonde puis choisit
                      =otlp        # force OTLP
                      =ingestion   # force l'API native
```

Vérifier le choix effectué :
```bash
grep 'Langfuse API auto-detected' /tmp/edgequake-backend.log
# → Langfuse API auto-detected → Otlp   (ou Ingestion)
```

> Seul un **404** déclenche le repli. Un 401 (mauvaises clés) ou une panne réseau
> conserve OTLP : une erreur d'authentification ne doit pas changer silencieusement de
> transport et masquer le vrai problème.

## Suivi des coûts

Langfuse calcule les coûts depuis **son propre catalogue** ; EdgeQuake n'émet jamais
d'attribut de coût (`LAW-124-12` — Langfuse reste la source unique de vérité). Le
catalogue livré avec Langfuse 3.1 datant de 2024, tout modèle récent affiche **$0.00**.

```bash
make langfuse-sync-prices              # pousse models.toml → Langfuse
make langfuse-sync-prices DRY_RUN=1    # aperçu sans écrire
make langfuse-sync-prices FORCE=1      # réécrit les modèles existants
```

Mesuré après synchronisation (catalogue 88 → 133 modèles) :

| Modèle | Tokens | Coût |
|---|---|---|
| `gpt-5.4` | 10 000 / 1 000 | $0.0400 |
| `claude-sonnet-4-6` | 10 000 / 1 000 | $0.0450 |
| `gemini-2.5-flash` | 10 000 / 1 000 | $0.0021 |
| `mistral-small-latest` | 10 000 / 1 000 | $0.0026 |

**À relancer** après tout ajout de modèle dans `models.toml`.

**Limite connue** — sans tokens, pas de coût possible : `query.embed` (l'API
d'embedding ne remonte pas de décompte) et `generate-answer` via `/query/stream`
(la branche `llm.stream` ne renseigne pas l'usage, contrairement à `llm.chat`).

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

### 4. `make stop` supprime le conteneur PostgreSQL

Si un **PostgreSQL hôte** occupe déjà le port 5432, le Makefile le détecte au
redémarrage (« PostgreSQL already reachable ») et l'utilise **à la place** de celui
d'EdgeQuake — d'où des `permission denied for schema public` au `migrate`.

Les données ne sont pas perdues : elles vivent dans le volume
`edgequake-dev_postgres-data-pg18`. Correctif — déplacer EdgeQuake sur un port libre :
```bash
# .env
POSTGRES_PORT=5433
DATABASE_URL=postgresql://edgequake:edgequake_secret@localhost:5433/edgequake?options=-c%20search_path%3Dpublic
```
Vérifier : `docker volume ls | grep edgequake` puis
`psql -h localhost -p 5433 -U edgequake -d edgequake -c 'SELECT count(*) FROM conversations;'`

### 5. Deux faux positifs à ne PAS diagnostiquer comme des pannes
- **`HttpTraceClient.ResponseParseError: invalid wire type value: 6`** dans les logs
  backend : cosmétique. Langfuse répond en JSON là où le client Rust attend du
  protobuf. La livraison réussit (HTTP 200).
- **`GET /api/public/traces` renvoie 0 en v4** : API *legacy*. v4 stocke dans
  `events_core`, **v3 dans `observations`** (et son API `/traces` fonctionne).
  Utiliser l'UI ou :
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

**Sur Langfuse 3.x** (table `observations`) :
```bash
docker exec eq-langfuse3-clickhouse-1 clickhouse-client \
  --user clickhouse --password clickhouse \
  -q "SELECT name, type, provided_model_name, toString(usage_details), total_cost \
      FROM default.observations ORDER BY start_time DESC LIMIT 10 FORMAT PrettyCompact"
```
`total_cost` à `NULL` sur une génération avec des tokens ⇒ modèle absent du catalogue
Langfuse → `make langfuse-sync-prices`.

## Fournisseur LLM

État constaté le 2026-08-26 (à revérifier) : la clé **OpenAI du `.env` était invalide** (401 `invalid_api_key`,
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

---

# Annexe — Langfuse en Kubernetes (pods séparés)

> Version complète et à jour : **[04-langfuse-kubernetes.md](04-langfuse-kubernetes.md)**
> (compatibilité par version, suivi des coûts, manifests, validation en 9 points).

## Le piège n°1 : repli silencieux vers Langfuse Cloud

`edgequake-observability/src/langfuse.rs:9`
```rust
pub const DEFAULT_LANGFUSE_BASE_URL: &str = "https://cloud.langfuse.com";
```

La résolution est : `LANGFUSE_BASE_URL` → sinon `LANGFUSE_HOST` → sinon **`https://cloud.langfuse.com`**.
Chaque variable est filtrée par `.filter(|v| !v.is_empty())` : une valeur **vide**
équivaut à une valeur **absente**.

Conséquence en Kubernetes : si la variable est absente, vide, ou injectée depuis une
clé de ConfigMap/Secret inexistante, EdgeQuake exporte vers **Langfuse Cloud** au lieu
du pod interne — sans aucune erreur visible.

Et l'activation ne dépend **que des clés** (`enabled = keys_ok`), pas de l'URL :
`export_active: true` ne prouve donc **pas** que l'on pointe sur le bon Langfuse.

**Toujours vérifier `base_url`, pas seulement `export_active` :**
```bash
kubectl exec -n <ns> deploy/edgequake -- \
  curl -s localhost:8080/api/v1/settings/langfuse | jq '{base_url, export_active, public_key_configured, secret_key_configured}'
```
Si `base_url` vaut `https://cloud.langfuse.com` → la variable n'est pas injectée.

## Le piège n°2 : `localhost`

`LANGFUSE_BASE_URL=http://localhost:3310` fonctionne en local mais **jamais** entre
pods : `localhost` désigne le pod EdgeQuake lui-même.

Valeur correcte (DNS de Service) :
```yaml
- name: LANGFUSE_BASE_URL
  value: "http://langfuse-web.<namespace>.svc.cluster.local:3000"
```
⚠️ Port **du Service** (souvent 3000), pas 3310 (mapping hôte local uniquement).
Pas de chemin ni de `/` final : le code ajoute `/api/public/otel/v1/traces`.

## Le piège n°3 : les clés ne sont pas transposables

Les clés locales (`pk-lf-edgequake-local` / `sk-lf-edgequake-local-dev`) proviennent du
`LANGFUSE_INIT_*` headless du compose. Votre Langfuse Kubernetes a **ses propres**
clés de projet — à créer dans son UI puis à injecter via un Secret. Réutiliser les
clés locales donne un **401 silencieux**.

## Le piège n°4 : les échecs d'export sont en DEBUG

Les erreurs d'export OTLP sont journalisées au niveau **DEBUG**. Avec `RUST_LOG=info`
(défaut production), un export qui échoue est **totalement invisible**.

Pour diagnostiquer :
```bash
kubectl set env deploy/edgequake -n <ns> RUST_LOG=info,opentelemetry_otlp=debug,opentelemetry_sdk=debug
kubectl logs -n <ns> deploy/edgequake --tail=100 | grep -i otlp
```
Rappel : `ResponseParseError: invalid wire type value: 6` est **normal** (Langfuse
répond en JSON, le client attend du protobuf) — la livraison réussit malgré tout.

## Le piège n°5 : course aux migrations Langfuse

Observé sur ce poste : le pod worker démarre avant la fin des migrations PostgreSQL.
```
relation "monitors" does not exist · public.batch_actions does not exist
```
Les requêtes OTLP renvoient **200** mais aucune trace n'est jamais créée.
En Kubernetes, les pods démarrant en parallèle, le risque est plus élevé qu'en compose.

**Vérifier :**
```bash
kubectl logs -n <ns> deploy/langfuse-worker | grep -ci "does not exist"   # doit valoir 0
```
**Corriger :** `kubectl rollout restart deploy/langfuse-web deploy/langfuse-worker -n <ns>`
**Prévenir :** initContainer attendant la fin des migrations, ou `readinessProbe` sur
le web avant démarrage du worker.

## Le piège n°6 : NetworkPolicy

Si des NetworkPolicies sont en place, autoriser explicitement l'egress
EdgeQuake → langfuse-web sur le port du Service. Test :
```bash
kubectl exec -n <ns> deploy/edgequake -- \
  curl -s -o /dev/null -w '%{http_code}\n' http://langfuse-web.<ns>.svc.cluster.local:3000/api/public/health
```
Attendu : **200**.

## Procédure de diagnostic ordonnée

| # | Contrôle | Commande | Attendu |
|---|---|---|---|
| 1 | Variables injectées | `kubectl exec deploy/edgequake -- env \| grep LANGFUSE` | 3 variables non vides |
| 2 | Cible réelle | `curl .../api/v1/settings/langfuse \| jq .base_url` | l'URL **interne**, pas cloud.langfuse.com |
| 3 | Joignabilité | `curl <svc>/api/public/health` depuis le pod EdgeQuake | 200 |
| 4 | Auth des clés | `curl -u pk:sk <svc>/api/public/projects` | le projet attendu |
| 5 | Migrations worker | `kubectl logs deploy/langfuse-worker \| grep -c "does not exist"` | 0 |
| 6 | Ingestion réelle | requête ClickHouse `events_core` (cf. §4 ci-dessus) | spans EdgeQuake présents |

Ne pas conclure sur `/api/public/traces` (API legacy, renvoie 0 en v4).
