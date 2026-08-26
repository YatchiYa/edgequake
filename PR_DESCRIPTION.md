# fix(streaming, conversations): restore SSE token streaming + conversation history, add internal ops documentation

## Résumé

Deux correctifs de bugs reproduits et vérifiés, plus un dossier de documentation
technique interne rédigé à partir du code v0.26.1.

Les deux bugs partagent un même profil : **le symptôme est invisible en `curl` et ne
se reproduit que dans un vrai navigateur**, ce qui explique qu'ils aient pu passer.

---

## 1. Correctif — le streaming SSE n'arrivait jamais dans le navigateur

**Symptôme** : l'UI de requête affichait « Processing your query… » pendant toute la
génération (15 s), puis la réponse complète d'un coup. Aucun token progressif.

**Cause racine** : Next.js applique `compress: true` par défaut. Le proxy de dev
(`/api/:path*` → backend) ajoutait donc `Content-Encoding: gzip` sur une réponse
`text/event-stream`. **L'encodeur gzip tamponne tout le corps** avant de le flusher,
ce qui détruit mécaniquement le streaming.

L'API Axum, elle, ne compresse pas le SSE — vérifié : aucun `content-encoding` sur
`:8090`, `Content-Encoding: gzip` ajouté par Next sur `:3010`.

**Mesure dans un vrai Chrome** (Playwright, lecture `ReadableStream`) :

| | chunks | premiers événements | encodage |
|---|---|---|---|
| Avant | **1** | `12393ms / 101043 B` | `gzip` |
| Après | **469** | `131ms · 1350ms · 1963ms · 1976ms · 2005ms…` | *(aucun)* |

**Correctif** : `compress: false` dans `edgequake_webui/next.config.ts`, commenté avec
la preuve.

> ⚠️ **Portée production** : nginx / Traefik / les Ingress Kubernetes compressent aussi
> par défaut. Il faut y **exclure explicitement `text/event-stream`**, sinon le
> streaming recasse à l'identique. Et le diagnostic est piégeux : **`curl` ne reproduit
> pas le bug** (il ne négocie pas la compression par défaut).

---

## 2. Correctif — l'historique des conversations était toujours vide

**Symptôme** : « No conversations yet » alors que les conversations étaient bien créées
(le flux SSE renvoie un `conversation_id`).

**Cause racine — asymétrie d'identité entre écriture et lecture.** En mode anonyme
(authentification désactivée), la politique est `UseSharedGuest` : les chemins
d'**écriture** passent par `ensure_postgres_user_exists`, qui **ignore délibérément**
l'identifiant fourni par le client (`// per-browser mint removed`) et renvoie un
identifiant invité partagé dérivé du tenant. Les chemins de **lecture**, eux,
filtraient sur l'en-tête brut `x-user-id`.

```
Écriture  → user_id 825175bc-2c0c-572f-…   (invité partagé, dérivé)
Lecture   → user_id aa495d59-9b52-4a6b-…   (en-tête navigateur brut)
```

Asymétrie mesurée : **3 sites d'écriture** dérivaient l'identifiant, **8 sites de
lecture** ne le faisaient pas.

**Correctif** : les 7 sites de lecture (`crud.rs`, `folders.rs`, `bulk.rs`) passent
désormais par `ensure_postgres_user_exists`, comme les écritures. Une seule source de
vérité : la symétrie est garantie par construction et un futur changement de politique
d'identité s'appliquera partout. Chaque site porte un commentaire expliquant le motif.

**Vérification** : `total = 0` → **`total = 21`** conversations restituées.

> **Question produit ouverte** (hors périmètre de ce correctif) : en mode anonyme, tous
> les visiteurs partagent le même identifiant invité, donc les mêmes conversations.
> Le correctif rétablit la cohérence lecture/écriture ; il ne tranche pas si ce partage
> est le comportement souhaité.

---

## 3. Documentation technique interne

`docs/documentation_interne/` — 5 fichiers, anonymisés (`{client}`, domaines
génériques), rédigés à partir du code v0.26.1 avec références `fichier:ligne`.

| Fichier | Objet |
|---|---|
| `README.md` | Page de garde, contrôle documentaire, historique des révisions |
| `01-deploiement-technique.md` | Architecture, composants, prérequis, flux de données, réseau, sécurité, activation de l'authentification, installation, recette en 15 points |
| `02-integration-it.md` | Exploitation, monitoring, sauvegarde/restauration, mise à jour, rollback, runbooks d'incident, checklists |
| `03-deep-dive-architecture-algorithme.md` | Crates, algorithme d'ingestion, modèle de données, moteur d'interrogation, 8 décisions d'architecture |
| `04-langfuse-kubernetes.md` | Compatibilité de version Langfuse, déploiement en pods séparés |

`RUNBOOK-LOCAL-LANGFUSE.md` — démarrage local (`make dev-bg-langfuse`), pièges
rencontrés, vérification du traçage, annexe Kubernetes.

### Constats établis par test (pas par lecture de la documentation existante)

- **Langfuse < 3.22x est incompatible** : `/api/public/otel/v1/traces` renvoie **404 en
  3.1**, répond en 3.225.5 et en 4. Chemin codé en dur (`langfuse.rs:105`), sans repli.
- **Repli silencieux vers Langfuse Cloud** : `DEFAULT_LANGFUSE_BASE_URL =
  "https://cloud.langfuse.com"` (`langfuse.rs:9`) ; les variables vides sont filtrées
  comme absentes. Une `LANGFUSE_BASE_URL` vide (clé de ConfigMap manquante) fait sortir
  les traces du réseau interne, sans erreur.
- **`export_active: true` n'atteste que de la présence des clés**, jamais de l'arrivée
  des traces (`enabled = keys_ok`). Les échecs d'export sont en `DEBUG`, donc invisibles
  avec `RUST_LOG=info`.
- **Course aux migrations Langfuse**, reproduite en 3.1 et en 4 : le worker démarre
  avant la fin des migrations ; l'OTLP répond 200 mais aucune trace n'est créée.
- **`EDGEQUAKE_TASK_DELIVERY`** : toute valeur non reconnue retombe silencieusement sur
  `local` (mono-processus) — piège en multi-réplique.
- **`GET /api/public/traces`** renvoie 0 en Langfuse v4 même quand tout fonctionne
  (données dans `events_core`) ; l'API reste exploitable en v3.

---

## 4. `.gitignore`

Ajoute `.env.*` avec négations `!.env.example` / `!.env.*.example`.
**Motif** : les motifs existants `.env` et `*.env` ne couvrent **pas** `.env.backup-…`
ni `.env.<suffixe>`. Un fichier de sauvegarde local contenant des secrets pouvait donc
être committé par inadvertance — constaté sur cette branche, corrigé avant push.

---

## Validation

- Streaming : **469 chunks progressifs** dans un vrai Chrome (contre 1 auparavant)
- Conversations : **21 restituées** (contre 0)
- Compilation Rust : `cargo build --locked` — **0 erreur**
- Stack complète : API `healthy`, `/ready` 200, UI 200, PostgreSQL healthy
- Ingestion réelle → `completed`, 15 entités, 9 relations
- Requête RAG `hybrid` → réponse correcte avec traversée de graphe et citations
- Traces Langfuse (ClickHouse `events_core`) : `ingest.document`, `ingest.chunking`,
  `pipeline_chunk_extraction`, `extract-entities-glean`, `embed-chunks`,
  `query_pipeline`, `query.embed`, `retrieval edgequake`, `query.fuse`, `query.rerank`,
  `generate-answer` — typage `GENERATION` / `EMBEDDING` / `RETRIEVER`, tokens comptés
- Rejoué contre Langfuse **3.225.5** (42 observations) et **4**
- Tous les liens relatifs du pack documentaire résolvent

**Non fait** : aucun test automatisé ajouté pour les deux correctifs. Une suite de
non-régression sur l'identité des conversations serait pertinente avant merge.

## Portée

Documentation, `.gitignore`, **plus deux correctifs de code** :
`edgequake_webui/next.config.ts` (1 option) et 3 fichiers de handlers conversations
(7 sites alignés). Aucune migration, aucun changement de schéma, aucun changement
d'API publique.
