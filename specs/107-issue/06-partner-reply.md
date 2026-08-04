## Reply draft (French)

Bonjour,

Oui — on constate les mêmes classes d’erreurs de notre côté, reproduites sur l’image **`ghcr.io/raphaelmansuy/edgequake:0.22.0`** (même cadence horaire pour le StorageInspector).

Synthèse :

| Log | Cause | Statut |
|-----|-------|--------|
| `column "id" does not exist` sur `workspaces` (~2300/j) | Monitor INV-D2 utilisait `id` au lieu de `workspace_id` | **Corrigé ≥ 0.24.0** |
| `relation "edgequake.Node" does not exist` (24/j) | Inspector joignait le graphe legacy `edgequake` ; le vrai graphe est `eq_eq_default_graph` | **Corrigé ≥ 0.24.0** |
| INV-03 CRITICAL (20 docs indexés sans chunks) | Alarme **légitime** (orphelins SAGA / delete partiel). Le monitor lit maintenant `public.chunks` \| KV | **Upgrade + nettoyage ops** |
| `tenants_slug_key` (6×) | Create tenant non idempotent sous retry | **Corrigé ≥ 0.24.0** (get-or-create → 201/200/409) |

Action recommandée : monter en **≥ 0.24.0** (idéal **0.24.1**), puis traiter les docs INV-03 (requeue ou delete) via le runbook `specs/107-issue/04-residual-ops.md`.

OK pour une session : on peut enchaîner sur inspect admin, IDs orphelins, et checklist upgrade.

Cordialement

## Session agenda (60–90 min)

1. **Confirm pin** — image tag on `quantalogic-prod-edgequake` (expect 0.22.0).
2. **E1/E2 smoke** — after upgrade candidate, grep PG logs: zero new `42703` / `edgequake."Node"`.
3. **INV-03** — run orphan SQL from [04](04-residual-ops.md); triage sample IDs; choose requeue or delete.
4. **E4** — retry `POST /tenants` same slug+name → expect 200, not 500.
5. **Follow-ups** — multi-workspace INV-C scope (SPEC-104 EC-05); node-count timeout capacity (SPEC-089 / SPEC-104 #5) if still in their longer dump. See [07-residual-risks.md](07-residual-risks.md).

## Artifacts to bring

- `GET /api/v1/admin/storage/inspect` JSON (redacted)
- `SELECT name FROM ag_catalog.ag_graph;`
- Image digest / tag
- Orphan doc id list (≥ sample of 20)
