# Lens 005 — Front Designer

## Surface inventory

| Surface | Change |
|---------|--------|
| Documents list status chip | Keep lifecycle verbs; failed cleanup uses failure reason string from API |
| Document detail | Show mapped error message; Retry control for reprocess/delete |
| Toast / alert | Prefer short title + one sentence; hide nested `Database error:` chains when mapped |

## Copy tokens (suggested)

| Key | EN |
|-----|----|
| `delete.graph_cleanup_timeout.title` | Graph cleanup timed out |
| `delete.graph_cleanup_timeout.body` | Cleanup did not finish before the time limit. Retry. If this continues, ask an admin to check graph indexes. |
| `reprocess.graph_cleanup_timeout.title` | Reprocess cleanup timed out |

## Visual rules

- Do not invent a new card layout for this error.
- Use existing destructive/warning alert pattern.
- Prefer text hierarchy over badges/chips for the failure reason.
