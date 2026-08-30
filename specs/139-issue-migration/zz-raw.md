
User issue during migration :

"""

Voici mes updates sur le problème de migration je pense qu'il y a du mieux mais j'ai toujours des erreurs. Voici les étapes que j'ai effectué : 

1. edgequake migrate
2. edgequake guard (qui affiche un status RED)
3. demarrage de edgequake pour traiter les résidu mais j'observe les erreurs suivantes en fin de process :

---
2026-08-26T15:16:33.894449Z ERROR edgequake_storage::migration_engine::runner: migration verification FAILED step="w3-chunk-embedding-backfill" report=VerifyReport { metric: "w3-chunk-embedding-fleet", expected: 44580, actual: 18503, sampled: 2416, mismatches: 1370 }
2026-08-26T15:16:33.895916Z  INFO edgequake_storage::migration_engine::runner: migration lease claimed — starting batches step="iw2-fleet-embedding-backfill" job_id=f2ef3a5d-cd8f-48d0-a44a-e03e9b6f521a owner=lucien:1
2026-08-26T15:16:34.031085Z ERROR edgequake_storage::migration_engine::runner: SPEC-091 migration engine terminated with error error=Database error: iw2 entity insert failed: error returned from database: ON CONFLICT DO UPDATE command cannot affect row a second time
 ---

Aussi, en faisant un edgequake guard après, je vois qu'il reste des résidus.

Je te joins tous les fichiers de logs que j'ai.

Merci pour ton aide.
""""