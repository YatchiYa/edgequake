"""Knowledge-graph tools for the explorer agent: subgraph fetch (REST),
PNG + interactive HTML rendering, shortest paths and workspace overview."""

from __future__ import annotations

import asyncio
import html
import io
import json
import re
import unicodedata
from collections import Counter
from typing import Any

import httpx
import networkx as nx
from google.adk.tools import ToolContext
from google.genai import types
from matplotlib.figure import Figure
from matplotlib.lines import Line2D

from edgequake_agents.common import rest

VIS_NETWORK_JS = "https://unpkg.com/vis-network@9.1.9/standalone/umd/vis-network.min.js"
PALETTE = [
    "#4e79a7", "#f28e2b", "#e15759", "#76b7b2", "#59a14f", "#edc948",
    "#b07aa1", "#ff9da7", "#9c755f", "#bab0ac", "#1f77b4", "#8c564b",
]


def pretty(label: str) -> str:
    return label.replace("_", " ")


def _slug(text: str) -> str:
    text = unicodedata.normalize("NFKD", text).encode("ascii", "ignore").decode()
    return re.sub(r"[^a-z0-9]+", "-", text.lower()).strip("-")[:40] or "graph"


# ---------------------------------------------------------------- fetching


def _merge(graph: nx.MultiDiGraph, payload: dict[str, Any]) -> None:
    """Add EdgeQuake /graph-style nodes and edges (both REST shapes) to `graph`."""
    for n in payload.get("nodes", []):
        deg = n.get("degree")
        graph.add_node(
            n["id"],
            label=n.get("label") or n["id"].split("::")[-1],
            type=(n.get("node_type") or n.get("entity_type") or "UNKNOWN").upper(),
            description=(n.get("description") or "").split("<SEP>")[0][:300],
            degree=deg.get("total", 0) if isinstance(deg, dict) else (deg or 0),
        )
    for e in payload.get("edges", []):
        if e["source"] in graph and e["target"] in graph:
            rel = e.get("relationship_type") or e.get("relation_type") or "RELATED_TO"
            if not graph.has_edge(e["source"], e["target"], key=rel):
                graph.add_edge(e["source"], e["target"], key=rel, type=rel,
                               description=(e.get("description") or "")[:200])


async def _subgraph(name: str, depth: int, max_nodes: int) -> dict[str, Any]:
    """GET /graph?start_node=… ; falls back to a label search when the exact
    entity name is unknown (e.g. "odyssey" -> ODYSSEY_2023)."""
    data = await rest.get("/graph", start_node=name, depth=depth, max_nodes=max_nodes)
    if not data.get("nodes"):
        hits = (await rest.get("/graph/nodes/search", q=name, limit=1)).get("nodes", [])
        if hits:
            data = await rest.get("/graph", start_node=hits[0]["label"], depth=depth, max_nodes=max_nodes)
    return data


async def fetch_graph(names: list[str], depth: int, max_nodes: int) -> tuple[nx.MultiDiGraph, list[str], list[str], bool]:
    """Merged subgraph around `names` -> (graph, center node ids, missing names, truncated)."""
    graph = nx.MultiDiGraph()
    centers, missing, truncated = [], [], False
    per_center = max(5, max_nodes // max(1, len(names)))
    for name, data in zip(names, await asyncio.gather(*(_subgraph(n, depth, per_center) for n in names))):
        if not data.get("nodes"):
            missing.append(name)
            continue
        centers.append(data["nodes"][0]["id"])  # EdgeQuake returns the start node first
        truncated |= bool(data.get("is_truncated"))
        _merge(graph, data)
    return graph, centers, missing, truncated


# ---------------------------------------------------------------- rendering


def _colors(graph: nx.MultiDiGraph) -> dict[str, str]:
    types_ = sorted({d["type"] for _, d in graph.nodes(data=True)})
    return {t: PALETTE[i % len(PALETTE)] for i, t in enumerate(types_)}


def render_png(graph: nx.MultiDiGraph, title: str, centers: list[str] = ()) -> bytes:
    colors = _colors(graph)
    n = graph.number_of_nodes()
    size = min(18, 8 + n / 8)
    fig = Figure(figsize=(size, size * 0.75), dpi=110)
    ax = fig.add_subplot()
    ax.set_title(title, fontsize=14)
    ax.axis("off")
    simple = nx.Graph(graph)
    pos = nx.spring_layout(simple, seed=7, k=1.6 / max(1, n) ** 0.5, iterations=200)
    degrees = [graph.nodes[v]["degree"] for v in simple]
    max_deg = max(degrees + [1])
    nx.draw_networkx_edges(simple, pos, ax=ax, edge_color="#999999", alpha=0.5, width=0.8)
    nx.draw_networkx_nodes(
        simple, pos, ax=ax,
        node_color=[colors[graph.nodes[v]["type"]] for v in simple],
        node_size=[250 + 1400 * (d / max_deg) ** 0.5 for d in degrees],
        edgecolors=["#111111" if v in centers else "#ffffff" for v in simple],
        linewidths=[3 if v in centers else 1 for v in simple],
    )
    nx.draw_networkx_labels(
        simple, pos, ax=ax, font_size=9 if n < 40 else 7,
        labels={v: pretty(graph.nodes[v]["label"])[:28] for v in simple},
        bbox={"boxstyle": "round,pad=0.15", "fc": "white", "ec": "none", "alpha": 0.7},
    )
    if graph.number_of_edges() <= 25:
        nx.draw_networkx_edge_labels(
            simple, pos, ax=ax, font_size=7, font_color="#555555",
            edge_labels={(u, v): d["type"] for u, v, d in graph.edges(data=True) if d["type"] != "RELATED_TO"},
        )
    ax.legend(
        handles=[Line2D([0], [0], marker="o", color="w", markerfacecolor=c, markersize=10, label=t)
                 for t, c in colors.items()],
        loc="lower left", fontsize=9, title="Entity type", frameon=True,
    )
    fig.tight_layout()
    buf = io.BytesIO()
    fig.savefig(buf, format="png")
    return buf.getvalue()


def render_html(graph: nx.MultiDiGraph, title: str, centers: list[str] = ()) -> str:
    colors = _colors(graph)
    nodes = [
        {"id": v, "label": pretty(d["label"]), "group": d["type"], "color": colors[d["type"]],
         "value": d["degree"] or 1, "borderWidth": 4 if v in centers else 1,
         "title": f"{pretty(d['label'])} [{d['type']}]\n{d['description']}"}
        for v, d in graph.nodes(data=True)
    ]
    edges = [
        {"from": u, "to": v, "arrows": "to", "title": f"{d['type']}: {d['description']}",
         "label": "" if d["type"] == "RELATED_TO" else d["type"]}
        for u, v, d in graph.edges(data=True)
    ]
    legend = "".join(
        f'<span><i style="background:{c}"></i>{html.escape(t)}</span>' for t, c in colors.items()
    )
    data = json.dumps({"nodes": nodes, "edges": edges}).replace("</", "<\\/")
    return f"""<!doctype html>
<html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>{html.escape(title)}</title>
<script src="{VIS_NETWORK_JS}"></script>
<style>
 :root {{ color-scheme: light dark; }}
 body {{ margin:0; font:14px system-ui,sans-serif; background:Canvas; color:CanvasText; }}
 header {{ padding:8px 12px; display:flex; flex-wrap:wrap; gap:6px 14px; align-items:center; }}
 header b {{ margin-right:8px; }}
 header span {{ display:inline-flex; align-items:center; gap:4px; font-size:12px; }}
 header i {{ width:10px; height:10px; border-radius:50%; display:inline-block; }}
 #g {{ height:calc(100vh - 44px); }}
</style></head><body>
<header><b>{html.escape(title)}</b>{legend}</header>
<div id="g"></div>
<script>
const data = {data};
const dark = matchMedia("(prefers-color-scheme: dark)").matches;
new vis.Network(document.getElementById("g"),
  {{ nodes: new vis.DataSet(data.nodes), edges: new vis.DataSet(data.edges) }},
  {{ nodes: {{ shape: "dot", scaling: {{ min: 8, max: 40 }}, font: {{ color: dark ? "#eee" : "#222" }} }},
     edges: {{ color: {{ color: "#999", opacity: 0.6 }}, font: {{ size: 10, align: "middle" }}, smooth: {{ type: "continuous" }} }},
     physics: {{ stabilization: {{ iterations: 250 }}, barnesHut: {{ springLength: 140 }} }},
     interaction: {{ hover: true, tooltipDelay: 120 }} }});
</script></body></html>"""


async def _save(tool_context: ToolContext, graph: nx.MultiDiGraph, stem: str, title: str, centers: list[str]) -> list[str]:
    png, page = render_png(graph, title, centers), render_html(graph, title, centers)
    names = [f"{stem}.png", f"{stem}.html"]
    await tool_context.save_artifact(names[0], types.Part.from_bytes(data=png, mime_type="image/png"))
    await tool_context.save_artifact(names[1], types.Part.from_bytes(data=page.encode(), mime_type="text/html"))
    return names


def _summary(graph: nx.MultiDiGraph) -> dict[str, Any]:
    top = sorted(graph.nodes(data=True), key=lambda x: -x[1]["degree"])[:8]
    return {
        "nodes": graph.number_of_nodes(),
        "edges": graph.number_of_edges(),
        "entity_types": dict(Counter(d["type"] for _, d in graph.nodes(data=True)).most_common()),
        "relationship_types": dict(Counter(d["type"] for *_, d in graph.edges(data=True)).most_common()),
        "top_entities": [f"{pretty(d['label'])} ({d['type']}, degree {d['degree']})" for _, d in top],
    }


def _http_error(e: httpx.HTTPError) -> dict[str, str]:
    return {"status": "error", "error": f"EdgeQuake request failed: {e}"}


# ---------------------------------------------------------------- tools


async def render_graph(entity_names: list[str], tool_context: ToolContext, depth: int = 1, max_nodes: int = 40) -> dict[str, Any]:
    """Draws the knowledge-graph neighbourhood of one or more entities and saves
    it as two artifacts: a PNG image and an interactive HTML page.

    Args:
      entity_names: Entity names to center the drawing on (e.g. ["Altaroc"] or
        ["Odyssey 2026", "Discovery FCPR"]). Case and spaces don't matter.
      depth: Hops around each entity, 1-3 (default 1).
      max_nodes: Maximum nodes drawn in total, 5-120 (default 40). Keep it
        small for readability.

    Returns:
      Artifact file names plus a summary (node/edge counts, entity types, most
      connected entities, missing names).
    """
    names = [n.strip() for n in entity_names if n and n.strip()][:5]
    if not names:
        return {"status": "error", "error": "entity_names is empty"}
    depth, max_nodes = min(max(depth, 1), 3), min(max(max_nodes, 5), 120)
    try:
        graph, centers, missing, truncated = await fetch_graph(names, depth, max_nodes)
    except httpx.HTTPError as e:
        return _http_error(e)
    if not graph:
        return {"status": "not_found", "missing": missing}
    title = f"{', '.join(names)} — depth {depth}"
    artifacts = await _save(tool_context, graph, f"graph-{_slug('-'.join(names))}", title, centers)
    return {"status": "ok", "artifacts": artifacts, "missing": missing, "truncated": truncated, **_summary(graph)}


async def find_path(source: str, target: str, tool_context: ToolContext) -> dict[str, Any]:
    """Finds the shortest chain of relationships linking two entities (multi-hop)
    and saves a drawing of that path as PNG + interactive HTML artifacts.

    Args:
      source: First entity name.
      target: Second entity name.

    Returns:
      The ordered path (entities and the relationship between each pair) or
      status "no_path".
    """
    # ponytail: unions the 2-hop neighbourhoods (≤500 nodes each) of both ends, so
    # paths up to ~4 hops are found; for longer paths add a server-side path endpoint.
    try:
        graph, centers, missing, _ = await fetch_graph([source, target], depth=2, max_nodes=1000)
    except httpx.HTTPError as e:
        return _http_error(e)
    if missing or len(centers) < 2:
        return {"status": "not_found", "missing": missing}
    try:
        hops = nx.shortest_path(nx.Graph(graph), centers[0], centers[1])
    except nx.NetworkXNoPath:
        return {"status": "no_path", "detail": "No path within about 4 hops."}
    path = graph.subgraph(hops).copy()
    steps = []
    for u, v in zip(hops, hops[1:]):
        rels = graph.get_edge_data(u, v) or graph.get_edge_data(v, u) or {}
        d = next(iter(rels.values()), {"type": "RELATED_TO", "description": ""})
        arrow = "->" if graph.has_edge(u, v) else "<-"
        steps.append(f"{pretty(graph.nodes[u]['label'])} {arrow}[{d['type']}: {d['description']}] {pretty(graph.nodes[v]['label'])}")
    title = f"Path: {pretty(graph.nodes[centers[0]]['label'])} → {pretty(graph.nodes[centers[1]]['label'])}"
    artifacts = await _save(tool_context, path, f"path-{_slug(source)}-{_slug(target)}", title, centers)
    return {"status": "ok", "hops": len(hops) - 1, "path": steps, "artifacts": artifacts}


async def graph_overview(top_n: int = 10) -> dict[str, Any]:
    """Workspace-wide graph overview: counts per entity/relationship type, the
    most connected entities (hubs) and detected communities.

    Args:
      top_n: Number of hub entities to return (default 10, max 50).
    """
    try:
        facets, popular, communities = await asyncio.gather(
            rest.get("/graph/facets"),
            rest.get("/graph/labels/popular", limit=min(max(top_n, 1), 50)),
            rest.get("/graph/communities"),
        )
    except httpx.HTTPError as e:
        return _http_error(e)
    comms = communities.get("communities", [])
    return {
        "entity_types": {f["name"]: f["count"] for f in facets.get("entity_types", [])},
        "relationship_types": {f["name"]: f["count"] for f in facets.get("relationship_types", [])},
        "hubs": [f"{pretty(l['label'])} ({l['entity_type']}, degree {l['degree']['total']})" for l in popular.get("labels", [])],
        "communities": {"count": len(comms), "top": comms[:10]},
    }
