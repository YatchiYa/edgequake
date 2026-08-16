# 00 — Why SPEC-130

## Trigger

Intake: [`zz-raw.md`](zz-raw.md) — GitHub [#380](https://github.com/raphaelmansuy/edgequake/issues/380).

Typed KG persist fail-closes with:

```text
SPEC-091: typed fleet mirror resolved 0/N rows
  (… ensure PostgresEntitySink wrote the spine before fleet mirror;
   SPEC-098 misses: [SRC->TGT:REL, …])
```

Relationship triples appear in the miss sample while `public.entities` and (after the attempt) `public.relationships` look correct by name. Retries reproduce the same misses with `EDGEQUAKE_TASK_MAX_WORKERS=1`.

## Product WHY

```ascii
  Operator: “199 docs Failed — entities and edges exist in SQL;
             why does fleet mirror say 0/N every reprocess?”
  Monitor:  “Error blames entity spine; entities are fine.
             Is this a race? A name bug? Permanent?”
       │
       ▼
  Today (gap):
       RelGraph sink INSERT relationships  ──► UUID discarded
       RelVectors mirror                     ──► re-resolve by name
                                               └── miss ⇒ GraphMerge permanent
       Error hint points at entity spine     ──► wrong diagnosis attractor
              │
              ▼
  Blind spots:
       1. Timing race narrative fits the ~1s created_at gap but not typed order
       2. Identity established once, then thrown away (DRY / SSOT break)
       3. Retry cannot heal a deterministic re-lookup miss
```

## Five WHYs

1. **Why does the document fail?** Typed `mirror_legacy_batch` reports `resolved < eligible`; merger fail-closes (LAW-098-4).
2. **Why is `resolved` zero for relationship rows?** `resolve_relationship_id_pool` returns `None` for each legacy id `SRC->TGT:TYPE`.
3. **Why does SELECT miss while a named edge often exists afterward?** Mirror does not use the UUID the sink just wrote; it re-derives endpoints via `EntityNameIndex` (oldest-wins) and queries by `(source_id, target_id, relation_type, workspace_id)`. Any divergence (duplicate names, alias pick, workspace meta, criteria skew) yields a permanent miss. A pure SELECT-before-INSERT race is **not** required — and contradicts leftover spine + identical retries.
4. **Why did diagnosis land on “unordered writers”?** Error text says “wrote the **spine** before fleet mirror,” and entity vs relationship `created_at` differs by ~1s (expected between EntityGraph sink and RelGraph sink).
5. **Root cause:** **Relationship identity is not passed from sink to mirror.** Re-resolve-by-name is a second, divergent SSOT. Sequencing RelGraph→RelVectors already exists under typed authority; it is necessary but insufficient when identity is discarded.

## Job to be done

> When a document’s relationships are sunk into `public.relationships`, the same merge session’s RelVectors fleet mirror must attach embeddings to those exact relationship rows — without a second name-based identity guess — so dense corpora stop failing permanently on every retry.

## Success criteria

1. In-session RelVectors mirror uses sink-returned relationship UUIDs (no name SELECT for those rows).
2. Typed RelGraph → RelVectors await order remains an invariant (documented + tested).
3. Fail-closed hint names relationship identity / FK miss correctly (not only entity spine).
4. Duplicate-name / alias divergence no longer fails in-session mirror when sink wrote the edge.
5. e2e + contract gates in [08-test-protocol.md](08-test-protocol.md) / [10-edge-cases.md](10-edge-cases.md) pass.
6. GitHub #380 carries an honest RC comment linking this pack.

## Cross-refs

- Laws: [01-first-principles.md](01-first-principles.md)
- Code: [03-code-as-is.md](03-code-as-is.md)
- Plan: [07-implementation-plan.md](07-implementation-plan.md)
- Edges: [10-edge-cases.md](10-edge-cases.md)
