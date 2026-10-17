# SPEC-165 — Agents platform (ChatGPT-style) on Google ADK

## Goal

A professional chat platform for EdgeQuake agents inside the web UI at `/agents`: conversation history per user, agent picker and gallery, streaming answers with tool activity and artifacts, and an **agent builder** to create custom agents (or customize a copy of a built-in) for one's own use case. Same sign-in as EdgeQuake; agents act with the signed-in user's own permissions.

## Architecture

```
Browser ──► Next.js (edgequake_webui)
              /agents/...                 pages (own full-screen layout, AuthGuard)
              /api/agents/[...path]       proxy: verifies the EdgeQuake session (GET /api/v1/auth/me,
                                          fail closed), forwards to the agents service with identity
          ──► agents service (agents/, FastAPI + Google ADK, internal only: http://agents:8765)
              /v1/...                     product API below (requires X-Agents-Internal-Token)
              ADK Runner per request, DatabaseSessionService (SQLite on /data), persistent artifacts
          ──► EdgeQuake API (MCP /mcp + REST) with the USER's bearer token + scope headers
```

The legacy ADK routes (`/list-apps`, `/run_sse`, …) and `agents/ui/index.html` stay available (dev / `agents.<host>` behind basic auth).

## Identity headers (proxy → agents service)

| Header | Meaning |
|---|---|
| `X-Agents-Internal-Token` | Shared secret (`AGENTS_INTERNAL_TOKEN`). Missing/wrong → 401. Constant-time compare |
| `X-EQ-User-Id`, `X-EQ-User-Name`, `X-EQ-User-Role` | From `/auth/me` (never from the browser) |
| `Authorization: Bearer <user access token>` | Forwarded so tools call EdgeQuake **as the user** |
| `X-Tenant-ID`, `X-Workspace-ID` | User's current scope (token is scope-bound; the proxy forwards the browser's values) |

The agents service passes the bearer + scope to every tool call of that request (MCP toolset `header_provider` or a context variable read by the toolset and the REST client). It never stores tokens. Read-only tools only (guardrails stay).

## Product API (`/v1`, JSON unless noted)

### Agents

`Agent` = `{id, kind: "builtin"|"custom"|"system", name, description, icon (emoji), instructions, model, tools: [tool_id], starters: [string], temperature, owner_id|null, visibility: "private"|"workspace", workspace_id|null, created_at, updated_at, editable: bool}`

- `GET /v1/agents[?include=system]` → `{items: Agent[]}` — built-ins (support_agent, explorer_agent, investigator_agent, edgequake_assistant; `instructions` included read-only, `editable:false`) + the caller's private customs + `workspace` customs of the caller's current workspace. System agents (`kind:"system"`: `agent_architect`) are listed only with `?include=system`; `GET /v1/agents/agent_architect` always works; duplicating a system agent → 400.
- `GET /v1/agents/{id}` → `Agent`
- `POST /v1/agents` body `{name, description?, icon?, instructions, model, tools, starters?, temperature?, visibility?}` → 201 `Agent` (owner = caller; workspace_id = current workspace when visibility=workspace). Validation: name 1–60, instructions ≤ 20 000 chars, tools ⊆ catalog, model ∈ `/v1/models`, ≤ 4 starters (≤ 200 chars), temperature 0–1.
- `PUT /v1/agents/{id}` (owner only, custom only) → `Agent`; `DELETE /v1/agents/{id}` → 204.
- `POST /v1/agents/{id}/duplicate` → 201 `Agent` (custom copy, editable, name "… (copy)"; works for built-ins: copies instructions/tools/model).
- `GET /v1/tools` → `{items: [{id, name, description, category: "knowledge"|"graph"|"documents"|"artifacts"}]}` — e.g. `eq_retrieve, eq_search, eq_fetch, eq_document_list, eq_document_get, eq_entity_search, eq_entity_get, eq_neighborhood, eq_workspace_stats, render_graph, find_path, graph_overview, open_support_ticket, save_markdown_report`.
- `GET /v1/models` → `{items: [{id, label, default: bool}]}` from `EQ_ALLOWED_MODELS` (comma list, default `mistral/mistral-medium-latest,mistral/mistral-small-latest,mistral/mistral-large-latest`).

### Conversations (per user and per workspace)

`Conversation` = `{id, agent_id, title, created_at, updated_at}`; `Message` = `{id, role: "user"|"assistant", author (first agent of the turn), agents? (every agent of the turn, in order), content (markdown), tools: [{id, name, status: "done"|"error"}], artifacts: [{name, mime_type, version}], created_at, agent_spec?: {spec, rationale, tests}, plan?: PlanItem[], steps?: Step[], sources?: Source[]}` (`agent_spec` only on architect turns that proposed a spec; `plan`/`steps`/`sources` = the last value of the matching stream events, see below). **One assistant message per user turn**: a routing hop (assistant → specialist) and internal sub-agents (planner, synthesizer) all fold into it; the planner's structured JSON is never part of `content`.

**Workspace isolation.** A conversation stores the `X-Tenant-ID` + `X-Workspace-ID` of its creation. Listing returns only the caller's conversations of the current workspace; GET/PATCH/DELETE/messages/artifacts on a conversation of another workspace (or another user) → **404** (never 403: no existence leak). Migration: rows from before scoping (workspace `NULL`) are claimed by the first workspace their owner lists or opens them from, then stay there (only ever the owner's own rows; nothing becomes visible to another user). Custom agents keep their rule: private = owner only (any workspace), workspace = members of that workspace.

- `GET /v1/conversations?agent_id=&q=&limit=50` → `{items: Conversation[]}` newest first (`q` = case-insensitive title match).
- `POST /v1/conversations` body `{agent_id}` → 201 `Conversation` (title "New chat").
- `GET /v1/conversations/{id}` → `Conversation & {messages: Message[]}` (rebuilt from the ADK session events).
- `PATCH /v1/conversations/{id}` body `{title}` → `Conversation`; `DELETE /v1/conversations/{id}` → 204.
- `POST /v1/conversations/{id}/messages` body `{content, context?: {draft_agent?: Partial<AgentInput>, editing_agent_id?: string}}` → **`text/event-stream`**, one JSON object per `data:` line. `context` is used by `agent_architect` only (ignored otherwise): unknown fields dropped, strings capped (instructions 20 000, others ≤ 1 000), ≤ 20 tools/starters, temperature clamped; `editing_agent_id` must be visible (404); the whole body stays ≤ 32 KB (413). It reaches the model as a separate data part of the user turn (`[builder context]`), never as instructions, and is hidden from the history.
  - `{"type":"message_start","author":"explorer_agent"}` (an agent starts contributing to the turn; clients keep ONE assistant message per turn and record the author)
  - `{"type":"delta","author":"…","text":"…"}` (token stream)
  - `{"type":"tool_start","id":"…","author":"…","name":"render_graph","args":{…}}`
  - `{"type":"tool_end","id":"…","ok":true}`
  - `{"type":"artifact","name":"graph-x.png","mime_type":"image/png","version":0}`
  - `{"type":"agent_spec","spec":{name, icon, description, instructions, model, tools, starters, temperature, visibility},"rationale":"…","tests":["…"]}` (agent architect: right after the `tool_end` of a successful `propose_agent_spec`; `spec` passed the `POST /v1/agents` validation)
  - `{"type":"plan","items":[{"id":"r1","label":"sub-question"}, …]}` (investigator: the searches it will run)
  - `{"type":"step","id":"plan|r1…rN|synth|graph","label":"…","status":"running|done|error","detail?":"…","sources_count?":12,"elapsed_ms?":5360}` (progress; later events for the same `id` update it)
  - `{"type":"sources","items":[{"document_id","title","section","snippet","content","score","chunk_id?","ref?","steps?":["r1"]}]}` (passages the answer is grounded on: emitted by Support and the investigator after their searches and after every successful `knowledge_query` / `knowledge_context` call of any agent; `ref` = the `[n]` cited inline in the answer, numbered once per turn across all searches; `section` = heading path; `snippet` = clean excerpt ≤ 280 chars around the query terms (no headings, table pipes, code fences, emoji); `content` = the whole chunk, markdown; clients merge by `chunk_id` + `document_id`). Agents cite `[n]` inline and never end with a textual sources list.
  - `{"type":"title","title":"…"}` (after the first exchange: first user message trimmed to 60 chars; architect conversations: `Conception : <spec name>` after each turn that proposed a spec)
  - `{"type":"error","message":"…"}` · `{"type":"done"}`
  Client disconnect cancels the run. Max request 32 KB. Per-user rate limit (20 runs/min) and the per-session LLM budget apply.
- `GET /v1/conversations/{id}/artifacts/{name}?version=N` → raw bytes with the stored `Content-Type` (owner only).

Errors: `{error: "message"}` with 400/401/403/404/409/413/429.

## UI (Next.js, `/agents`)

- `/agents` — new chat: centered greeting "Par où commencer ?"-style, agent picker (current agent chip + dropdown), the agent's starters as cards, composer. First send creates the conversation and navigates to `/agents/c/[id]`.
- `/agents/c/[id]` — conversation: messages (user bubbles right; ONE assistant message per turn, full-width markdown with avatar = the conversation's agent; a placeholder reply appears as soon as the message is sent), ONE compact **activity line** per turn (ChatGPT-style, no box): while running, the current step or tool in shimmering text, or a status phrase when the model is working with nothing to report ("Réflexion", "Rédaction de la réponse"; for the architect "Analyse de votre besoin" → "Conception de l’agent" → "Rédaction des instructions" → "Finalisation de la spécification", advancing every 6 s); once done, a one-line summary "🎧 Support · 20 passages consultés · N outils utilisés · 1 s" (hand-off target, passages, tools, time); click to unfold the timeline of steps and tools (per-step passage counts focus the sources), a **Sources** section from `sources` shown once the answer is complete (document chips with counts, documents ordered by their best passage → expandable passages: section breadcrumb, clean excerpt, relevance in % of the best passage of the turn with a bar, `[n]`, "Voir le passage complet" rendering the whole chunk as markdown, link to `/documents/{id}`), a floating **Investigation panel** (bottom-right above the composer, bottom sheet on phones) for turns with a `plan`: checklist with live status and elapsed time, per-step counts (click → Sources filtered to that step), "Voir le graphe"; it folds into a pill once the answer streams and can be reopened. Architect turns end with an **"Agent proposé" announcement card** (icon, name, description, tool and starter counts); while the architect works, the builder's specification pane shows a skeleton ("L’architecte prépare la spécification…") or, for a revision, a "Révision en cours — version N" banner over the current spec. Artifact cards (image with lightbox + download, sandboxed interactive HTML iframe with "open full screen", markdown rendered), copy button, stop button while streaming, auto-scroll with "jump to latest".
- Left sidebar (ChatGPT-like, collapsible, mobile drawer): **New chat**, search conversations, "Agents" (Explore + up to 5 recent/pinned agents), conversation list grouped Today / Yesterday / Previous 7 days / Older with rename + delete (dropdown), bottom: user + link back to EdgeQuake dashboard, current workspace selector reuse.
- `/agents/explore` — gallery: built-in and "My agents" cards (icon, name, description, Start chat, Duplicate & customize, Edit for owned).
- `/agents/new`, `/agents/[agentId]/edit` — builder (see *Agent architect* below): left form (icon emoji picker, name, description, instructions with templates, model select, tools grouped checkboxes with descriptions, starters, temperature slider, visibility), right **live preview chat** (uses a real temporary conversation with the unsaved config? → Save first, then preview; preview uses the saved agent).
- Fully i18n (fr/en/zh), dark/light, keyboard: Ctrl+Shift+O new chat, Enter send, Shift+Enter newline, Esc stop.

## Agent architect (AI builder)

- **Backend:** system agent `agent_architect` (`agents/edgequake_agents/common/architect.py`): interviews briefly (goal, audience, tone, language, sources, do/don't, output format, example questions), inspects the current workspace read-only (`eq_workspace_stats`, `graph_overview`, `knowledge_context`, `eq_entity_search`), then calls `propose_agent_spec(spec, rationale, tests)` with a complete spec (instructions structured as Role / Scope / Method / Tool usage / Citations / Refusals & escalation / Output format / Language; model ∈ allowed list; tools ⊆ catalog; ≤ 4 starters; 2–3 test questions). Catalog ids + descriptions and the allowed models are injected into its instruction at run time. The tool runs the `POST /v1/agents` validation and returns the errors to the model so it self-corrects. Later turns revise the spec from feedback (including a test transcript).
- **UI:** two tabs, **Créer avec l'IA** (default on `/agents/new`) and **Configurer** (default on edit; `?mode=ai|manual`). AI mode: the architect chat (normal per-user conversation, `?design=<conversation id>` resumes it; `/agents/c/<id>` of an architect conversation has "Ouvrir dans le concepteur") next to a live spec card (fields, tools as chips, starters, rationale, tests; changed fields highlighted vs the previous version, added / removed tools) with **Appliquer au formulaire**, **Créer l'agent** / **Mettre à jour**, then **Tester** (preview chat with the test questions as chips) and **Améliorer** (sends the test transcript to the architect). Each architect message carries the form state as `context`.
- **Manual form:** sections with help texts (identity, behavior, tools, starters, model & settings, sharing), structured instruction templates (blank skeleton, research, graph, support; confirm before replacing), live counters, tool presets "recommandé pour…", starters add/remove, inline validation errors, unsaved-changes guard (reload/close + in-app links), sticky save bar, preview pane always visible on desktop.
- **Workspace switch:** the shell (outside `TenantGuard`) resets cached conversations, leaves `/agents/c/<id>` for `/agents` and drops `?design=`; lists are keyed by workspace and refetch.

## Knowledge tools and the investigator pipeline (2026-10-17)

- **`knowledge_query`** (`agents/edgequake_agents/common/knowledge.py`): `POST /api/v1/query` with `mode: "mix"`, `enable_rerank: true`, `include_references: true`, `content_granularity: "agent"` — the pipeline of the Query page (graph local/global arms + vectors, reranking, grounded LLM answer). Returns `{answer, sources: [{chunk_id, document_id, title, snippet (≤ 4000 chars, for the model), score, ref}], entities}`. **`knowledge_context`**: `POST /api/v1/query/context` (same retrieval, no answer LLM) → `{sources, entities, relationships}`. Both use the caller's bearer + scope (like the REST graph tools). Chunk ids are `<document id>-chunk-<n>`; titles come from a per-workspace cache of `GET /documents`.
- They are catalog tools (category `knowledge`, selectable by custom agents and the architect; `knowledge_context` first, `knowledge_query` as fallback), the evidence tool of `explorer_agent` and the retrieval of the investigator. Citation numbers (`ref`) are shared by all knowledge calls of a run (keyed by invocation id); `knowledge_query` remaps EdgeQuake's own `[n]` in its answer.
- **Support** = custom `BaseAgent` `support_agent`: one `/query/context` on the latest user message (a short follow-up is prefixed with the previous one; under 3 words: no search), a `step` + `sources` event, then ONE LLM call (`support_writer`) answering from the numbered passages and graph facts injected into its instruction. It still has `knowledge_context`, `knowledge_query` (fallback), the document tools and `open_support_ticket` for follow-ups.
- **Investigator** = custom ADK `BaseAgent`: planner (LLM, `output_schema`) → one `knowledge_context` per sub-question, concurrently, **no LLM call** → one synthesizer LLM call streamed token by token, grounded on the numbered merged passages (≤ 12, deduped by chunk) and the graph facts, inline `[n]` citations → `render_graph` on the planner's key entities (REST + networkx, no LLM) → `investigation-<slug>.md`. 2 LLM calls per question whatever the number of sub-questions (was `1 + 2N + 2..4`). No critic loop (an optional single-pass check is left out until it proves cheap enough). UI events ride on `Event.custom_metadata["eq_ui"]`, so they are persisted with the session and replayed into `GET /v1/conversations/{id}`.

## Out of scope (v1)

File uploads in chat, sharing conversations publicly, voice.

## Status (2026-10-17) — implemented and deployed

- **Agents service** (`agents/`): `/v1` router in `edgequake_agents/common/v1.py`, catalog/built-ins/custom-agent builder in `catalog.py`, SQLite store in `store.py` (`agents.db`: custom agents, conversations, persisted OAuth client), ADK `SqliteSessionService` (`sessions.db`) and the ADK file artifact store under `EQ_DATA_DIR` (`/data` volume in production). 37 offline tests + 2 live tests.
- **Per-user credentials:** EdgeQuake's `/mcp` only accepts API keys or MCP-audience tokens, so the service exchanges the user's web JWT for a read+query MCP token through EdgeQuake's own OAuth 2.1 flow (PKCE, consent auto-approved with the user's bearer; one OAuth client registered once and reused). REST tools use the user's bearer directly. Headers live in a per-run `ContextVar`; tokens are never persisted or logged.
- **Web UI** (`edgequake_webui/src/app/agents/**`, `src/components/agents/**`, proxy `src/app/api/agents/[...path]/route.ts` + `src/lib/server/agents-proxy.ts`): ChatGPT-style shell, explore gallery, builder with live preview; identity only from `/auth/me`; browser `X-EQ-*` headers dropped; 13 unit tests.
- **Deployment:** Caddy routes `/api/agents*` to the frontend (unbuffered); secret `edgequake-agents-internal-token` shared by frontend and agents containers; agents data on the `agents-data` volume.

**2026-10-17 (later): agent architect + workspace-scoped conversations** — see the sections above. Agents service 44 offline tests (architect hidden from listing, `propose_agent_spec` validation, SSE `agent_spec` with a fake LLM, `context` validation/capping, workspace 404s, legacy-row claim, in-place schema migration); web UI vitest for the spec diff, the `agent_spec` reducer and the test transcript. Live: one architect design on MPW (2 turns, workspace-tailored spec, 7 Mistral calls) and a headless browser run of the AI builder (apply, create, test, improve: 5 more calls).

**2026-10-17 (UX v2): answer quality + one-message turns.** Root cause of the poor Support answers: `eq_retrieve` (MCP) returned 3 chunks, none from the product sheet (`altaroc_produits.md`), with empty `document_id`s; its top chunk ("Odyssey 2026 SLP : documents disponibles sur demande") became the answer. Now `knowledge_query`/`knowledge_context` (above). Measured on MPW, "c'est quoi le produit altaroc odyssey 2026 ?" (8 key facts: SLP, 80/20, 100 000 €, 10 %/semestre, TRI 13 %, 1,7x, frais 2,5 %→1,65 %, carried 20 % co-invest): direct `POST /query` 6/8 in 12.9 s; Support before 3/8 (6.7 s, 2 calls) → after 8/8 (17.4 s, 2 calls); Assistant after 8/8 (10.4 s, 3 calls); Investigator before 3/8 (22.8 s, 9 calls, first answer token 19.1 s) → after 8/8 (11.3 s, 2 calls, first token 5.6 s). Agents service 46 offline tests; web vitest for the plan/step/sources reducer and snippet parsing.

**2026-10-17 (UX v3): Support context-first, clean sources.** Same question: Support 8/8 facts, **first token 4.5 s, total 6.8 s, 1 LLM call** (was 14.3 s / 17.4 s with `knowledge_query` first); Investigator 8/8, first token 5.9 s, total 12.3 s, 2 calls; no textual sources list in either answer. 48 offline tests (Support pipeline: retrieval before its single LLM call, sources before deltas; turn-wide citation numbers; excerpt cleaning).

Deviations from the contract above: `SqliteSessionService` instead of `DatabaseSessionService` (no sqlalchemy dependency); extra `409` when a reply is already running; non-message bodies capped at 256 KB; built-in `name` is a display label while `id`/stream `author` stay the ADK names; custom agents stream `author` = slug of their name.
