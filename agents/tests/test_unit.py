"""Offline unit tests (no EdgeQuake, no LLM)."""

import pytest

from edgequake_agents.common import config
from edgequake_agents.common.mcp import edgequake_toolset, pin_workspace
from edgequake_agents.explorer_agent import graph as g

FAKE = {
    "nodes": [
        {"id": "ws::ALTAROC", "label": "ALTAROC", "node_type": "ORGANIZATION", "description": "PE firm<SEP>dup", "degree": {"total": 3}},
        {"id": "ws::ODYSSEY_2026", "label": "ODYSSEY_2026", "node_type": "PRODUCT", "description": "Fund </script>", "degree": {"total": 2}},
        {"id": "ws::FREDERIC_STOLAR", "label": "FREDERIC_STOLAR", "entity_type": "PERSON", "degree": {"total": 1}},
    ],
    "edges": [
        {"source": "ws::ODYSSEY_2026", "target": "ws::ALTAROC", "relationship_type": "PART_OF", "description": "fund of"},
        {"source": "ws::FREDERIC_STOLAR", "target": "ws::ALTAROC", "relation_type": "RELATED_TO"},
        {"source": "ws::ALTAROC", "target": "ws::MISSING"},  # dangling edge is dropped
    ],
}


class FakeCtx:
    def __init__(self):
        self.saved = {}

    async def save_artifact(self, filename, artifact):
        self.saved[filename] = artifact
        return 0


@pytest.fixture
def fake_rest(monkeypatch):
    async def get(path, **params):
        if path == "/graph":
            return FAKE if params["start_node"].upper().replace(" ", "_") in {"ALTAROC", "ODYSSEY_2026"} else {"nodes": [], "edges": []}
        return {"nodes": []}  # /graph/nodes/search: no fuzzy match

    monkeypatch.setattr(g.rest, "get", get)


def built():
    graph = g.nx.MultiDiGraph()
    g._merge(graph, FAKE)
    return graph


def test_merge_normalises_both_rest_shapes():
    graph = built()
    assert graph.number_of_nodes() == 3 and graph.number_of_edges() == 2
    assert graph.nodes["ws::FREDERIC_STOLAR"]["type"] == "PERSON"
    assert graph.nodes["ws::ALTAROC"]["description"] == "PE firm"


def test_render_png_is_valid_png():
    png = g.render_png(built(), "t", ["ws::ALTAROC"])
    assert png.startswith(b"\x89PNG\r\n\x1a\n") and len(png) > 5000


def test_render_html_contains_nodes_and_escapes_script():
    page = g.render_html(built(), "Altaroc <graph>", ["ws::ALTAROC"])
    assert "ODYSSEY 2026" in page and "FREDERIC STOLAR" in page and "PART_OF" in page
    assert g.VIS_NETWORK_JS in page
    assert "Fund </script>" not in page and "Altaroc &lt;graph&gt;" in page


async def test_render_graph_saves_png_and_html(fake_rest):
    ctx = FakeCtx()
    out = await g.render_graph(["Altaroc", "nobody"], ctx, depth=9, max_nodes=1)
    assert out["status"] == "ok" and out["missing"] == ["nobody"]
    assert out["artifacts"] == ["graph-altaroc-nobody.png", "graph-altaroc-nobody.html"]
    assert ctx.saved["graph-altaroc-nobody.png"].inline_data.mime_type == "image/png"
    assert ctx.saved["graph-altaroc-nobody.html"].inline_data.mime_type == "text/html"
    assert out["nodes"] == 3 and out["entity_types"]["ORGANIZATION"] == 1


async def test_render_graph_not_found(fake_rest):
    ctx = FakeCtx()
    assert (await g.render_graph(["nobody"], ctx))["status"] == "not_found"
    assert (await g.render_graph([], ctx))["status"] == "error"
    assert ctx.saved == {}


async def test_find_path(fake_rest):
    ctx = FakeCtx()
    out = await g.find_path("Altaroc", "Odyssey 2026", ctx)
    assert out["status"] == "ok" and out["hops"] >= 0 and len(ctx.saved) == 2


def test_settings_and_headers(monkeypatch):
    monkeypatch.setenv("EDGEQUAKE_URL", "http://eq:8090/")
    monkeypatch.setenv("EDGEQUAKE_API_KEY", "k")
    monkeypatch.setenv("EDGEQUAKE_TENANT_ID", "t")
    monkeypatch.setenv("EDGEQUAKE_WORKSPACE_ID", "w")
    s = config.settings()
    assert s.url == "http://eq:8090"
    assert s.headers == {"Authorization": "Bearer k", "X-Tenant-ID": "t", "X-Workspace-ID": "w"}


def test_model_selection(monkeypatch):
    from google.adk.models.lite_llm import LiteLlm

    monkeypatch.delenv("EQ_AGENT_MODEL", raising=False)
    m = config.model()
    assert isinstance(m, LiteLlm) and m.model == config.DEFAULT_MODEL
    monkeypatch.setenv("EQ_AGENT_MODEL", "gemini-2.5-flash")
    assert config.model() == "gemini-2.5-flash"  # native ADK Gemini (AI Studio / Vertex)
    assert isinstance(config.model("vertex_ai/gemini-2.5-pro"), LiteLlm)


def test_toolset_rejects_write_tools():
    with pytest.raises(ValueError):
        edgequake_toolset(["eq_search", "eq_document_delete"])
    edgequake_toolset(["eq_search"])  # lazy: no network until tools are listed


def test_pin_workspace_drops_model_supplied_workspace():
    args = {"query": "x", "workspace_id": "other"}
    pin_workspace(None, args, None)
    assert args == {"query": "x"}


# ---------------------------------------------------------------- guardrails + investigator

from types import SimpleNamespace

from edgequake_agents.common import guardrails


class FakeCallbackCtx(FakeCtx):
    agent_name = "test"

    def __init__(self, state=None):
        super().__init__()
        self.state = state or {}


def test_guard_tool_blocks_write_tools_and_pins_workspace():
    ctx = FakeCallbackCtx()
    for name in ("eq_document_delete", "eq_ingest", "edgequake_workspace_delete", "eq_unknown_new_tool"):
        assert guardrails.guard_tool(SimpleNamespace(name=name), {}, ctx)["status"] == "error"
    args = {"query": "x", "workspace_id": "other"}
    assert guardrails.guard_tool(SimpleNamespace(name="eq_retrieve"), args, ctx) is None
    assert args == {"query": "x"}
    assert guardrails.guard_tool(SimpleNamespace(name="render_graph"), {"entity_names": ["a"]}, ctx) is None


def test_llm_budget_per_session(monkeypatch):
    monkeypatch.setenv("EQ_SESSION_LLM_BUDGET", "2")
    ctx = FakeCallbackCtx()
    assert guardrails.count_llm_call(ctx, None) is None and guardrails.count_llm_call(ctx, None) is None
    stop = guardrails.count_llm_call(ctx, None)
    assert ctx.state["llm_calls"] == 2 and "budget" in stop.content.parts[0].text


def all_agents(agent):
    yield agent
    for sub in agent.sub_agents:
        yield from all_agents(sub)


def test_investigator_tree_and_coordinator_wiring():
    from edgequake_agents.edgequake_assistant.agent import create_agent as coordinator
    from edgequake_agents.investigator_agent import agent as inv

    root = inv.create_agent()
    assert [a.name for a in root.sub_agents] == ["planner", "synthesizer"]  # retrieval is code, not an LLM
    assert root.sub_agents[0].output_key == "plan" and root.sub_agents[1].output_key == "answer"
    assert root.sub_agents[1].include_contents == "none"
    tree = [a.name for a in all_agents(coordinator())]
    assert len(tree) == len(set(tree))
    assert {"support_agent", "explorer_agent", "investigator_agent"} <= set(tree)


def test_merge_sources_dedupes_and_tags_steps():
    from edgequake_agents.investigator_agent import agent as inv

    a = {"chunk_id": "d1-chunk-0", "document_id": "d1", "title": "a.md", "snippet": "x", "score": 0.4}
    b = {"chunk_id": "d2-chunk-3", "document_id": "d2", "title": "b.md", "snippet": "y", "score": 0.9}
    merged = inv.merge_sources([{"sources": [a]}, {"sources": [{**a, "score": 0.7}, b]}, {"status": "error"}])
    assert [(s["chunk_id"], s["ref"], s["steps"], s["score"]) for s in merged] == [
        ("d2-chunk-3", 1, ["r2"], 0.9), ("d1-chunk-0", 2, ["r1", "r2"], 0.7)]


def test_adk_lists_agents_and_loads_yaml_agent():
    from pathlib import Path

    from google.adk.cli.utils.agent_loader import AgentLoader

    loader = AgentLoader(str(Path(__file__).parents[1] / "edgequake_agents"))
    assert loader.list_agents() == ["edgequake_assistant", "explorer_agent", "faq_agent",
                                    "investigator_agent", "support_agent"]  # not common/, not templates/
    faq = loader.load_agent("faq_agent")
    assert faq.name == "faq_agent" and guardrails.guard_tool in faq.before_tool_callback
