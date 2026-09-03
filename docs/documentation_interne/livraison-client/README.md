# EdgeQuake + Langfuse — fichiers a deployer

Version EdgeQuake : **v0.26.1**
Versions Langfuse couvertes : **3.22 → 3.155.1 OSS → 4.x** (transport auto-detecte)

---

## 1. Ce qu'il y a a livrer

| # | Fichier | Obligatoire | Destination |
|---|---------|-------------|-------------|
| 1 | `edgequake.env.example` | **oui** | Source des variables du pod EdgeQuake (voir §2) |
| 2 | `models.toml` | non | ConfigMap monte sur `/app/models.toml`, **seulement** pour surcharger le catalogue |
| 3 | `scripts/langfuse_sync_model_prices.py` | non | Poste d'admin, execute une fois contre Langfuse (voir §5) |

Les fichiers 2 et 3 ne sont pas dupliques ici : les prendre tels quels dans le
depot au tag **v0.26.1** (`edgequake/models.toml` et
`scripts/langfuse_sync_model_prices.py`), pour eviter toute derive de version.

Il n'y a **rien d'autre**. Pas de fichier de configuration applicatif : EdgeQuake
se configure integralement par variables d'environnement, et le catalogue de
modeles (`models.toml`) est **embarque dans le binaire a la compilation**
(`include_str!`). Le livrer n'a d'interet que pour ajouter un modele ou corriger
un tarif sans recompiler.

---

## 2. Ou mettre les variables

### Kubernetes (deux pods : EdgeQuake + Langfuse)

`edgequake.env.example` n'est **pas** lu par le conteneur : c'est le modele a
transcrire. Le decoupage recommande :

**ConfigMap** — tout ce qui n'est pas marque `[SECRET]` dans le fichier :

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: edgequake-config
data:
  EDGEQUAKE_DEFAULT_LLM_PROVIDER: "mistral"
  EDGEQUAKE_DEFAULT_LLM_MODEL: "mistral-small-latest"
  EDGEQUAKE_DEFAULT_EMBEDDING_PROVIDER: "mistral"
  EDGEQUAKE_DEFAULT_EMBEDDING_MODEL: "mistral-embed"
  EDGEQUAKE_DEFAULT_EMBEDDING_DIMENSION: "1024"
  EDGEQUAKE_VISION_PROVIDER: "mistral"
  EDGEQUAKE_VISION_MODEL: "pixtral-12b-2409"
  LANGFUSE_BASE_URL: "http://langfuse-web.langfuse.svc.cluster.local:3000"
  LANGFUSE_PROJECT_ID: "<project-id>"
  EDGEQUAKE_LANGFUSE_ENABLED: "1"
  EDGEQUAKE_LANGFUSE_API: "auto"
  EDGEQUAKE_HOST: "0.0.0.0"
  EDGEQUAKE_PORT: "8080"
  RUST_LOG: "info"
  EDGEQUAKE_DEV_MODE: "false"
  EDGEQUAKE_AUTH_ENABLED: "true"
  EDGEQUAKE_STRICT_STARTUP: "1"
  EDGEQUAKE_CORS_ORIGINS: "https://edgequake.<domaine-interne>"
  WORKER_THREADS: "4"
```

**Secret** — les 5 valeurs `[SECRET]` :
`DATABASE_URL`, `MISTRAL_API_KEY` (ou equivalent fournisseur), `JWT_SECRET`,
`LANGFUSE_PUBLIC_KEY`, `LANGFUSE_SECRET_KEY`.

**Deployment** :

```yaml
    envFrom:
      - configMapRef: { name: edgequake-config }
      - secretRef:    { name: edgequake-secrets }
```

> Preferer l'URL du **Service** Langfuse (`http://langfuse-web...svc.cluster.local:3000`)
> a l'URL Ingress publique : cela evite le TLS interne et la question de la CA.
> Si vous passez malgre tout par l'Ingress HTTPS avec une CA interne, la CA doit
> etre presente dans le trust store **systeme** du conteneur EdgeQuake
> (`/etc/ssl/certs`, `update-ca-certificates`).

### Deploiement local depuis les sources

Copier le modele vers **`.env` a la racine du depot** — c'est ce fichier que le
`Makefile` charge (`Makefile:228 : -include $(ROOT_DIR)/.env`). Le fichier
`edgequake/.env` n'est **jamais** lu par les cibles `make dev*`.

---

## 3. Compatibilite Langfuse — ce qu'il faut retenir

Laisser `EDGEQUAKE_LANGFUSE_API=auto`. Au demarrage, EdgeQuake sonde
`POST {LANGFUSE_BASE_URL}/api/public/otel/v1/traces` :

| Reponse de la sonde | Version concernee | Transport retenu |
|---------------------|-------------------|------------------|
| 401 / 2xx / 4xx ≠ 404 | ≥ 3.22, dont **3.155.1** et 4.x | **OTLP/HTTP** |
| 404 | < 3.22 (ex. 3.1.x) | **API native** `/api/public/ingestion` |

**3.155.1 OSS possede l'endpoint OTLP** (verifie par mesure). Forcer
`EDGEQUAKE_LANGFUSE_API=ingestion` sur cette version est inutile et n'apporte
rien : gardez `auto`.

Verification apres demarrage — l'ecran *Settings → Langfuse Observability*
affiche `Base URL` et le statut. Statut attendu : **Enabled**.

Piege le plus frequent : `LANGFUSE_BASE_URL` vide ou absente. EdgeQuake bascule
alors **silencieusement** sur `https://cloud.langfuse.com`, et les traces
n'arrivent jamais sur l'instance interne. Ne definissez pas non plus
`LANGFUSE_HOST` en parallele avec une valeur divergente.

---

## 4. Prerequis hors fichiers

1. **PostgreSQL 16/17/18** avec `pgvector` **et** `Apache AGE` installes.
2. **Migrations** : les 147 migrations sont **embarquees dans le binaire**
   (`sqlx::migrate!`) — aucun fichier SQL a livrer. Mais le serveur ne migre
   **jamais** tout seul : lancer `edgequake migrate` en Job d'init ou
   initContainer **avant** le demarrage de l'API. Si le schema est
   desynchronise, l'API refuse de demarrer et sort en **code 78** (`EX_CONFIG`).
   Previsualisation : `edgequake migrate dry-run`.
3. **Reseau** : le pod EdgeQuake doit joindre le Service Langfuse et l'API du
   fournisseur LLM.

---

## 5. Couts dans Langfuse (facultatif)

EdgeQuake n'emet **jamais** d'attribut de cout : Langfuse est la source unique
de verite (regle interne LAW-124-12). Le cout affiche vient donc du catalogue de
prix **de Langfuse**, pas d'EdgeQuake.

Les instances 3.x embarquent un catalogue fige a 2024 : les modeles recents
(gpt-5.x, claude-sonnet-4.x, gemini-2.5, mistral-*) y sont absents, d'ou un
cout affiche a **0**. Correction, une seule fois par instance :

```bash
LANGFUSE_BASE_URL=... LANGFUSE_PUBLIC_KEY=... LANGFUSE_SECRET_KEY=... \
  python3 scripts/langfuse_sync_model_prices.py          # DRY_RUN=1 pour simuler
```

Le script pousse les tarifs de `models.toml` dans Langfuse. Il n'agit que sur
les traces **futures** ; les traces deja enregistrees gardent leur cout a 0.

---

## 6. Ne pas transmettre

Le fichier `.env` de developpement du depot **n'est pas transferable** : il
contient des chemins absolus de poste de travail, des ports de contournement
locaux (PostgreSQL 5433), une instance Langfuse `localhost:3330` et une cle API
en clair. Utiliser exclusivement `edgequake.env.example`.
