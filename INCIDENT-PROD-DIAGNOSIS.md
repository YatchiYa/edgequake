# Incident prod EdgeQuake — diagnostic & remédiation

**TL;DR : oui, ce sont de vrais problèmes. Ils ont UNE seule cause racine.**
La migration de schéma **SPEC-062 (`eq_node_id` / `eq_source_id` / `eq_target_id`)** n'arrive **jamais à s'appliquer sur le gros graphe de prod** (178 273 nœuds). Comme ces colonnes/index n'existent pas, le code de requête et de merge qui les suppose présents **échoue en dur** (chat) ou **retombe sur des scans O(N) non indexés** (les requêtes à 30 min qui saturent le CPU). En PPD le graphe est petit → la migration passe instantanément → aucun symptôme. Ton hypothèse « c'est lié au nombre de documents » est **exacte**.

---

## 1. La chaîne causale (une seule)

```
Deploy 0.20.2 (SPEC-062) sur un graphe de 178k nœuds
        │
        ▼
Au boot, ensure_eq_id_columns() tente :
  ALTER TABLE "Node" ADD COLUMN eq_node_id
  ALTER TABLE "EDGE" ADD COLUMN eq_source_id / eq_target_id
  UPDATE ... backfill (agtype_to_json par ligne, 178k lignes)
  CREATE INDEX ...
        │
        ├─ l'ALTER a besoin d'un verrou ACCESS EXCLUSIVE sur "Node"/"EDGE"
        │  mais des SELECT agtype de 30 min tiennent déjà ACCESS SHARE
        │  → l'ALTER attend dans la file de verrous → statement_timeout (300s)
        │  → "canceling statement due to statement timeout"   [tes logs]
        │
        ▼
Les colonnes eq_* ne sont JAMAIS créées.
eq_id_schema_ready() = false, indexes_verified = false
        │
        ├──► CHAT / LOCAL / GLOBAL cassés ───────────────────────────┐
        │    pg_node_degrees_batch() écrit "e.eq_source_id" SANS       │
        │    fallback → column e.eq_source_id does not exist [ton erreur]
        │
        ├──► INGESTION en boucle ────────────────────────────────────┤
        │    le merge natif upsert sur ON CONFLICT (eq_*) → arbiter    │
        │    manquant → "1 knowledge-graph merge error" → saga         │
        │    rollback (174 nœuds) → doc "failed" → statut qui boucle   │
        │                                                              │
        └──► CPU DB à 100% ──────────────────────────────────────────┘
             les chemins sans colonne eq_* retombent sur le scan
             legacy : agtype_to_json(e.properties)->>'source_id' LIKE
             '%...%' sur 178k arêtes → 30 min, seq scan, verrous tenus
             → l'ALTER ne peut TOUJOURS pas passer → cercle vicieux
```

**Le point technique unique** : un `ALTER TABLE … ADD COLUMN` est normalement instantané (métadonnée seule). S'il met 300 s, ce n'est pas le travail — c'est qu'il **attend un verrou** derrière les longues requêtes. Et comme une demande `ACCESS EXCLUSIVE` bloque aussi les nouvelles requêtes derrière elle, ça amplifie la congestion pendant qu'elle attend.

Références code (branche 0.20.2) :
- DDL qui timeout : [graph_lifecycle.rs:409-470](edgequake/crates/edgequake-storage/src/adapters/postgres/graph/helpers/graph_lifecycle.rs#L409) (`ensure_eq_id_columns`)
- Requête cassée sans fallback : [nodes_ops/read.rs:107-185](edgequake/crates/edgequake-storage/src/adapters/postgres/graph/nodes_ops/read.rs#L107) (`pg_node_degrees_batch` — utilise `e.eq_source_id` inconditionnellement)
- Scan 30 min (legacy) : [scan_ops.rs:321](edgequake/crates/edgequake-storage/src/adapters/postgres/graph/scan_ops.rs#L321) (`pg_find_edges_by_source_prefixes`) + [source_lineage_sql.rs](edgequake/crates/edgequake-storage/src/adapters/postgres/graph/helpers/source_lineage_sql.rs) (« Unindexed source_chunk_ids … causes Seq Scan timeouts »)

---

## 2. Chaque symptôme → sa cause

| Symptôme observé | Cause | Réel ? |
|---|---|---|
| `column e.eq_source_id does not exist` (chat) | colonnes SPEC-062 jamais créées + **aucun fallback** dans `pg_node_degrees_batch` | 🔴 bloquant |
| Ingestion boucle sur le statut / doc disparaît au retour | merge natif échoue (arbiter `ON CONFLICT (eq_*)` absent) → saga rollback → doc `failed` | 🔴 bloquant |
| DB CPU à 100%, requête à 30 min | chemins retombent sur le scan agtype `LIKE '%…%'` non indexé sur 178k arêtes | 🔴 critique |
| `ALTER TABLE … eq_source_id` → statement timeout 300s (×3) | verrou ACCESS EXCLUSIVE bloqué par les longues requêtes | 🔴 cause racine |
| `eq_eq_default_graph.unk_ids does not exist` (delete workspace) | bug AGE distinct : le clear référence une label-table jamais créée. **Non bloquant** (log « continuing »), mais laisse des restes | 🟠 mineur |
| `CRITICAL: Storage invariant violations detected (1)` au boot | l'inspecteur SPEC-021 signale précisément le schéma eq_* incomplet | 🟠 symptôme, pas cause |
| Front n'affiche pas le chunk d'erreur SSE | le front ignore la frame `{"type":"error"}` du stream | 🟠 bug front séparé |
| PPD OK, prod KO | graphe PPD petit → ALTER prend le verrou instantanément → schéma créé | ✅ confirme le diagnostic |

---

## 3. Remédiation

### 3.A — Débloquer la PROD maintenant (sans redéploiement)

L'idée : appliquer manuellement le schéma eq_* **une fois**, verrou libre, backfill batché, index en `CONCURRENTLY`. Dès que les colonnes+index+triggers existent, l'app arrête d'essayer la DDL, le chemin rapide s'active, le CPU retombe.

**Étape 1 — tuer les requêtes qui tiennent les verrous** (fenêtre calme) :

```sql
-- repérer les longues requêtes graphe
SELECT pid, now()-query_start AS age, left(query,80)
FROM pg_stat_activity
WHERE state='active' AND query LIKE '%_ag_label_edge%' AND now()-query_start > interval '30s'
ORDER BY age DESC;

-- les terminer (adapter les pid)
SELECT pg_terminate_backend(pid) FROM pg_stat_activity
WHERE state='active' AND query LIKE '%_ag_label_edge%' AND now()-query_start > interval '1 min';
```

**Étape 2 — poser le schéma eq_* proprement** (une session dédiée, verrou borné) :

```sql
SET lock_timeout = '5s';       -- échoue vite si un verrou traîne, ne bloque personne
SET statement_timeout = 0;     -- pour le backfill

-- colonnes (instantané une fois le verrou libre)
ALTER TABLE eq_eq_default_graph."Node" ADD COLUMN IF NOT EXISTS eq_node_id  text;
ALTER TABLE eq_eq_default_graph."EDGE" ADD COLUMN IF NOT EXISTS eq_source_id text;
ALTER TABLE eq_eq_default_graph."EDGE" ADD COLUMN IF NOT EXISTS eq_target_id text;

-- backfill (une passe ; si trop long, batcher par ctid — voir note)
UPDATE eq_eq_default_graph."Node"
   SET eq_node_id = ag_catalog.agtype_to_json(properties)->>'node_id'
 WHERE eq_node_id IS NULL;

UPDATE eq_eq_default_graph."EDGE"
   SET eq_source_id = ag_catalog.agtype_to_json(properties)->>'source_id',
       eq_target_id = ag_catalog.agtype_to_json(properties)->>'target_id'
 WHERE eq_source_id IS NULL OR eq_target_id IS NULL;
```

**Étape 3 — index en CONCURRENTLY** (hors transaction, ne bloque pas les écritures) :

```sql
CREATE UNIQUE INDEX CONCURRENTLY IF NOT EXISTS idx_node_eq_node_id
  ON eq_eq_default_graph."Node" (eq_node_id) WHERE eq_node_id IS NOT NULL;
CREATE UNIQUE INDEX CONCURRENTLY IF NOT EXISTS idx_edge_eq_source_target
  ON eq_eq_default_graph."EDGE" (eq_source_id, eq_target_id)
  WHERE eq_source_id IS NOT NULL AND eq_target_id IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_edge_eq_source_id
  ON eq_eq_default_graph."EDGE" (eq_source_id) WHERE eq_source_id IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_edge_eq_target_id
  ON eq_eq_default_graph."EDGE" (eq_target_id) WHERE eq_target_id IS NOT NULL;

-- ET les GIN du chemin "source prefixes" (sinon la requête 30 min persiste)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_node_source_ids_gin
  ON eq_eq_default_graph."Node"
  USING gin ((ag_catalog.agtype_to_json(properties)::jsonb -> 'source_ids') jsonb_ops);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_edge_source_ids_gin
  ON eq_eq_default_graph."EDGE"
  USING gin ((ag_catalog.agtype_to_json(properties)::jsonb -> 'source_ids') jsonb_ops);
```

**Étape 4 — redémarrer edgequake.** Au boot, `eq_id_schema_ready()` renverra `true` → plus aucune DDL sur le hot path, chat + ingestion repartent, CPU retombe.

> **Note backfill volumineux** : si l'`UPDATE` sur `"EDGE"` dépasse plusieurs minutes, batche-le par `ctid` pour éviter une transaction géante :
> ```sql
> DO $$ DECLARE n int; BEGIN LOOP
>   UPDATE eq_eq_default_graph."EDGE" SET
>     eq_source_id = ag_catalog.agtype_to_json(properties)->>'source_id',
>     eq_target_id = ag_catalog.agtype_to_json(properties)->>'target_id'
>   WHERE ctid IN (SELECT ctid FROM eq_eq_default_graph."EDGE"
>                  WHERE eq_source_id IS NULL LIMIT 20000);
>   GET DIAGNOSTICS n = ROW_COUNT; EXIT WHEN n = 0; COMMIT;
> END LOOP; END $$;
> ```

**Vérifier après coup :**

```sql
-- doit renvoyer 0 : plus de LIKE lourd en cours
SELECT count(*) FROM pg_stat_activity
WHERE state='active' AND query LIKE '%agtype_to_json%LIKE%';

-- la requête doit passer en <100ms maintenant
EXPLAIN ANALYZE SELECT eq_source_id, count(*) FROM eq_eq_default_graph."EDGE"
WHERE eq_source_id = '59eb6a8d-56b3-4b4b-bf87-6ad130343821' GROUP BY eq_source_id;
```

### 3.B — À vérifier côté DBA (2 leviers)

1. **`EDGEQUAKE_SOURCE_PREFIX_LEGACY`** — le scan à 30 min est le chemin *legacy* (opt-in, défaut off, [scan_ops.rs:181](edgequake/crates/edgequake-storage/src/adapters/postgres/graph/scan_ops.rs#L181)). S'il est à `1`/`true` en prod → le retirer. S'il est off, c'est que les GIN modernes manquaient (réglé en 3.A étape 3).
2. **`statement_timeout` du pool** — l'ALTER a été coupé à 300 s. Vérifie ce que vaut le `statement_timeout` par défaut de la connexion edgequake (`SHOW statement_timeout`). Ce n'est pas la cause (la cause est le verrou), mais ça borne l'échec.

### 3.C — Correctifs code (prochaine release) — les vrais bugs

Ces trois-là évitent que ça se reproduise sur tout gros graphe :

1. **`pg_node_degrees_batch` doit avoir un fallback** quand `eq_id_schema_ready()` est faux : utiliser l'expression `agtype_to_json(properties)->>'source_id'` (index `idx_edge_source_id`) au lieu de référencer `eq_source_id` en dur. Aujourd'hui : échec sec → **chat down**. C'est la correction la plus urgente.
2. **La DDL SPEC-062 ne doit pas s'exécuter sous verrou bloquant + backfill monolithique** : `SET lock_timeout` court sur les `ALTER` (échouer vite, réessayer, ne pas bloquer la file), **backfill batché**, index `CONCURRENTLY`. Aujourd'hui : `ensure_eq_id_columns` fait un `UPDATE` global et des `CREATE INDEX` non-concurrents sous verrou → ingérable au-delà de ~quelques dizaines de milliers de lignes.
3. **L'upsert natif de merge doit avoir un arbiter de repli** quand l'unique eq_* n'existe pas encore (sinon ingestion en boucle). Idem : garder le chemin agtype tant que le schéma moderne n'est pas prêt.

Bonus (indépendants) :
- **Front** : gérer la frame SSE `{"type":"error"}` (afficher l'erreur au lieu d'un chat muet).
- **`unk_ids`** : le clear-workspace référence une label-table AGE inexistante → à garder mais ne casse rien (bien loggé « continuing »).

---

## 4. Ordre d'action conseillé

1. **Maintenant** : 3.A (kill scans → schéma eq_* manuel → restart). Débloque chat + ingestion + CPU en une fenêtre.
2. **Aujourd'hui** : 3.B (vérifier les 2 env/timeouts).
3. **Prochaine release** : 3.C.1 en priorité (fallback degrees), puis 3.C.2/3.

Le point de bascule est clair : **tant que `eq_node_id`/`eq_source_id`/`eq_target_id` + leurs index n'existent pas sur le graphe de prod, tout casse ; dès qu'ils existent, tout repart.** Le reste (front, `unk_ids`) est cosmétique à côté.
