"""Knowledge tools backed by EdgeQuake's full RAG pipeline, the one behind the Query page.

- knowledge_query    POST /api/v1/query          mode "mix" (local + global + naive arms, graph
                     + vectors), reranking, include_references: EdgeQuake's grounded answer
                     plus the passages it used, with document titles.
- knowledge_context  POST /api/v1/query/context  same retrieval, no LLM: ranked passages,
                     entities and relationships.

Both run with the caller's credentials (config.eq_headers, like rest.get). Their `sources`
list is also what v1.translate turns into the SSE `sources` event. The MCP `eq_retrieve`
tool is narrower: plain top-k passages, no graph context and no document id on chunks.
"""

from __future__ import annotations

import logging
import re
import unicodedata
from collections import OrderedDict
from typing import Any

import httpx
from google.adk.tools import ToolContext

from . import rest
from .config import eq_headers

log = logging.getLogger("edgequake_agents")
MODE = "mix"
MAX_PASSAGE = 4000  # characters of each passage handed to the model (EdgeQuake chunks are ~1-3.5k)
TOOL_NAMES = ("knowledge_query", "knowledge_context")

# ponytail: per-process title cache keyed by workspace, filled from the first page (100) of
# GET /documents; ids not on that page show as "document". Page through if a workspace grows past it.
_titles: dict[str, dict[str, str]] = {}


def doc_id_of(chunk_id: str) -> str:
    """EdgeQuake chunk ids are "<document id>-chunk-<n>"."""
    return chunk_id.rsplit("-chunk-", 1)[0] if "-chunk-" in chunk_id else ""


async def titles(doc_ids: set[str], known: dict[str, str] | None = None) -> dict[str, str]:
    cache = _titles.setdefault(eq_headers().get("X-Workspace-ID", ""), {})
    cache.update(known or {})
    if doc_ids - cache.keys():
        try:
            page = await rest.get("/documents", page_size=100)
            cache.update({d["id"]: d.get("title") or d.get("file_name") or "document" for d in page.get("documents", [])})
        except httpx.HTTPError as e:
            log.warning("document titles unavailable: %s", e)
    return cache


def _source(chunk_id: str, document_id: str, names: dict[str, str], text: str, score: Any, ref: Any = None) -> dict:
    s = {"chunk_id": chunk_id, "document_id": document_id, "title": names.get(document_id, "document"),
         "snippet": (text or "").strip()[:MAX_PASSAGE], "score": round(float(score or 0), 4)}
    if ref is not None:
        s["ref"] = ref
    return s


async def run_query(question: str) -> dict:
    r = await rest.post("/query", {"query": question, "mode": MODE, "include_references": True, "enable_rerank": True,
                                   "content_granularity": "agent", "include_subgraph": False})
    srcs = r.get("sources") or []
    chunks = [s for s in srcs if s.get("source_type") == "chunk"]
    names = await titles({s.get("document_id") or doc_id_of(s["id"]) for s in chunks})
    return {
        "status": "ok",
        "answer": r.get("answer", ""),
        "sources": [_source(s["id"], s.get("document_id") or doc_id_of(s["id"]), names, s.get("snippet"),
                            s.get("rerank_score") or s.get("score"), s.get("reference_id")) for s in chunks],
        "entities": [{"name": s.get("id", "").split(":")[-1], "type": s.get("entity_type"), "description": (s.get("snippet") or "")[:300]}
                     for s in srcs if s.get("source_type") == "entity"][:10],
    }


async def run_context(question: str) -> dict:
    r = await rest.post("/query/context", {"query": question, "mode": MODE, "enable_rerank": True,
                                           "include_documents": True, "content_granularity": "agent"})
    b = r.get("bundle") or {}
    chunks = b.get("chunks") or []
    ids = {(c.get("lineage") or {}).get("document_id") or doc_id_of(c["id"]) for c in chunks}
    names = await titles(ids, {d["document_id"]: d["title"] for d in b.get("documents") or [] if d.get("title")})
    g = b.get("subgraph") or {}
    return {
        "status": "ok",
        "sources": [_source(c["id"], (c.get("lineage") or {}).get("document_id") or doc_id_of(c["id"]), names,
                            c.get("content"), c.get("rerank_score") or c.get("score")) for c in chunks],
        "entities": [{"name": e.get("name"), "type": e.get("entity_type"), "description": (e.get("description") or "")[:300]}
                     for e in g.get("entities") or []][:12],
        "relationships": [{"source": x.get("source_label") or x.get("source"), "target": x.get("target_label") or x.get("target"),
                           "type": x.get("relation_type"), "description": (x.get("description") or "")[:200]}
                          for x in g.get("relationships") or []][:12],
    }


def _error(e: Exception) -> dict:
    return {"status": "error", "error": f"EdgeQuake request failed: {e}"}


# ponytail: citation numbers per run kept in memory, keyed by invocation id (parallel tool calls
# of one turn share them); the 256 most recent runs are kept. Move to session state if runs span replicas.
_turn_refs: OrderedDict[str, dict[str, int]] = OrderedDict()


def number_sources(sources: list[dict], invocation_id: str) -> dict:
    """Gives every passage its turn-wide citation number `ref` (same chunk = same number).
    Returns {previous ref: new ref} for answers that cite the previous numbering."""
    refs = _turn_refs.setdefault(invocation_id, {})
    _turn_refs.move_to_end(invocation_id)
    while len(_turn_refs) > 256:
        _turn_refs.popitem(last=False)
    remap = {}
    for s in sources:
        new = refs.setdefault(s["chunk_id"], len(refs) + 1)
        if s.get("ref") is not None:
            remap[s["ref"]] = new
        s["ref"] = new
    return remap


async def knowledge_query(query: str, tool_context: ToolContext) -> dict:
    """Answers a question with EdgeQuake's full RAG pipeline (graph + vectors, reranking,
    grounded LLM answer). Slower (~10 s): use it only when `knowledge_context` did not
    return what the question needs.

    Args:
      query: The user's question, self-contained (resolve "it"/"this" from the
        conversation), in the user's language. Pass the question itself, not keywords.

    Returns:
      `answer`: EdgeQuake's grounded answer, citing passages as [n]; `sources`: those
      passages (`ref` = n, document `title`, text in `snippet`); `entities`.
    """
    try:
        out = await run_query(query)
    except (httpx.HTTPError, KeyError, ValueError) as e:
        return _error(e)
    remap = number_sources(out["sources"], tool_context.invocation_id)
    out["answer"] = re.sub(r"\[(\d+)\]", lambda m: f"[{remap[int(m[1])]}]" if int(m[1]) in remap else "", out["answer"])
    return {**out, "query": query}


async def knowledge_context(query: str, tool_context: ToolContext) -> dict:
    """Retrieves context from the knowledge base in ~2 s, without generating an answer:
    ranked passages plus the knowledge-graph entities and relationships matching the
    query. The first call for any factual question.

    Args:
      query: The user's question (self-contained, user's language), or a short precise
        query for one aspect, e.g. "Odyssey frais de gestion TRI cible".

    Returns:
      `sources`: passages (`ref` = the number to cite as [n], document `title`, text in
      `snippet`, `score`); `entities` (name, type, description); `relationships`
      (source, target, type, description).
    """
    try:
        out = await run_context(query)
    except (httpx.HTTPError, KeyError, ValueError) as e:
        return _error(e)
    number_sources(out["sources"], tool_context.invocation_id)
    return {**out, "query": query}


# ---------------------------------------------------------------- prompt context (support, investigator)


def format_passages(sources: list[dict]) -> str:
    """Numbered passages for a prompt: "[ref] (title)\ntext"."""
    return "\n\n".join(f"[{s['ref']}] ({s['title']})\n{s['snippet']}" for s in sources) or "(no passage found)"


def format_facts(results: list[dict]) -> str:
    """Entities and relationships of one or more context results, deduped, for a prompt."""
    seen, lines = set(), []
    for r in results:
        for e in r.get("entities", []):
            if e["name"] and e["name"] not in seen and e.get("description"):
                seen.add(e["name"])
                lines.append(f"- {e['name']} ({e.get('type') or 'entity'}): {e['description']}")
        for x in r.get("relationships", []):
            key = (x["source"], x["type"], x["target"])
            if key not in seen:
                seen.add(key)
                lines.append(f"- {x['source']} —{x['type']}→ {x['target']}: {x.get('description', '')}")
    return "\n".join(lines[:30]) or "(none)"


# ---------------------------------------------------------------- display (SSE `sources`, history)

EXCERPT = 280
_EMOJI = re.compile("[\U0001F000-\U0001FAFF\u2600-\u27BF\u2B00-\u2BFF\uFE0F\u200D]")
_STOP = set("les des une est que qui quoi pour dans avec sur par aux ces son ses leur the and what are how "
            "est-ce c'est quel quelle quels quelles comment produit".split())


def _fold(text: str) -> str:
    return unicodedata.normalize("NFKD", text.lower()).encode("ascii", "ignore").decode()


def passage_parts(text: str) -> tuple[str, list[str]]:
    """Chunk markdown -> (section path from the leading headings, clean text lines):
    headings, code fences, rules and table separators dropped, table rows joined with
    " · ", markdown emphasis and emoji removed."""
    heads, lines = [], []
    for raw in (text or "").splitlines():
        line = raw.strip()
        if line.startswith("#"):
            if not lines:
                heads.append(line.lstrip("#").strip())
            continue
        if not line or line.startswith("```") or re.fullmatch(r"[-*_=|:\s]{3,}", line):
            continue
        if line.startswith("|"):
            line = " · ".join(c.strip() for c in line.strip("|").split("|") if c.strip())
        line = _EMOJI.sub("", re.sub(r"[*_`>]+|^\s*[-•]\s+", "", line)).strip(" ·")
        if line:
            lines.append(line)
    heads = [_EMOJI.sub("", h).strip() for h in heads]
    return " › ".join((heads[1:] if len(heads) > 1 else heads)[-2:]), lines


def excerpt(lines: list[str], query: str, size: int = EXCERPT) -> str:
    """~`size` chars starting at the sentence that matches most query terms."""
    # (line index, sentence): sentences of different lines are joined with " · "
    sentences = [(n, x) for n, line in enumerate(lines) for x in re.split(r"(?<=[.!?])\s+", line) if x]
    if not sentences:
        return ""
    terms = {w for w in re.findall(r"\w{3,}", _fold(query)) if w not in _STOP}
    best = max(range(len(sentences)), key=lambda i: (sum(t in _fold(sentences[i][1]) for t in terms), -i))
    out, prev = "", None
    for n, x in sentences[best:]:
        if out and len(out) + len(x) > size:
            break
        out = x if not out else f"{out}{' ' if n == prev or out.endswith(':') else ' · '}{x}"
        prev = n
    return out if len(out) <= size else out[: size - 1].rstrip() + "…"


def ui_sources(items: list[dict], query: str = "") -> list[dict]:
    """Sources as sent to the browser: `section` (heading path), `snippet` (clean excerpt
    around the query terms) and `content` (the whole chunk, markdown)."""
    out = []
    for s in items:
        section, lines = passage_parts(s.get("snippet", ""))
        out.append({**{k: s[k] for k in ("document_id", "title", "score", "chunk_id", "ref", "steps") if k in s},
                    "section": section, "snippet": excerpt(lines, query), "content": s.get("snippet", "")})
    return out
