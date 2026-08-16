# 00 — Why SPEC-132

## Trigger

Intake: [`zz-raw.md`](zz-raw.md) — GitHub [#378](https://github.com/raphaelmansuy/edgequake/issues/378).
On Docker **v0.24.4**, selecting multiple PDFs in Documents upload leaves files stuck in the upload process; reporters say files are not uploaded.

## Product WHY

```ascii
  Operator: “I selected five PDFs — why are they still Uploading
             and none appear in the table?”
       │
       ▼
  Today (possible planes):
       A. Admit hang — HTTP never returns; client pool (cap=3) freezes
       B. Admit OK, convert starved — UI still looks like “upload”
       C. Wrong route — /upload/batch rejects PDFs (API clients/docs)
       D. Capacity — slow KG (this is #361, not “not uploading”)
              │
              ▼
  Blind spots:
       1. “Upload” vocabulary conflates transfer+admit with processing
       2. One hung admit blocks siblings behind the client executor
       3. Multi-PDF e2e only covers Markdown (GH-350), not PDF
       4. Wake channel send().await can still block HTTP (F-091-19)
```

## Five WHYs

1. **Why do multi-PDFs look stuck?** Transfer list stays on pending/uploading/extracting without terminal success for all files.
2. **Why doesn’t each file finish admit?** Either HTTP admit blocks/times out, or progress never terminals after admit.
3. **Why can admit block?** `enqueue_task` awaits bounded wake `send().await` after durable persist; full channel stalls the handler.
4. **Why do siblings freeze?** WebUI shares a concurrency-3 executor; hung slots never release; `Promise.all` waits on the whole batch.
5. **Root cause class:** Admit/wake honesty + per-file isolation gap (and optional UX/docs mislabel of post-admit queue as “upload failed”) — **not** missing multi-select.

## Job to be done

> When a user selects N PDF files, each file either admits (HTTP 2xx + durable document/task + `task_id` + list presence) or fails with a clear per-file error — without silently freezing the rest of the selection, and without calling slow convert “upload failure.”

## Success criteria

1. Multi-PDF WebUI path admits ≥2 PDFs with distinct `task_id`s and table rows (KG complete not required).
2. Full wake channel does not hang HTTP admit forever (202 + queued / hydrate).
3. One timed-out admit frees the executor so remaining files proceed or error individually.
4. Docs never tell operators to put PDFs on `/documents/upload/batch`.
5. Edge matrix in [10-edge-cases.md](10-edge-cases.md) has named tests.

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
