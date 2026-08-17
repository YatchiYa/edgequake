
## 🔴 1. CRITICAL — Colonne "id" inexistante dans la table ⁠ workspaces ⁠ (2 304 occurrences !)

*Le problème :*
La requête suivante échoue systématiquement (PostgreSQL error ⁠ 42703 ⁠) :

⁠ sql
SELECT EXISTS (SELECT 1 FROM workspaces WHERE id::text = $1)
 ⁠

La table ⁠ workspaces ⁠ n'a visiblement pas de colonne ⁠ id ⁠. C'est un hot path vu le volume — ça tourne en continu depuis 23h.

*Détails :*
•⁠  ⁠*DB :* ⁠ edgequake ⁠ / user ⁠ edgequake ⁠
•⁠  ⁠*Host source :* ⁠ 10.70.0.246 ⁠
•⁠  ⁠*Phase :* PARSE (cursor position 47)
•⁠  ⁠*Volume :* 2 304 occurrences entre ⁠ 2026-08-02T07:27 ⁠ et ⁠ 2026-08-03T06:27 ⁠
•⁠  ⁠*Environnement :* Production (⁠ quantalogic-prod-db ⁠)

*Sample IDs* (champ filtré : ⁠ _id ⁠) :

_id:603451b0-20b5-43f5-aa22-50c0374080ee OR _id:b2bf5141-ec44-4dad-a70e-8f330f3df100 OR _id:7cbc6f56-0ff6-417f-b91a-a31e5cbf12e4


*Fix suggéré :* Vérifier le vrai nom de la colonne PK dans ⁠ workspaces ⁠ (probablement ⁠ workspace_id ⁠ ?) et mettre à jour la query.

---

## 🟠 2. HIGH — Relation ⁠ edgequake."Node" ⁠ inexistante (24 occurrences)

*Le problème :*
Le storage inspector (⁠ edgequake_api::storage_inspector ⁠) exécute une CTE query horaire qui joint sur ⁠ edgequake."Node" ⁠, mais cette relation n'existe pas (error ⁠ 42P01 ⁠). D'après les logs du timeout (issue #6 ci-dessous), le bon schéma semble être ⁠ eq_eq_default_graph."Node" ⁠.

*Détails :*
•⁠  ⁠*DB :* ⁠ edgequake ⁠ / user ⁠ edgequake ⁠
•⁠  ⁠*Host source :* ⁠ 10.70.0.246 ⁠
•⁠  ⁠*Phase :* PARSE (cursor position 523)
•⁠  ⁠*Volume :* 24 occurrences (1x/heure) entre ⁠ 2026-08-02T07:27 ⁠ et ⁠ 2026-08-03T06:27 ⁠
•⁠  ⁠*Environnement :* Production (⁠ quantalogic-prod-db ⁠)

*Sample IDs* (champ filtré : ⁠ _id ⁠) :

_id:98b29485-dc70-4d63-aebe-7f8690e6b742 OR _id:1d22f716-cc35-445c-84b3-cbe8e81f28ca OR _id:4f0669bf-114a-45f4-9e13-2ddec02df5ab


*Fix suggéré :* Remplacer ⁠ edgequake."Node" ⁠ par ⁠ eq_eq_default_graph."Node" ⁠ dans le code du storage inspector.

---

## 🟠 3. HIGH — Drift critique : 20 documents indexés sans KV chunks (INV-03)

*Le problème :*
Le monitor d'invariants SPEC-021 P-D1 (⁠ edgequake_api::storage_inspector ⁠) remonte un *CRITICAL drift* chaque heure : 20 documents sont présents dans l'index mais n'ont aucun KV chunk associé. Probablement un SAGA d'ingestion qui a partiellement échoué sans rollback propre.

*Détails :*
•⁠  ⁠*Image :* ⁠ ghcr.io/raphaelmansuy/edgequake:0.22.0 ⁠
•⁠  ⁠*Module :* ⁠ edgequake_api::storage_inspector ⁠ / invariant ⁠ INV-03 ⁠
•⁠  ⁠*Documents affectés :* 20
•⁠  ⁠*Volume :* 24 occurrences (1x/heure) entre ⁠ 2026-08-02T07:27 ⁠ et ⁠ 2026-08-03T06:27 ⁠
•⁠  ⁠*Environnement :* Production (⁠ quantalogic-prod-edgequake ⁠)

*Sample IDs* (champ filtré : ⁠ _id ⁠) :

_id:19edb004-68af-496c-b50e-5e920fbafe15 OR _id:6a5d1bf3-9d57-4147-9196-4d68dedd4b2b OR _id:aba5b2c5-034e-480c-8ffc-9f0dda80a0cc


*Fix suggéré :* Identifier les 20 documents orphelins, soit relancer le step KV chunk de la SAGA, soit nettoyer les entrées d'index. Investiguer pourquoi la compensation SAGA n'a pas nettoyé.

---

## 🟡 4. MEDIUM — Duplicate key sur ⁠ tenants_slug_key ⁠ (6 occurrences)

*Le problème :*
L'INSERT dans la table ⁠ tenants ⁠ échoue avec une violation de contrainte unique (⁠ 23505 ⁠) sur ⁠ tenants_slug_key ⁠. Deux slugs en cause : ⁠ novagen-orga-cff5cf8b ⁠ (3x) et ⁠ novagen-orga ⁠ (3x). Race condition ou retry sans idempotence.

*Détails :*
•⁠  ⁠*DB :* ⁠ edgequake ⁠ / user ⁠ edgequake ⁠
•⁠  ⁠*Statement :*
⁠ sql
INSERT INTO tenants (tenant_id, name, slug, is_active, metadata, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, '{}'::jsonb, $6, $7)
 ⁠
•⁠  ⁠*Volume :* 6 occurrences entre ⁠ 2026-08-02T14:48:16 ⁠ et ⁠ 2026-08-02T14:49:15 ⁠
•⁠  ⁠*Environnement :* Production (⁠ quantalogic-prod-db ⁠)

*Sample IDs* (champ filtré : ⁠ _id ⁠) :

_id:6fbfcb20-aefb-444c-adb1-d6559124178e OR _id:c49eec8a-4bb8-4ba8-a717-1bb1d9997045 OR _id:502669e4-5698-4e45-ab45-539039221c24


*Fix suggéré :* Utiliser un ⁠ INSERT ... ON CONFLICT (slug) DO NOTHING ⁠ ou ajouter un check préalable + retry avec régénération de suffix.

---

## 🟡 5. MEDIUM — Statement timeout sur query graph node counts (4 occurrences)

*Le problème :*
La query ⁠ DATA-AGE-GRAPH-NODE-COUNTS-BY-SOURCE-PREFIXES ⁠ sur ⁠ eq_eq_default_graph."Node" ⁠ est annulée pour cause de timeout (⁠ 57014 ⁠). Le pattern CROSS JOIN + containment check JSONB (⁠ @> ⁠) sur ⁠ source_ids ⁠ est trop coûteux sans index GIN.

*Détails :*
•⁠  ⁠*DB :* ⁠ edgequake ⁠ / user ⁠ edgequake ⁠
•⁠  ⁠*Host source :* ⁠ 10.70.0.246 ⁠
•⁠  ⁠*Query ID :* ⁠ 3844827553670455804 ⁠
•⁠  ⁠*Volume :* 4 occurrences entre ⁠ 2026-08-02T14:48:17 ⁠ et ⁠ 2026-08-02T14:48:18 ⁠
•⁠  ⁠*Environnement :* Production (⁠ quantalogic-prod-db ⁠)

*Sample IDs* (champ filtré : ⁠ _id ⁠) :

_id:d005c014-dc41-4259-a620-607b0317e5a4 OR _id:81202f14-a2fd-4c15-8189-8b5c16b48d42 OR _id:120bf835-94f5-43d6-844f-90d9d1a2f47a


*Fix suggéré :* Ajouter un index GIN sur ⁠ (ag_catalog.agtype_to_json(properties))::jsonb -> 'source_ids' ⁠ dans la table Node, ou revoir le pattern de query (pré-matérialiser les mappings node→source).

---

## Résumé rapide

| # | Criticité | Problème | Volume |
|---|-----------|----------|--------|
| 1 | 🔴 Critical | Colonne ⁠ id ⁠ inexistante dans ⁠ workspaces ⁠ | 2 304 |
| 2 | 🟠 High | Relation ⁠ edgequake."Node" ⁠ inexistante | 24 |
| 3 | 🟠 High | 20 docs indexés sans KV chunks (INV-03) | 24 |
| 4 | 🟡 Medium | Duplicate slug sur ⁠ tenants ⁠ | 6 |
| 5 | 🟡 Medium | Timeout sur graph node count query | 4 |

Le plus urgent c'est clairement le #1 vu le volume (100 erreurs/heure non-stop). Les #2 et #3 sont liés au storage inspector et tournent en boucle aussi.

Le point sur les 20 documents indexés sans KV chunks (INV-03), on va investiguer de notre coté je pense que c'est une ingestion qui a fail ou un doc mal supprimé.

N'hésite pas si tu veux qu'on creuse un point ensemble.

A++
Steven JAMAN.