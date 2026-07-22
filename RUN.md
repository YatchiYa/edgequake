# Lancer EdgeQuake — guide

## TL;DR

```bash
cd /home/yarab/Bureau/discovery/edgequake

EDGEQUAKE_VERSION=0.20.1 \
EDGEQUAKE_POSTGRES_TAG=0.20.1-pg18 \
EDGEQUAKE_DEV_MODE=true \
docker compose -p edgequake -f docker-compose.quickstart.yml up -d
```

| Service | URL | Rôle |
|---|---|---|
| WebUI | http://localhost:3000 | interface |
| API | http://localhost:8080 | REST |
| Swagger | http://localhost:8080/swagger-ui | doc API interactive |
| Health | http://localhost:8080/health | statut |
| PostgreSQL | interne (`edgequake-postgres`) | pgvector + Apache AGE |

Le stack se lance en 3 temps (chaîne stricte) : **postgres** (healthy) → **api** (migrations au boot, healthy) → **frontend**.

---

## Ce que valent les variables

| Variable | Valeur | Effet |
|---|---|---|
| `EDGEQUAKE_VERSION` | `0.20.1` | tag des images api + frontend (GHCR) |
| `EDGEQUAKE_POSTGRES_TAG` | `0.20.1-pg18` | image postgres — **choisit la version majeure PG** (voir plus bas) |
| `EDGEQUAKE_DEV_MODE` | `true` | **API ouverte, sans login** — pratique en local, jamais en prod |
| `-p edgequake` | — | nom de projet → volume `edgequake_edgequake-pg-data` |

> ⚠️ La route batch `/api/v1/tenants/{id}/workspaces/stats` **n'existe que dans les sources 0.20.2** (ta branche `perf/workspaces-list-stats`). Aucune image publique ne l'a — les images `0.20.x` de GHCR viennent de l'upstream (0.18.0) et renvoient 404. Pour l'avoir en local, il faut **builder l'image depuis les sources** (voir dernière section).

---

## Commandes du quotidien

```bash
# état
docker ps --filter name=edgequake- --format '{{.Names}}\t{{.Status}}'

# logs api en direct
docker logs -f edgequake-api

# health
curl -s http://localhost:8080/health | python3 -m json.tool

# arrêt (garde les données)
docker compose -p edgequake -f docker-compose.quickstart.yml down

# arrêt + EFFACE les données
docker compose -p edgequake -f docker-compose.quickstart.yml down -v
```

---

## PostgreSQL 16 → 18 : comment, et est-ce « plus puissant » ?

### La réponse courte sur la perf

**Non, pas de façon notable pour EdgeQuake.** Le travail lourd (recherche vectorielle HNSW via pgvector, traversée de graphe via Apache AGE) dépend des **extensions**, pas du cœur PostgreSQL. Entre PG16 et PG18, pour cette charge précise, le gain réel est **marginal**.

Ce que PG18 apporte vraiment (générique, pas spécifique EdgeQuake) :
- **I/O asynchrone** (`io_method`) — surtout utile sur gros scans séquentiels sur disque lent
- améliorations du planner et du VACUUM
- support plus long (fin de vie plus tardive)

Raison honnête de passer à PG18 : **fraîcheur + durée de support**, pas la vitesse. Ne l'attends pas comme un boost de latence sur tes requêtes RAG.

> Les vrais leviers de perf ici sont ailleurs : `EDGEQUAKE_HNSW_EF_CONSTRUCTION` (32 par défaut, 128 recommandé en prod), la RAM/`shared_buffers` du conteneur, et le provider LLM/embeddings.

### Le piège : on ne « met pas à niveau » un volume PG16 vers PG18

Un data-dir PostgreSQL est **lié à sa version majeure**. Pointer une image PG18 sur un volume PG16 → **le conteneur refuse de démarrer** (`database files are incompatible with server`). Deux voies :

**Cas A — les données actuelles sont jetables** (c'est ton cas : le volume ne contient qu'1 workspace)

```bash
# 1. arrêt + suppression du volume PG16
docker compose -p edgequake -f docker-compose.quickstart.yml down -v

# 2. relance en PG18 (volume neuf, init automatique)
EDGEQUAKE_VERSION=0.20.1 \
EDGEQUAKE_POSTGRES_TAG=0.20.1-pg18 \
EDGEQUAKE_DEV_MODE=true \
docker compose -p edgequake -f docker-compose.quickstart.yml up -d
```

**Cas B — tu veux GARDER des données existantes** → dump puis restore

```bash
# avant de tout arrêter, avec le stack PG16 encore up :
docker exec edgequake-postgres pg_dump -U edgequake -d edgequake -Fc -f /tmp/eq.dump
docker cp edgequake-postgres:/tmp/eq.dump ./eq-backup.dump

# repars en PG18 (Cas A étapes 1-2), attends postgres healthy, puis :
docker cp ./eq-backup.dump edgequake-postgres:/tmp/eq.dump
docker exec edgequake-postgres pg_restore -U edgequake -d edgequake --clean --if-exists /tmp/eq.dump
```

### Tags PostgreSQL disponibles

`EDGEQUAKE_POSTGRES_TAG` accepte, pour une version donnée :

| Tag | PG |
|---|---|
| `0.20.1-pg16` | 16 |
| `0.20.1-pg17` | 17 |
| `0.20.1-pg18` | 18 *(défaut du projet)* |

Chaque image embarque **pgvector 0.8.5** + **Apache AGE** (1.7.0, ou 1.6.0 en pg16 — d'où une légère raison de préférer pg17/18 : AGE plus récent).

---

## Obtenir la route batch en local (build 0.20.2 depuis les sources)

Nécessaire seulement si tu veux tester `/tenants/{id}/workspaces/stats` en local (sinon elle est déjà sur ton serveur 0.20.2).

```bash
cd /home/yarab/Bureau/discovery/edgequake

# build de l'image API depuis ta branche (multi-stage Rust, ~15-30 min)
docker build -f edgequake/docker/Dockerfile -t edgequake-local:0.20.2 .

# lance en pointant l'API sur ton image
EDGEQUAKE_VERSION=0.20.1 \
EDGEQUAKE_POSTGRES_TAG=0.20.1-pg18 \
EDGEQUAKE_DEV_MODE=true \
docker compose -p edgequake -f docker-compose.quickstart.yml up -d

docker stop edgequake-api && docker rm edgequake-api
docker run -d --name edgequake-api --network edgequake_default -p 8080:8080 \
  -e DATABASE_URL="postgres://edgequake:edgequake_secret@edgequake-postgres:5432/edgequake" \
  -e EDGEQUAKE_DEV_MODE=true \
  edgequake-local:0.20.2

# test
curl -s "http://localhost:8080/api/v1/tenants/00000000-0000-0000-0000-000000000000/workspaces/stats" | python3 -m json.tool
```

---

## État actuel (session)

- Stack **0.20.1** lancé, 3 services **healthy**, WebUI sur :3000, API sur :8080.
- PostgreSQL **PG16** (volume `edgequake_edgequake-pg-data`, ~81 Mo, **1 workspace**).
- Route batch : **404** (normal — pas dans l'image 0.20.1).
