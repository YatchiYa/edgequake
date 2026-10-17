"""Live smoke tests: real EdgeQuake (MPW workspace) + real LLM.

Skipped unless EDGEQUAKE_API_KEY is set (agents/.env counts). ~15-25 LLM calls total.
    uv run pytest -m live -s
"""

import os

import pytest
from google.adk.runners import InMemoryRunner
from google.genai import types

import edgequake_agents.common.config  # noqa: F401  (loads agents/.env)

pytestmark = [
    pytest.mark.live,
    pytest.mark.skipif(not os.getenv("EDGEQUAKE_API_KEY"), reason="EDGEQUAKE_API_KEY not set"),
]


async def ask(agent, question: str):
    runner = InMemoryRunner(agent=agent, app_name="smoke")
    session = await runner.session_service.create_session(app_name="smoke", user_id="test")
    text, tools, artifacts = [], [], []
    msg = types.Content(role="user", parts=[types.Part(text=question)])
    async for event in runner.run_async(user_id="test", session_id=session.id, new_message=msg):
        tools += [c.name for c in event.get_function_calls()]
        artifacts += list(event.actions.artifact_delta)
        if event.is_final_response() and event.content and event.content.parts:
            text += [p.text for p in event.content.parts if p.text]
    session = await runner.session_service.get_session(app_name="smoke", user_id="test", session_id=session.id)
    print("LLM CALLS:", session.state.get("llm_calls"))
    keys = await runner.artifact_service.list_artifact_keys(app_name="smoke", user_id="test", session_id=session.id)
    await runner.close()
    return "".join(text), tools, artifacts, keys


async def test_support_agent_answers_with_kb():
    from edgequake_agents.support_agent.agent import create_agent

    answer, tools, *_ = await ask(create_agent(), "Quel est le ticket minimum pour investir dans Odyssey 2026 ?")
    print("\nTOOLS:", tools, "\nANSWER:", answer)
    assert answer.strip()
    assert any(t.startswith("eq_") for t in tools)


async def test_explorer_agent_renders_graph():
    from edgequake_agents.explorer_agent.agent import create_agent

    answer, tools, artifacts, keys = await ask(
        create_agent(), "Dessine le graphe autour de Discovery FCPR (profondeur 1, 25 nœuds max)."
    )
    print("\nTOOLS:", tools, "\nARTIFACTS:", artifacts, "\nANSWER:", answer)
    assert "render_graph" in tools
    assert any(k.endswith(".png") for k in keys) and any(k.endswith(".html") for k in keys)
    assert answer.strip()


async def test_investigator_plans_researches_and_reports():
    from edgequake_agents.investigator_agent.agent import create_agent

    answer, tools, artifacts, keys = await ask(
        create_agent(), "Compare Odyssey 2026 et Discovery FCPR : ticket minimum et stratégie d'investissement."
    )
    print("\nTOOLS:", tools, "\nARTIFACTS:", artifacts, "\nANSWER:", answer)
    assert any(t.startswith("eq_") for t in tools)
    assert any(k.startswith("investigation-") and k.endswith(".md") for k in keys)
    assert answer.strip()
