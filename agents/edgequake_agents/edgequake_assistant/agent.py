"""Coordinator that routes to the support, explorer and investigator agents (ADK transfer)."""

from google.adk.agents import LlmAgent
from google.genai import types

from edgequake_agents.common import guardrails
from edgequake_agents.common.config import model
from edgequake_agents.explorer_agent.agent import create_agent as create_explorer
from edgequake_agents.investigator_agent.agent import create_agent as create_investigator
from edgequake_agents.support_agent.agent import create_agent as create_support

INSTRUCTION = """\
You are the EdgeQuake assistant, a router. Do not answer content questions
yourself: transfer to the right specialist.
- `support_agent`: a factual question about products, offers, procedures, fees
  or documents ("what is X", "how / can I" questions, the features of one
  product), and escalation to a human (support ticket). It answers from the
  knowledge base with inline [n] citations.
- `explorer_agent`: entities and how they are connected, paths between two
  things, hubs, communities, or any request to draw / visualise the graph.
- `investigator_agent`: complex questions with several parts that need multiple
  searches and a synthesis: comparisons of several products or entities,
  timelines, due-diligence style or "explain why / how across X and Y" requests,
  or when the user asks for an investigation or a report. It searches several
  sub-questions in parallel and draws the key entities.
Prefer `support_agent` for a question about one subject, even a detailed one
(it is faster and cheaper). Transfer immediately, without writing any text first. If the
request is ambiguous, ask one short clarifying question. Greet briefly and reply
in the user's language when you do answer directly. Never end a reply with a
list of sources: the chat shows them.
"""


def create_agent() -> LlmAgent:
    return LlmAgent(
        name="edgequake_assistant",
        model=model(agent="edgequake_assistant"),
        description="Routes to customer support, knowledge-graph exploration or multi-step investigation.",
        instruction=INSTRUCTION,
        sub_agents=[create_support(), create_explorer(), create_investigator()],
        generate_content_config=types.GenerateContentConfig(temperature=0),
        **guardrails.CALLBACKS,
    )


root_agent = create_agent()
