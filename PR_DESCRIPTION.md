## Résumé

Ajoute un **dossier de documentation technique interne** (déploiement, exploitation,
architecture, observabilité) rédigé à partir du code source v0.26.1, et un **runbook
de développement local Langfuse**. Toutes les affirmations techniques sont adossées à
une référence `fichier:ligne` du dépôt et ont été vérifiées contre le code.

Le pack est **anonymisé** (`{client}` en substitution des noms clients, domaines
génériques) et destiné à être réutilisable pour tout déploiement.

## Contenu

### `docs/documentation_interne/` — dossier technique (5 fichiers)

| Fichier | Objet |
|---|---|
| `README.md` | Page de garde, contrôle documentaire, historique des révisions |
| `01-deploiement-technique.md` | Architecture déployée, composants, prérequis, flux de données, réseau, sécurité, activation de l'authentification, installation, recette en 15 points |
| `02-integration-it.md` | Exploitation, monitoring, sauvegarde/restauration, mise à jour, rollback, runbooks d'incident, checklists |
| `03-deep-dive-architecture-algorithme.md` | Découpage en crates, algorithme d'ingestion, modèle de données, moteur d'interrogation, 8 décisions d'architecture |
| `04-langfuse-kubernetes.md` | Compatibilité de version Langfuse et déploiement en pods séparés |

### `RUNBOOK-LOCAL-LANGFUSE.md`

Démarrage local avec Langfuse (`make dev-bg-langfuse`), pièges rencontrés, procédure
de vérification du traçage, annexe Kubernetes.

### `.gitignore`

Ajoute `.env.*` avec négations `!.env.example` / `!.env.*.example`.
**Motif** : les motifs existants `.env` et `*.env` ne couvrent pas `.env.backup-…`
ni `.env.<suffixe>` — un fichier de sauvegarde local contenant des secrets peut donc
être committé par inadvertance (constaté sur cette branche, corrigé avant push).

## Constats techniques documentés

Ces points ont été établis par test sur le code v0.26.1, pas par lecture de
documentation existante :

- **Langfuse < 3.22x est incompatible.** L'endpoint OTLP `/api/public/otel/v1/traces`
  utilisé par `edgequake-observability` renvoie **404 en Langfuse 3.1** ; il répond en
  3.225.5 et en 4. Chemin codé en dur (`langfuse.rs:105`), sans repli.
- **Repli silencieux vers Langfuse Cloud.** `DEFAULT_LANGFUSE_BASE_URL =
  "https://cloud.langfuse.com"` (`langfuse.rs:9`), et les variables vides sont
  filtrées comme absentes. Une variable `LANGFUSE_BASE_URL` vide (clé de ConfigMap
  manquante en Kubernetes) fait donc sortir les traces du réseau interne, sans erreur.
- **`export_active: true` n'atteste que de la présence des clés**, jamais de l'arrivée
  des traces (`enabled = keys_ok`). Les échecs d'export sont journalisés en `DEBUG`,
  donc invisibles avec `RUST_LOG=info`.
- **Course aux migrations Langfuse** reproduite en 3.1 et en 4 : le worker démarre
  avant la fin des migrations appliquées par le web ; l'OTLP répond 200 mais aucune
  trace n'est créée.
- **`EDGEQUAKE_TASK_DELIVERY`** : toute valeur non reconnue retombe silencieusement
  sur `local` (mono-processus) — piège en déploiement multi-réplique.
- **`GET /api/public/traces`** renvoie 0 en Langfuse v4 même lorsque tout fonctionne
  (données dans `events_core`) ; l'API reste exploitable en v3.

## Validation

Chaîne complète vérifiée de bout en bout sur cette branche :

- Stack démarrée via `make dev-bg-langfuse` — API `healthy`, `/ready` 200, UI 200
- Ingestion réelle → `completed`, 15 entités, 9 relations
- Requête RAG `hybrid` → réponse correcte avec traversée de graphe et citations
- Traces reçues dans Langfuse (ClickHouse `events_core`) : `ingest.document`,
  `ingest.chunking`, `pipeline_chunk_extraction`, `extract-entities-glean`,
  `embed-chunks`, `query_pipeline`, `query.embed`, `retrieval edgequake`, `query.fuse`,
  `query.rerank`, `generate-answer` — avec typage `GENERATION` / `EMBEDDING` /
  `RETRIEVER` et décompte de tokens
- Rejoué avec succès contre Langfuse **3.225.5** (42 observations) et **4**
- Tous les liens relatifs du pack résolvent

## Portée

Documentation et `.gitignore` uniquement — **aucune modification du code applicatif**,
aucun impact sur le comportement à l'exécution, aucune migration.
