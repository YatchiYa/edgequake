# 08 — Operator reply

Oui — il faut **v0.26.3** (`ghcr.io/raphaelmansuy/edgequake:0.26.3`), pas
`0.26.1`.

L’image `0.26.1` applique bien le SAFE SCHEMA (migration **149**). Les DROP OLD
**125 / 126 / 131** restent bloqués tant que la copie n’est pas terminée —
c’est voulu.

Sur `0.26.1` le moteur de copie plante :

1. `iw2-fleet-embedding-backfill` — Postgres refuse un `ON CONFLICT` interne
   (deux clés `entity:` qui se normalisent vers la même ligne).
2. `w3-chunk-embedding-backfill` — le verify compte mal, passe le job en
   `failed`, et ne le relance plus.
3. Les sidecars KV (lineage / multimodal / hash / shells) ne sont pas rejoués
   après la création des documents (migrations 117–122 one-shot).

**Ne pas** `--confirm-drop` tant que `edgequake migrate guard` est RED.
Après copie, quelques orphelins (pas de parent `documents` / alias SPEC-111)
peuvent rester RED — ne pas forcer le DROP.

Après upgrade **0.26.3** :

```bash
# 1. Déployer ghcr.io/raphaelmansuy/edgequake:0.26.3 (pas 0.26.1)
# 2. Démarrer le serveur (EDGEQUAKE_MIGRATION_MODE=automatic)
# 3. Suivre : edgequake migrate status / guard
# 4. Quand GREEN + backup :
edgequake migrate --confirm-drop
edgequake migrate    # assert 142
```

`migrate guard` ne copie rien. Détail : [09-ops-runbook.md](09-ops-runbook.md).
