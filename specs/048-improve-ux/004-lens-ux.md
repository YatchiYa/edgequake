# 004 — Lens: UX

**Job:** reduce uncertainty during long async work  
**Research anchors:** LogRocket async UI patterns · Execution Progress View · endowed progress

---

## 1. Mental model (target)

```text
  "I submitted a document."
           │
           ▼
  ┌─────────────────────────────────────┐
  │  ONE run: this file, this track_id  │
  │  Stages are a checklist I can scan  │
  │  Numbers mean real work units       │
  │  I can leave; it continues          │
  └─────────────────────────────────────┘
```

**Today’s broken model:** “The header is busy, the table is done, the dropzone is extracting — which is real?”

---

## 2. Information architecture (progress)

Priority of signals (top → bottom):

1. **Verb + object** — “Extracting entities from *areal_….pdf*”
2. **Position** — stage k of n (checklist)
3. **Count** — chunk 42/351 · page 18/117 · entities 1200/4657 unique
4. **Health** — live / queued / stuck / partial failure
5. **Cost** — secondary (don’t compete with stage)

---

## 3. Anxiety controls

| Anxiety | UX control | FP |
|---------|------------|-----|
| Is it stuck? | Heartbeat: message or counter changes ≤15s while Working | FP-03 |
| How long? | ETA band when countable; else “Long step — typically minutes” | FP-04 |
| Did I break it? | Terminal sticky; errors named | FP-05 |
| Should I refresh? | Auto-update; Refresh is optional | FP-08 |
| Is Busy a lie? | Busy only if active work | FP-06 |

---

## 4. Content principles (microcopy)

1. **Stage names are nouns/verbs users understand** — “Converting PDF”, not `preprocessing`.
2. **Never concatenate two systems’ strings blindly** — banner = `headline` + `detail` from one model.
3. **Partial failure speaks counts** — “12 chunks failed · Retry available”.
4. **Soft-reprocess modes are visible** — “Reusing stored extractions (merge)” vs “Re-extracting”.

---

## 5. UX anti-patterns to kill

| Anti-pattern | Where today | Fix |
|--------------|-------------|-----|
| Four-step upload legend for 10-stage server | `upload-progress-list` | Map client phases → server stages after `track_id` |
| Duplicate stage_message under badge | table row | One line only |
| Queued counted as Processing in tab | `useDocumentTitle` | Split ⏳ Working vs ⏸ Queued |
| Details required for basics | banner | Put N/M on banner when available |

Cross-ref: [009 screens](./009-screens-ascii.md) · [005 UI](./005-lens-ui-designer.md)
