---
title: "EdgeQuake × Langfuse — Compatibilité, suivi des coûts et déploiement Kubernetes"
version: "2.0"
date: "2026-08-27"
produit: "EdgeQuake v0.26.1"
methode: "Tests empiriques sur Langfuse 3.1.1, 3.225.5 et 4 — résultats reproductibles"
---

# EdgeQuake × Langfuse — Compatibilité, coûts et Kubernetes

> **Conclusion en une phrase** : EdgeQuake sait exporter ses traces vers **toutes les
> versions de Langfuse ≥ 2.x**, y compris **3.1**. Le transport est choisi
> automatiquement au démarrage : OTLP quand l'endpoint existe, **API d'ingestion
> native** sinon. Aucune montée de version n'est requise.

> **Révision 2.0 — ce qui a changé.** La version 1.0 de ce document concluait que
> Langfuse 3.1 était **incompatible** et qu'une montée en 3.225+ était obligatoire.
> C'était exact à l'époque : EdgeQuake n'exportait qu'en OTLP. Depuis, un
> **exportateur natif** a été ajouté (`langfuse_ingestion.rs`) et la contrainte a
> disparu. Les sections traitant de la montée de version sont conservées à titre
> d'option, plus d'obligation.

---

## 1. Compatibilité par version

| Version Langfuse | `/api/public/otel/v1/traces` | `/api/public/ingestion` | Transport retenu | Traces |
|---|---|---|---|---|
| **3.1.1** | **404 — absent** | ✅ 207 | **Ingestion** (auto) | ✅ **validé** |
| **3.225.5** | ✅ 200 | ✅ | OTLP (auto) | ✅ validé |
| **4.x** | ✅ 200 | ✅ | OTLP (auto) | ✅ validé |

L'endpoint OTLP n'existe qu'à partir de Langfuse **3.22x**. L'API d'ingestion native,
elle, est présente **depuis la v2** — c'est le socle commun sur lequel s'appuie le
repli.

### Sélection du transport

```bash
EDGEQUAKE_LANGFUSE_API=auto        # défaut : sonde puis choisit
                      =otlp        # force OTLP
                      =ingestion   # force l'API native
```

En `auto`, EdgeQuake envoie une requête de sonde sur l'endpoint OTLP au démarrage :

- **404** → l'instance précède le support OTLP → bascule sur l'ingestion native
- **toute autre réponse** (200, 401, 405) ou échec réseau → **conserve OTLP**

> **Garde-fou volontaire** : seul un 404 déclenche la bascule. Une erreur
> d'authentification ou une panne réseau ne doit pas changer silencieusement de
> transport et masquer le vrai problème.

Le transport retenu est journalisé au démarrage :
```
Langfuse API auto-detected → Ingestion
Langfuse native ingestion exporter enabled → http://…/api/public/ingestion
```

### Le code concerné

`edgequake/crates/edgequake-observability/src/langfuse.rs`
```rust
pub enum LangfuseApi { Otlp, Ingestion, Auto }
pub fn probe_langfuse_api(base_url: &str, auth_token: &str) -> LangfuseApi
```
`edgequake/crates/edgequake-observability/src/langfuse_ingestion.rs` — exportateur
`SpanExporter` traduisant les spans OTel en événements `trace-create` /
`generation-create` / `span-create`.

---

## 2. Pourquoi le diagnostic est trompeur

Trois signaux donnent l'illusion d'un fonctionnement normal :

| Signal observé | Réalité |
|---|---|
| `GET /api/v1/settings/langfuse` → `export_active: true` | L'activation ne dépend **que des clés** (`enabled = keys_ok`) — jamais de la joignabilité ni de la validité de l'URL |
| `GET /api/public/health` → 200 | Langfuse **est** joignable — mais l'endpoint OTLP, lui, n'existe pas |
| Aucune erreur dans les journaux | Les échecs d'export sont journalisés en **DEBUG**. Avec `RUST_LOG=info` (défaut production), le 404 est **invisible** |

> **À retenir pour l'exploitation** : `export_active: true` signifie « des clés sont
> configurées », **pas** « les traces arrivent ». Le seul contrôle probant est de
> compter les spans côté Langfuse (§6).

---

## 3. Piège complémentaire : le repli silencieux vers Langfuse Cloud

`langfuse.rs:9`
```rust
pub const DEFAULT_LANGFUSE_BASE_URL: &str = "https://cloud.langfuse.com";
```

Ordre de résolution : `LANGFUSE_BASE_URL` → `LANGFUSE_HOST` → **Langfuse Cloud**.
Chaque variable passe par `.filter(|v| !v.is_empty())` : **une valeur vide équivaut à
une valeur absente**.

En Kubernetes, une variable référençant une clé de ConfigMap/Secret inexistante est
injectée comme **chaîne vide** → EdgeQuake exporte vers `cloud.langfuse.com`, avec des
clés internes invalides, **sans erreur visible**.

> ⚠️ **Implication sécurité** : dans cette situation, le contenu des traces (prompts,
> extraits de documents, réponses) est émis vers un service **externe**. À vérifier
> impérativement avant toute mise en production sur données sensibles.

**Contrôle obligatoire :**
```bash
kubectl exec -n <ns> deploy/edgequake -- \
  curl -s localhost:8080/api/v1/settings/langfuse | jq .base_url
```
Toute valeur autre que l'URL interne attendue est une anomalie bloquante.

---

## 4. Suivi des coûts

### 4.1 Pourquoi les coûts s'affichent à $0.00

Langfuse calcule le coût d'une observation à partir de **son propre catalogue de
modèles**, et EdgeQuake **n'émet jamais** d'attribut de coût — c'est une règle
explicite du code :

```rust
/// LAW-124-12: never emit these attribute keys (Langfuse cost ingestion).
COST_ATTR_DENYLIST = ["gen_ai.usage.cost", "langfuse.observation.cost_details", …]
```

Langfuse reste ainsi la **source unique de vérité** pour les coûts. La conséquence :
une instance auto-hébergée ne sait tarifer que les modèles présents dans le catalogue
livré avec sa version. Langfuse **3.1 embarque une liste datant de 2024** :

| Fournisseur | Présent en 3.1 | Absent |
|---|---|---|
| OpenAI | `gpt-4o`, `gpt-4`, `gpt-3.5` | `gpt-4.1`, `gpt-5.x` |
| Google | `gemini-1.5-*` | `gemini-2.5-*` |
| Anthropic | `claude-3.5-*` | `claude-sonnet-4-x`, `claude-opus-4-x` |
| Mistral | *(aucun)* | tous |
| xAI, MiniMax | *(aucun)* | tous |

Tout modèle récent affiche donc **$0.00**, quels que soient les tokens remontés.

### 4.2 Correctif : synchroniser les tarifs depuis `models.toml`

`models.toml` contient déjà les tarifs de tous les fournisseurs supportés. Un script
les pousse dans Langfuse via `POST /api/public/models` — **sans toucher au chemin
d'export ni contourner LAW-124-12**.

```bash
make langfuse-sync-prices              # synchronise
make langfuse-sync-prices DRY_RUN=1    # aperçu, n'écrit rien
make langfuse-sync-prices FORCE=1      # réécrit les modèles déjà présents
```

Équivalent direct (utile en Kubernetes, hors dépôt) :
```bash
LANGFUSE_BASE_URL=https://langfuse.intra.example \
LANGFUSE_PUBLIC_KEY=pk-lf-… LANGFUSE_SECRET_KEY=sk-lf-… \
python3 scripts/langfuse_sync_model_prices.py
```

**Comportement** : conversion per-1k → per-token (Langfuse tarife au token), modèles
d'embedding tarifés côté entrée uniquement, modèles locaux gratuits (Ollama,
LM Studio) ignorés, idempotent — les modèles déjà connus sont sautés.

**Résultat mesuré** — catalogue 88 → 133 modèles, puis :

| Modèle | Tokens | Coût calculé |
|---|---|---|
| `gpt-5.4` | 10 000 / 1 000 | **$0.0400** |
| `claude-sonnet-4-6` | 10 000 / 1 000 | **$0.0450** |
| `gemini-2.5-flash` | 10 000 / 1 000 | **$0.0021** |
| `mistral-small-latest` | 10 000 / 1 000 | **$0.0026** |

> **À faire une fois par instance Langfuse**, et à rejouer après chaque ajout de
> modèle dans `models.toml`.

### 4.3 Limite connue : observations sans tokens

Sans décompte de tokens, aucun coût n'est calculable, quel que soit le catalogue.
Deux cas subsistent :

| Observation | Cause | État |
|---|---|---|
| `query.embed` | l'API d'embedding ne remonte pas de décompte | Non résolu — un chiffre estimé serait trompeur dans un tableau de coûts |
| `generate-answer` **via `/query/stream`** | la branche `llm.stream(&prompt)` ne renseigne pas l'usage, contrairement à `llm.chat()` | Non résolu |

Le correctif propre passerait par `chat_with_tools_stream`, qui porte à la fois les
rôles system/user **et** un `Done { usage }` — il résoudrait aussi le compromis
streaming/fidélité de prompt. C'est une réécriture du chemin de génération, non
engagée à ce jour.

### 4.4 Repli : désactiver l'export

```yaml
- name: EDGEQUAKE_LANGFUSE_ENABLED
  value: "0"          # coupe l'export quelles que soient les clés
```
Le reste de l'observabilité (métriques Prometheus `/metrics`, journaux structurés,
`retrieval_id` rejouable via `/api/v1/query/context/{id}`) demeure **pleinement
opérationnel**.

### 4.5 Montée de version Langfuse — désormais optionnelle

La montée en 3.225+ n'est **plus nécessaire** pour le traçage. Elle reste pertinente
pour bénéficier des correctifs amont et d'un catalogue de modèles plus récent.

```yaml
image: langfuse/langfuse:3.225.5          # web
image: langfuse/langfuse-worker:3.225.5   # worker
```
> Épingler une version exacte, jamais `:3` ni `:latest`.

Séquence : sauvegarde PostgreSQL + ClickHouse → worker à 0 → montée du web (il applique
les migrations) → `GET /api/public/health` 200 → remontée du worker → validation §6.

---

## 5. Configuration Kubernetes de référence

### 5.1 Secret et variables EdgeQuake

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: edgequake-langfuse
  namespace: <ns>
type: Opaque
stringData:
  LANGFUSE_PUBLIC_KEY: "pk-lf-..."   # clés DU projet de CE Langfuse
  LANGFUSE_SECRET_KEY: "sk-lf-..."
---
# Deployment EdgeQuake — conteneur api
env:
  # URL interne — surtout PAS localhost, surtout PAS de valeur vide
  - name: LANGFUSE_BASE_URL
    value: "http://langfuse-web.<ns>.svc.cluster.local:3000"
  - name: LANGFUSE_PROJECT_ID
    value: "<project-id>"            # deep-links UI
  - name: EDGEQUAKE_LANGFUSE_ENABLED
    value: "1"
  - name: LANGFUSE_PUBLIC_KEY
    valueFrom: {secretKeyRef: {name: edgequake-langfuse, key: LANGFUSE_PUBLIC_KEY}}
  - name: LANGFUSE_SECRET_KEY
    valueFrom: {secretKeyRef: {name: edgequake-langfuse, key: LANGFUSE_SECRET_KEY}}
```

**Cinq règles impératives :**

1. **Jamais `localhost`** — dans un pod, `localhost` désigne le pod lui-même.
2. **Port du Service** (typiquement 3000), pas le port d'un mapping hôte local.
3. **Pas de chemin ni de `/` final** — le code ajoute lui-même le chemin du transport retenu (`/api/public/otel/v1/traces` ou `/api/public/ingestion`).
4. **Jamais de valeur vide** — équivaut à « non défini » → repli vers Langfuse Cloud (§3).
5. **Clés du projet de CETTE instance** — les clés d'une autre instance donnent un 401 silencieux.

### 5.2 Les clés ne sont pas transposables

Une paire `pk-lf-…` / `sk-lf-…` n'est valide que dans l'instance Langfuse qui l'a
émise. Créer le projet dans le Langfuse du client, récupérer **ses** clés, les injecter
via Secret.

Vérification :
```bash
kubectl exec -n <ns> deploy/edgequake -- \
  curl -s -u "$LANGFUSE_PUBLIC_KEY:$LANGFUSE_SECRET_KEY" \
  http://langfuse-web.<ns>.svc.cluster.local:3000/api/public/projects
```
Doit renvoyer le projet attendu.

### 5.3 Course aux migrations — reproduite sur 3.1 **et** 4

Sur les deux versions, le **worker démarre avant la fin des migrations** appliquées par
le web. Symptômes :
```
relation "monitors" does not exist
public.batch_actions does not exist
```
Les requêtes OTLP renvoient alors **200** mais **aucune trace n'est jamais créée** —
le plus trompeur des modes de défaillance.

En Kubernetes le risque est **supérieur** au compose : les pods démarrent en parallèle.

**Prévention :**
```yaml
# Deployment langfuse-worker
initContainers:
  - name: wait-for-web-migrations
    image: curlimages/curl:8.10.1
    command: ['sh','-c','until curl -sf http://langfuse-web.<ns>.svc.cluster.local:3000/api/public/health; do sleep 5; done']
```
**Détection :**
```bash
kubectl logs -n <ns> deploy/langfuse-worker | grep -ci "does not exist"   # attendu : 0
```
**Correction :** `kubectl rollout restart deploy/langfuse-web deploy/langfuse-worker -n <ns>`

### 5.4 Piège YAML — `ENCRYPTION_KEY`

Rencontré lors de nos tests : une clé composée uniquement de chiffres, **non quotée**,
est interprétée par YAML comme un **nombre**.
```yaml
ENCRYPTION_KEY: 0000000000000000000000000000000000000000000000000000000000000000   # ❌ → "0"
ENCRYPTION_KEY: "0000000000000000000000000000000000000000000000000000000000000000" # ✅
```
Langfuse refuse alors de démarrer :
`ENCRYPTION_KEY must be 256 bits, 64 string characters in hex format`.
Toujours **quoter** les valeurs numériques ou hexadécimales dans ConfigMaps et
manifests.

### 5.5 NetworkPolicy

Autoriser explicitement l'egress EdgeQuake → langfuse-web sur le port du Service :
```bash
kubectl exec -n <ns> deploy/edgequake -- \
  curl -s -o /dev/null -w '%{http_code}\n' \
  http://langfuse-web.<ns>.svc.cluster.local:3000/api/public/health   # attendu : 200
```

---

## 6. Procédure de validation — dans l'ordre

Ne pas passer à l'étape suivante tant que la précédente échoue.

| # | Contrôle | Commande | Attendu |
|---|---|---|---|
| 1 | **Version Langfuse** | `curl <svc>/api/public/health` | toute version ≥ 2.x — informatif seulement |
| 2 | **Transport retenu** | `kubectl logs deploy/edgequake \| grep 'Langfuse API auto-detected'` | `Otlp` ou `Ingestion` — un **404** sur l'endpoint OTLP est normal en 3.1 et déclenche le repli |
| 3 | Variables injectées | `kubectl exec deploy/edgequake -- env \| grep LANGFUSE` | 3 variables **non vides** |
| 4 | **Cible réelle** | `curl .../api/v1/settings/langfuse \| jq .base_url` | URL interne (**pas** cloud.langfuse.com) |
| 5 | Joignabilité | `curl <svc>/api/public/health` depuis le pod EdgeQuake | 200 |
| 6 | Validité des clés | `curl -u pk:sk <svc>/api/public/projects` | le projet attendu |
| 7 | Migrations worker | `kubectl logs deploy/langfuse-worker \| grep -c "does not exist"` | **0** |
| 8 | **Traces réellement ingérées** | requête ClickHouse ci-dessous | spans EdgeQuake présents |

**Étape 8 — le seul contrôle probant.** Générer une requête RAG, puis :
```bash
kubectl exec -n <ns> <clickhouse-pod> -- clickhouse-client \
  --user <u> --password <p> \
  -q "SELECT name, count() FROM default.events_core GROUP BY name ORDER BY 2 DESC LIMIT 20"
```

Spans attendus après une ingestion et une requête :
```
ingest.document · ingest.chunking · pipeline_chunk_extraction
extract-entities-glean · embed-chunks · ingest.persist
query_pipeline · query.embed · retrieval edgequake
query.fuse · query.rerank · extract-keywords · generate-answer
```
Les spans LLM portent `type=GENERATION`, le modèle et le décompte de tokens.

**Selon la majeure, la table et l'API diffèrent :**

| Majeure | Table ClickHouse | `GET /api/public/traces` |
|---|---|---|
| **3.x** | `observations` | ✅ **exploitable** — renvoie les traces |
| **4.x** | `events_core` / `events_full` | ❌ renvoie **0** même quand tout fonctionne |

En **3.x** (cas du client), le contrôle le plus simple est donc :
```bash
kubectl exec -n <ns> deploy/edgequake -- \
  curl -s -u "$LANGFUSE_PUBLIC_KEY:$LANGFUSE_SECRET_KEY" \
  "http://langfuse-web.<ns>.svc.cluster.local:3000/api/public/traces?limit=5"
```
Une liste non vide après une requête RAG prouve la chaîne complète.

**Étape 9 — coûts.** Après `make langfuse-sync-prices`, une génération doit porter un
coût non nul :
```bash
curl -s -u "$LANGFUSE_PUBLIC_KEY:$LANGFUSE_SECRET_KEY" \
  "http://langfuse-web.<ns>.svc.cluster.local:3000/api/public/observations?limit=5" \
  | jq '.data[] | {name, model, usage, calculatedTotalCost}'
```
`calculatedTotalCost: null` sur une génération avec des tokens ⇒ le modèle manque au
catalogue Langfuse (§4).

> ⚠️ Après une éventuelle montée en v4, ce contrôle cesse d'être valable : basculer
> sur la requête ClickHouse `events_core`.

---

## 7. Faux positifs à ne pas traiter comme des pannes

| Observation | Explication | Action |
|---|---|---|
| `HttpTraceClient.ResponseParseError: invalid wire type value: 6` | Langfuse répond en **JSON**, le client Rust attend du **protobuf**. Erreur de lecture de la réponse — la **livraison a réussi** (HTTP 200) | Aucune |
| `GET /api/public/traces` renvoie 0 **en v4** | API legacy — en v4 les données sont dans `events_core` (en **v3 elle fonctionne**) | Utiliser ClickHouse ou l'UI |
| `export_active: true` sans trace | N'atteste que de la présence des clés | Dérouler §6 |
| Coût **$0.00** sur une génération | Modèle absent du catalogue Langfuse — EdgeQuake n'émet jamais le coût (LAW-124-12) | `make langfuse-sync-prices` (§4) |
| Coût **$0.00** sur `query.embed` | Aucun décompte de tokens remonté par l'API d'embedding | Limite connue (§4.3) |

---

## 8. Synthèse pour le client

1. **Traçage** : aucune montée de Langfuse n'est requise. EdgeQuake détecte
   l'absence d'endpoint OTLP (404) et bascule seul sur l'API d'ingestion native,
   présente depuis Langfuse v2. **Rien à configurer.**
2. **Coûts** : lancer **une fois** `make langfuse-sync-prices` (ou le script
   équivalent) sur l'instance Langfuse — sans quoi tout modèle récent affiche $0.00,
   le catalogue de 3.1 datant de 2024.
3. **Contrôle préalable indispensable** : vérifier que `base_url` ne pointe pas vers
   `cloud.langfuse.com` (repli silencieux avec risque d'émission de données hors du SI).
4. **Point de vigilance déploiement** : ordonner web → worker (course aux migrations),
   quoter les valeurs hexadécimales en YAML.
5. **Repli** : `EDGEQUAKE_LANGFUSE_ENABLED=0` — le reste de l'observabilité demeure
   opérationnel.

---

*Révision 2.0 du 2026-08-27, à partir de tests exécutés sur Langfuse 3.1.1, 3.225.5
et 4, avec EdgeQuake v0.26.1. Documents liés :
[Déploiement technique](01-deploiement-technique.md) ·
[Intégration IT](02-integration-it.md).*
