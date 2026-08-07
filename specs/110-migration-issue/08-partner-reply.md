# 08 — Partner Reply (SPEC-110)

> Send after confirming image tag / local proof. FR primary (partner thread); EN appendix.

## Français

Bonjour,

Oui — ton analyse est bonne : **il faut une nouvelle version d’EdgeQuake** pour débloquer la migration.

### Ce qui casse

Sur `ghcr.io/raphaelmansuy/edgequake:0.24.1`, la migration **118** (`spec091 wsdoc backfill`) embarque un SQL qui fait :

`INSERT … SELECT DISTINCT … ON CONFLICT (id) DO UPDATE`

Or les clés legacy `wsdoc:{workspace}:{document}` sont un **index d’appartenance** : le même `document_id` peut apparaître sous plusieurs workspaces. `SELECT DISTINCT` garde alors **plusieurs lignes avec le même `id`**, et PostgreSQL refuse (erreur `ON CONFLICT DO UPDATE command cannot affect row a second time`).

Chez toi : `latest_applied = 117`, donc 118 n’a **jamais** été enregistrée (rollback transactionnel) — un re-run après correctif est sûr.

### Correctif produit

- Dedup sur la clé de conflit : `DISTINCT ON (document_id)` (+ durcissement analogue de la **121**).
- Nouvelle image / binaire (SQL embarqué) — cible **v0.24.2**.
- Les flottes qui auraient déjà appliqué l’ancienne 118 auront un chemin de **réparation de checksum** (comme M078), pas besoin dans ton cas stuck@117.

### Conduite à tenir (PPD)

1. Backup / point de restauration Postgres (toujours avant migrate 091).
2. Quand **0.24.2** (ou image patchée équivalente) est publiée, basculer le tag.
3. Rejouer :

```bash
docker run --rm --env-file /etc/edgequake/.env \
  ghcr.io/raphaelmansuy/edgequake:0.24.2 migrate --confirm-drop
```

4. Vérifier : `edgequake migrate status` (plus de pending 118 ; train jusqu’à 142 selon consent DROP OLD).

Tant que tu restes sur **0.24.1**, modifier le repo en local ne change pas le SQL dans le conteneur.

Spec interne : `specs/110-migration-issue/` · runbook : `09-ops-runbook.md`.

## English (appendix)

Yes — a **new EdgeQuake release** is required. v0.24.1 embeds buggy migration **118**: multi-workspace `wsdoc` keys produce duplicate conflict targets under `ON CONFLICT DO UPDATE`. Your DB stopped at **117** (safe to re-run). Pull **v0.24.2**, re-run `migrate --confirm-drop`. Details: SPEC-110.
