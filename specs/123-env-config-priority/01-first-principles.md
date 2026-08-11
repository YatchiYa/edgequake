# 01 — First Principles

## Root axiom (deliberateness)

> **A value wins because someone deliberately set it at that layer — not because a lower layer was painted into a higher one.**

Industry configuration precedence ([config precedence patterns](https://python-config-secrets-hub.com/core-configuration-patterns-file-formats/configuration-precedence-rules/), multi-tenant cascades) converges on one rule: **more specific / more deliberate beats more general**. The order is fixed; only which layers are populated changes.

EdgeQuake’s product form of that rule:

```ascii
  Request/Upload  >  Workspace override  >  Tenant default  >  Env/server leaf  >  Compiled default
```

## Derived axioms

1. **Configuration is a contract.** The resolved value is what the system must execute (or an explicit Auto mode that may rewrite).
2. **Unset is not consent.** Empty Option / missing override metadata means “fall through”, never “invent Auto” or “permission to silent rewrite”.
3. **Paint is not provenance.** Filling DTO fields from tenant/env for display must not make the resolver attribute `source=workspace`.
4. **One pure resolver per domain (DRY).** Admit, reprocess, recovery, query, rebuild, and UI preview call the same function (or an honest FE mirror).
5. **Separation of concerns (SOLID).** Resolution ≠ routing ≠ conversion ≠ labeling.
6. **E2E proves the contract.** Unit tests cannot catch “UI says Vision, lineage says EdgeParse”.

## Laws

| Law | Statement | First-principles root |
|-----|-----------|------------------------|
| **LAW-123-1** | What you resolve is what you run | Contract |
| **LAW-123-2** | Request/Upload > Workspace > Tenant > Env > Default | Deliberateness |
| **LAW-123-3** | “Server Default (X)” means X runs (not soft Auto) | Unset ≠ consent |
| **LAW-123-4** | Auto is an explicit enum; never inferred | Unset ≠ consent |
| **LAW-123-5** | One SSOT resolver per domain; FE mirrors; no mutate-lower-into-request | DRY + paint ban |
| **LAW-123-6** | Batch cannot widen/drop overrides vs single-file | Specificity |
| **LAW-123-7** | E2E makes the chain inviolable | Proof |
| **LAW-123-8** | Workspace layer only if **override deliberate** (metadata / Option::Some); inherit-painted DTO fields do not count | Paint ≠ provenance |

## What counts as a deliberate workspace override?

| Domain | Deliberate signal | Not deliberate |
|--------|-------------------|----------------|
| PDF parser | `pdf_parser_backend: Some(_)` | `None` (“Server Default”) |
| Vision LLM | metadata `vision_llm_*` **or** stored Option set by user | `None` after clear; inherit-painted Some |
| LLM / embedding | metadata `llm_*` / `embedding_*` keys present | Concrete DTO strings filled only by inherit |

## Domains under LAW-123-2

| Domain | Resolver | Notes |
|--------|----------|-------|
| PDF parser | `resolve_pdf_parser_choice` | Auto opt-in only |
| LLM | `resolve_llm_choice` | Workspace layer gated by metadata (LAW-123-8) |
| Embedding | `resolve_embedding_choice` | Same + dimension cascade |
| Vision LLM | `resolve_vision_llm_choice` | VLM — **not** an embedding model; tenant vision before workspace LLM fallback |

### Intentional exception (document, do not blur)

**Extract / Keyword Acc pins** remain **env-first** (`resolve_extract_role_llm` / keyword env) until product unlocks LAW-123-2 for those roles. This is Acc reproducibility, not a silent rewrite of Vision/parser.

## Causal diagram (PDF honesty break)

```ascii
  WHY EdgeParse when UI says Vision?
    → SPEC-038 auto-route fired
  WHY auto-route fired?
    → !backend_explicit && resolved Vision && AUTO_PDF_ROUTING
  WHY !explicit?
    → workspace None + upload omit (+ env unset)
  WHY UI said Vision?
    → labels print resolved default as if inviolable
  WHY wrong?
    → LAW-123-1 / LAW-123-3: resolve promised Vision, run delivered EdgeParse
```

## Causal diagram (model provenance break)

```ascii
  WHY source=workspace when operator cleared overrides?
    → resolve_inherited painted tenant/env into workspace.llm_* / vision_*
  WHY paint?
    → GET responses wanted concrete “effective” strings
  WHY wrong?
    → LAW-123-8: painted values are not deliberate workspace overrides
  FIX
    → Resolver uses metadata / Option unset; UI shows effective via resolve_* + source
```

## Design choices (locked)

```ascii
  PDF:
    choice ∈ { vision, edgeparse, auto }
    runtime ∈ { vision, edgeparse }
    vision|edgeparse → inviolable
    auto            → SPEC-038 may rewrite
    unset           → cascade; never invent Auto

  Models:
    no “vision embedding” type
    VLM ≠ text embedding
    resolve at use; never copy lower layers into request fields
```

## Cross-refs

- WHY: [00-why.md](00-why.md)
- Target: [04-target-architecture.md](04-target-architecture.md)
- Tests: [08-test-protocol.md](08-test-protocol.md)
- Industry: deliberateness / first-wins precedence (config cascades)
