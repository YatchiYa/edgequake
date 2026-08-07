# issue-363 — iw2 fleet backfill silent join miss

**GH:** https://github.com/raphaelmansuy/edgequake/issues/363  
**Status:** Confirmed present on HEAD and **v0.24.1**  
**Severity:** P0 (false GREEN → under-filled typed embeddings)

## WHY

Irreversible vector DROP must not follow a job that scanned 100% but wrote ~0%.

## Code law

- Lookup: `es.name = $1` exact (`write_relationship_batch` / `write_entity_batch`).
- Miss: `continue`.
- Progress: `processed_count += scanned`.

Legacy keys are normalized forms; AGE-imported spines may store display names → systematic miss.

## Fix (summary)

Normalize on join (LAW-111-6); fail verify on coverage shortfall (LAW-111-4). Phase B in [04-fix-plan](04-fix-plan.md).

## Partner workaround (validated by reporter)

Regenerate embeddings from current `entities`/`relationships` via embed text templates — correct for serving; does not fix iw2; interacts with #364 numeric verify.
