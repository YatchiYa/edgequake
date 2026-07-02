# SPEC-040 — Product Owner Lens

**Lens:** Product owner / business value  
**Audience:** Release prioritization, customer communication

---

## Executive summary

Five GitHub issues from v0.12.11 production users share a narrative: **“The app looks broken or frozen after real workloads.”** They erode trust in upgrades (#250), block re-upload workflows (#253), prevent local model adoption (#251), and make multi-workspace deployments unusable (#259, #262).

Fixing #262 and #253 unblocks the majority of support tickets; #251 and #250 are quick wins for operator satisfaction.

---

## Issue impact matrix

| Issue | User persona | Pain | Frequency | Revenue risk |
| ----- | ------------ | ---- | --------- | -------------- |
| #262 | Power user / admin | Graph page empty, stats timeout, “frozen” dashboard | Every large workspace | **High** — churn on graph-first workflows |
| #259 | Multi-tenant admin | Query errors mid-session, slow switching | Intermittent, scales with workspaces | **High** — ECS/production |
| #253 | Document curator | Cannot re-upload after upgrade; duplicate loop | After migrations / failed deletes | **Medium** — blocks ingestion |
| #251 | Self-hosted / Docker | Custom Ollama models invisible | Every custom catalog attempt | **Medium** — docs promise override |
| #250 | Admin / support | “Are we on the right version?” | Every mismatched deploy | **Low** — confusion, not data loss |

---

## User stories (acceptance criteria)

### #262 — Graph performance

> **As an** admin with 25k+ entities,  
> **I want** workspace stats and the knowledge graph to load in seconds,  
> **So that** I can validate ingestion without assuming the product crashed.

**Acceptance:**

- [ ] `/workspaces/{id}/stats` returns entity counts in <4s (or `stale: true` with cached values)
- [ ] Graph stream SSE delivers first nodes in <15s on 30k/23k graph
- [ ] No `Graph query timed out (tokio)` in logs under normal load

### #253 — Duplicate replace

> **As a** user re-uploading a markdown file after cleanup,  
> **I want** Replace to reprocess the document,  
> **So that** I don’t see an empty list plus a duplicate dialog.

**Acceptance:**

- [ ] Empty document list ⇒ no duplicate dialog for same content (orphan hash recycled)
- [ ] Replace ⇒ document appears in list within one refresh cycle
- [ ] Error toast if replace cannot proceed (not silent loop)

### #259 — Multi-workspace queries

> **As a** user switching workspaces,  
> **I want** queries to succeed without database errors,  
> **So that** I can explore multiple knowledge bases reliably.

**Acceptance:**

- [ ] No `messages_conversation_id_fkey` errors in normal use
- [ ] Workspace switch clears or validates conversation before submit
- [ ] Friendly recovery if conversation deleted during long query

### #251 — Runtime models

> **As a** Docker operator,  
> **I want** to mount `models.toml` and see new models in the picker,  
> **So that** I don’t rebuild images for each Ollama pull.

**Acceptance:**

- [ ] `EDGEQUAKE_MODELS_CONFIG` honored; log line confirms path
- [ ] Custom model appears in `/api/v1/models/llm` and UI picker

### #250 — Version display

> **As an** admin,  
> **I want** one clear version indicator,  
> **So that** I know upgrades succeeded.

**Acceptance:**

- [ ] Footer and API health show same semver on coupled releases, OR
- [ ] Explicit “Release bundle 0.13.1” label with CI enforcement

---

## ROI estimate

| Fix bundle | Eng effort | Support hours saved / quarter | Break-even |
| ---------- | ---------- | ------------------------------ | ---------- |
| #262 migration + verify script | 2–3 days | ~40h (graph timeout triage) | Immediate |
| #253 orphan hash recycler | 1 day | ~15h (duplicate upload) | 2 weeks |
| #259 conversation lifecycle | 1–2 days | ~20h (multi-WS ECS) | 3 weeks |
| #251 models precedence | 2 hours | ~10h (Docker model config) | 1 week |
| #250 release manifest | 4 hours | ~5h (version confusion) | 1 month |

---

## Release communication template

```markdown
### Fixed in v0.13.x
- **Graph performance (#262):** Workspace-scoped graph queries now use child-table indexes and updated planner statistics. Large graphs (25k+ nodes) no longer hit 15s timeouts on stats/graph views.
- **Duplicate upload (#253):** Orphan content-hash keys are recycled when no visible document exists; Replace reliably reprocesses markdown and PDF uploads.
- **Multi-workspace chat (#259):** Conversation state resets on workspace switch; streaming saves validate conversation existence.
- **models.toml (#251):** Runtime catalog override via EDGEQUAKE_MODELS_CONFIG now works as documented.
- **Version display (#250):** UI and API versions synchronized at build time for official Docker images.
```

---

## Out of scope (PO agreement)

- Raising global graph timeout beyond 15s without index fix
- Disabling duplicate detection entirely
- Merging UI and API into single binary (keep dual artifact with lockfile)
