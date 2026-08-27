# 12 — Lens: Database Expert

> **Cross-refs:** [LAW-138-3](01-first-principles.md) · [Architecture](00-architecture-data.md)

## EdgeQuake Postgres

- Image: `edgequake-postgres` (pgvector + AGE baked in)
- StatefulSet + RWO PVC
- Extensions: see [`init-extensions.sql`](../../edgequake/docker/init-extensions.sql)
- Migrations: SQLx on API boot (unchanged)

## Langfuse Postgres

- Separate instance in `langfuse` namespace (Langfuse Helm bundled)
- **Never** share `DATABASE_URL` with EdgeQuake (EC8)

## PVC notes

- kind: default StorageClass `standard`
- Production: pin `storageClassName`; size >= 20Gi recommended

## shm / memory

- Compose uses `shm_size: 256m`; K8s uses `emptyDir` medium Memory on postgres pod
