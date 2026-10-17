"""Agents registry for the /v1 product API (SPEC-165): tool catalog, allowed models,
built-in agents (read-only) and the runtime that turns a custom-agent spec into an
LlmAgent."""

from __future__ import annotations

import os
import re
import unicodedata
from dataclasses import dataclass
from datetime import datetime, timezone
from functools import lru_cache
from typing import Any, Callable

from google.adk.agents import LlmAgent
from google.adk.tools import ToolContext
from google.genai import types

from . import guardrails
from .config import model, settings
from .mcp import edgequake_toolset

# ---------------------------------------------------------------- tools


def _slug(text: str, sep: str = "-") -> str:
    text = unicodedata.normalize("NFKD", text).encode("ascii", "ignore").decode()
    return re.sub(r"[^a-z0-9]+", sep, text.lower()).strip(sep)[:40]


async def save_markdown_report(title: str, markdown: str, tool_context: ToolContext) -> dict:
    """Saves a markdown document (report, summary, note) as a downloadable artifact.
    Use it when the user asks for a report, a summary file or a written deliverable.

    Args:
      title: Short document title; it also names the file.
      markdown: Full document body in markdown (cite sources by document title).

    Returns:
      The artifact file name and its version.
    """
    name = f"report-{_slug(title) or 'report'}.md"
    body = markdown if markdown.lstrip().startswith("#") else f"# {title}\n\n{markdown}"
    version = await tool_context.save_artifact(name, types.Part.from_bytes(data=body.encode(), mime_type="text/markdown"))
    return {"status": "ok", "artifact": name, "version": version}


@dataclass(frozen=True)
class Tool:
    id: str
    name: str
    description: str
    category: str  # knowledge | graph | documents | artifacts
    fn: Callable[..., Any] | None = None  # None = EdgeQuake MCP tool with the same name

    def public(self) -> dict:
        return {"id": self.id, "name": self.name, "description": self.description, "category": self.category}


def _python_tools() -> dict[str, Callable]:
    from edgequake_agents.explorer_agent.graph import find_path, graph_overview, render_graph
    from edgequake_agents.support_agent.tickets import open_support_ticket

    from .knowledge import knowledge_context, knowledge_query

    return {f.__name__: f for f in (knowledge_query, knowledge_context, render_graph, find_path, graph_overview,
                                    open_support_ticket, save_markdown_report)}


_PY = _python_tools()
TOOLS: dict[str, Tool] = {t.id: t for t in [
    Tool("knowledge_context", "Retrieve context", "Ranked, numbered passages plus knowledge-graph entities and "
         "relationships for a query, in ~2 s, without generating an answer. First call for factual questions.",
         "knowledge", _PY["knowledge_context"]),
    Tool("knowledge_query", "Ask the knowledge base", "EdgeQuake's full RAG pipeline (same as the Query page): grounded "
         "answer plus its numbered sources. Slower (~10 s): fallback when the passages do not answer.", "knowledge",
         _PY["knowledge_query"]),
    Tool("eq_retrieve", "Retrieve passages", "Hybrid search over the knowledge base, returns cited passages.", "knowledge"),
    Tool("eq_search", "Search", "Search the knowledge base; returns a retrieval id to fetch details.", "knowledge"),
    Tool("eq_fetch", "Fetch results", "Fetch the chunks or citations of a previous search.", "knowledge"),
    Tool("eq_workspace_stats", "Workspace stats", "Document, chunk, entity and relationship counts.", "knowledge"),
    Tool("eq_document_list", "List documents", "List the documents of the workspace.", "documents"),
    Tool("eq_document_get", "Read document", "Read one document (metadata and text).", "documents"),
    Tool("eq_entity_search", "Find entities", "Find knowledge-graph entities by name or keyword.", "graph"),
    Tool("eq_entity_get", "Entity details", "Full description of one entity.", "graph"),
    Tool("eq_neighborhood", "Entity neighbourhood", "Typed neighbours of an entity (1-2 hops).", "graph"),
    Tool("render_graph", "Draw graph", "Draws the neighbourhood of entities as PNG + interactive HTML artifacts.", "graph", _PY["render_graph"]),
    Tool("find_path", "Find path", "Shortest chain of relationships linking two entities, with a drawing.", "graph", _PY["find_path"]),
    Tool("graph_overview", "Graph overview", "Entity/relationship type counts, hubs and communities.", "graph", _PY["graph_overview"]),
    Tool("open_support_ticket", "Support ticket", "Writes a support ticket (markdown artifact) for a human agent.", "artifacts", _PY["open_support_ticket"]),
    Tool("save_markdown_report", "Markdown report", "Saves a markdown report as a downloadable artifact.", "artifacts", _PY["save_markdown_report"]),
]}


@lru_cache(maxsize=None)
def shared_toolset(mcp_tools: tuple[str, ...]):
    """One McpToolset per tool set, shared by all runs: headers are per request
    (header_provider -> ContextVar) and ADK pools MCP sessions per header hash."""
    return edgequake_toolset(list(mcp_tools))


# ---------------------------------------------------------------- models

DEFAULT_ALLOWED = "mistral/mistral-medium-latest,mistral/mistral-small-latest,mistral/mistral-large-latest"


def allowed_models() -> list[dict]:
    ids = [m.strip() for m in os.getenv("EQ_ALLOWED_MODELS", DEFAULT_ALLOWED).split(",") if m.strip()]
    default = settings().model if settings().model in ids else ids[0]
    label = lambda i: i.split("/")[-1].removesuffix("-latest").replace("-", " ").title()  # noqa: E731
    return [{"id": i, "label": label(i), "default": i == default} for i in ids]


def default_model() -> str:
    return next(m["id"] for m in allowed_models() if m["default"])


# ---------------------------------------------------------------- built-in agents

_EPOCH = datetime.now(timezone.utc).isoformat()


def _builtins() -> dict[str, dict]:
    from edgequake_agents.edgequake_assistant import agent as assistant
    from edgequake_agents.explorer_agent import agent as explorer
    from edgequake_agents.investigator_agent import agent as inv
    from edgequake_agents.support_agent import agent as support

    support_tools = ["knowledge_context", "knowledge_query", "eq_document_list", "eq_document_get", "open_support_ticket"]
    explorer_tools = ["eq_entity_search", "eq_entity_get", "eq_neighborhood", "knowledge_context",
                      "eq_workspace_stats", "render_graph", "find_path", "graph_overview"]
    inv_text = "\n\n".join(f"## {role}\n\n{text}" for role, text in [
        ("Planner", inv.PLANNER), ("Retrieval", "One `knowledge_context` search per sub-question, in parallel, no LLM call."),
        ("Synthesizer", inv.SYNTHESIZER)])
    specs = [
        (support.root_agent, "Support", "🎧", support.INSTRUCTION, support_tools, 0.1,
         ["Quel est le ticket minimum pour investir ?", "Quels documents sont disponibles ?",
          "Je voudrais parler à un conseiller."]),
        (explorer.root_agent, "Explorer", "🕸️", explorer.INSTRUCTION, explorer_tools, 0.2,
         ["Qu'y a-t-il dans ce graphe ?", "Quelles sont les entités les plus connectées ?",
          "Dessine le graphe autour d'une entité."]),
        (inv.root_agent, "Investigator", "🔎", inv_text, inv.RESEARCH_TOOLS + ["render_graph", "save_markdown_report"], 0.2,
         ["Compare deux produits : ticket minimum et stratégie.", "Rédige un rapport sur un sujet du corpus."]),
        (assistant.root_agent, "EdgeQuake assistant", "✨", assistant.INSTRUCTION,
         list(dict.fromkeys(support_tools + explorer_tools)), 0.0,
         ["Que peux-tu faire ?", "Quels sont les produits proposés ?", "Montre-moi le graphe de connaissances."]),
    ]
    out = {}
    for agent, label, icon, instructions, tools, temp, starters in specs:
        out[agent.name] = {
            "id": agent.name, "kind": "builtin", "name": label, "description": agent.description, "icon": icon,
            "instructions": instructions, "model": os.getenv(f"EQ_MODEL_{agent.name.upper()}") or settings().model,
            "tools": tools, "starters": starters, "temperature": temp, "owner_id": None, "visibility": "workspace",
            "workspace_id": None, "created_at": _EPOCH, "updated_at": _EPOCH, "editable": False, "_agent": agent,
        }
    # System agents (kind "system"): runnable, hidden from GET /v1/agents unless ?include=system.
    from . import architect

    mcp = tuple(sorted(t for t in architect.READ_TOOLS if TOOLS[t].fn is None))
    out[architect.AGENT_ID] = architect.builtin_spec(shared_toolset(mcp), _EPOCH)
    return out


BUILTINS = _builtins()


def public(spec: dict) -> dict:
    return {k: v for k, v in spec.items() if not k.startswith("_")}


# ---------------------------------------------------------------- runtime


def app_name(agent_id: str) -> str:
    return agent_id if agent_id in BUILTINS else f"custom-{agent_id}"


def agent_name(display_name: str) -> str:
    """LlmAgent.name (the event author) from the display name: a Python identifier."""
    from .v1 import HIDDEN_AUTHORS  # lazy: v1 imports this module

    name = _slug(display_name, "_")
    # "user" is ADK's author for user turns; v1 hides the text of internal authors (the planner).
    return name if name.isidentifier() and name not in {"user", *HIDDEN_AUTHORS} else f"agent_{name}".rstrip("_")


def build_agent(spec: dict):
    """Root agent for a run: the shared built-in tree, or a fresh LlmAgent for a custom spec."""
    if spec["kind"] in ("builtin", "system"):
        return spec["_agent"]
    mcp = tuple(sorted(t for t in spec["tools"] if TOOLS[t].fn is None))
    tools = ([shared_toolset(mcp)] if mcp else []) + [TOOLS[t].fn for t in spec["tools"] if TOOLS[t].fn]
    text = spec["instructions"]
    return LlmAgent(
        name=agent_name(spec["name"]),
        model=model(spec["model"]),
        description=spec["description"] or spec["name"],
        instruction=lambda _ctx: text,  # provider = no {state} templating of user-written text
        tools=tools,
        generate_content_config=types.GenerateContentConfig(temperature=spec["temperature"]),
        **guardrails.CALLBACKS,
    )
