---
title: 'EdgeQuake Feature Registry'
---

# EdgeQuake Feature Registry

This file maintains traceability between code features and business requirements.

## Index

| Feature ID | Description                                        | Status    | Spec / Issue         |
| ---------- | -------------------------------------------------- | --------- | -------------------- |
| FEAT-0001  | Tenant Workspace Quota Management                  | Completed | SPEC-0001 / #133     |
| FEAT-0002  | Knowledge Injection (Glossaries & Synonyms)        | Completed | SPEC-0002 / #131     |
| FEAT-0003  | Explainability                                     | Planned   | SPEC-0003 / #128     |
| FEAT-0004  | Graph Edge Labels                                  | Planned   | SPEC-0004 / #91      |
| FEAT-0005  | Custom Entity Configuration                        | Planned   | SPEC-0005 / #85      |
| FEAT-006   | Unified Streaming Response Protocol                | Completed | SPEC-006 / #56       |
| FEAT-007   | Vector Storage SQL Pre-Filtering                   | Completed | SPEC-007             |

## Feature Definitions

### FEAT-0002 — Knowledge Injection

**Issue**: [#131](https://github.com/raphaelmansuy/edgequake/issues/131)  
**Spec**: [specifications/0002_knowledge_injection_issue_131](../specifications/0002_knowledge_injection_issue_131/)  
**Released**: v0.8.0 (2026-04-03)  
**Status**: ✅ Completed

**Problem**: Domain-specific acronyms (OEE, NLP) and synonyms are unknown to the embedding model. Queries for "OEE" miss documents that say "Overall Equipment Effectiveness", degrading retrieval quality.

**Solution**: Workspace owners inject glossary definitions as named entries. These are processed through the standard entity-extraction pipeline, enriching the knowledge graph. At query time, injection entities expand the query terms. Injection entries are **never shown as source citations**.

**API Surface**:
- `PUT /api/v1/workspaces/:id/injection` — create/replace text injection
- `POST /api/v1/workspaces/:id/injection/upload` — upload file injection
- `GET /api/v1/workspaces/:id/injection` — list all entries
- `GET /api/v1/workspaces/:id/injection/:injection_id` — get detail
- `PATCH /api/v1/workspaces/:id/injection/:injection_id` — update name/content
- `DELETE /api/v1/workspaces/:id/injection/:injection_id` — delete + cascade cleanup

**UI**: `/knowledge` page with list, add dialog (text/file tabs), detail page, inline edit, delete confirmation.

**Test Coverage**: 1 000+ line Rust E2E suite + 5 Playwright browser tests.

---

**Last Updated**: 2026-04-03
**Total Features**: 7
