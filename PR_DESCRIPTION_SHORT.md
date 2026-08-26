## Description

Corrige deux bugs qui rendaient l'interface de requête inutilisable, et ajoute un
dossier de documentation technique interne. Les deux bugs sont **invisibles en `curl`
et ne se reproduisent que dans un vrai navigateur** — d'où leur passage inaperçu.

**1. Le streaming SSE n'arrivait jamais dans le navigateur.** Next.js gzippe les
réponses proxifiées par défaut (`compress: true`) ; l'encodeur gzip tamponne tout le
corps, donc le `text/event-stream` arrivait en un seul bloc à la fin. L'API Axum, elle,
ne compresse pas le SSE. Mesuré dans Chrome : **1 chunk à 12,4 s → 469 chunks
progressifs**. Correctif : `compress: false` dans `edgequake_webui/next.config.ts`.

**2. L'historique des conversations était toujours vide.** Asymétrie d'identité : en
mode anonyme, les chemins d'**écriture** passent par `ensure_postgres_user_exists`, qui
ignore volontairement l'identifiant client et renvoie un invité partagé ; les chemins de
**lecture** filtraient sur l'en-tête brut `x-user-id`. 3 sites d'écriture contre 8 de
lecture. Correctif : les 7 sites de lecture (`crud.rs`, `folders.rs`, `bulk.rs`) passent
par la même fonction que les écritures. **0 → 21 conversations restituées.**

**3. Documentation** : `docs/documentation_interne/` (5 fichiers, anonymisés) +
`RUNBOOK-LOCAL-LANGFUSE.md`, rédigés depuis le code v0.26.1 avec références
`fichier:ligne`. Constats établis par test réel : **Langfuse 3.1 est incompatible**
(endpoint OTLP → 404), repli silencieux vers Langfuse Cloud sur variable vide,
`export_active: true` ne prouve pas l'arrivée des traces, course aux migrations Langfuse.

**4. `.gitignore`** : ajoute `.env.*` avec négations `!.env*.example`. Les motifs
existants (`.env`, `*.env`) ne couvraient pas `.env.backup-…` — un fichier de secrets
pouvait être committé par inadvertance (constaté sur cette branche, corrigé avant push).

Fixes # (aucune issue ouverte — bugs remontés en usage direct)

## Type of change

- [x] Bug fix
- [ ] New feature
- [ ] Breaking change
- [x] Documentation update
- [ ] Other (describe below)

## Checklist

- [x] My code follows the style guidelines of this project
- [x] I have performed a self-review of my code
- [x] I have commented my code, particularly in hard-to-understand areas
- [x] I have made corresponding changes to the documentation
- [x] My changes generate no new warnings
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing tests pass locally with my changes

> **Deux cases non cochées, volontairement :** aucun test automatisé n'accompagne les
> deux correctifs, et la suite complète n'a pas été rejouée localement. La validation
> est manuelle et reproductible (ci-dessous). Une non-régression sur l'identité des
> conversations serait pertinente avant merge.

## Additional context

**Validation manuelle**

| Contrôle | Avant | Après |
|---|---|---|
| Chunks SSE reçus dans Chrome | 1 (à 12,4 s) | **469** progressifs |
| Conversations listées | 0 | **21** |
| `cargo build --locked` | — | 0 erreur |

Stack complète vérifiée : API `healthy`, `/ready` 200, UI 200, PostgreSQL healthy,
ingestion réelle → `completed` (15 entités, 9 relations), requête RAG `hybrid` avec
citations, traces Langfuse reçues dans ClickHouse (`generate-answer`, `retrieval
edgequake`, `embed-chunks`… typées `GENERATION` / `EMBEDDING` / `RETRIEVER`), rejoué
contre Langfuse 3.225.5 et 4.

**Portée** : documentation, `.gitignore`, et 2 correctifs de code (1 option Next.js,
7 sites Rust alignés). Aucune migration, aucun changement de schéma ni d'API publique.

**⚠️ À prévoir en production** : nginx / Traefik / les Ingress Kubernetes compressent
aussi par défaut. Il faut y **exclure explicitement `text/event-stream`**, sinon le
streaming recassera à l'identique — et le diagnostic sera pénible, puisque `curl` ne
reproduit pas le symptôme.
