"""Support-ticket artifact (markdown) for the support agent."""

from __future__ import annotations

import uuid
from datetime import datetime, timezone

from google.adk.tools import ToolContext
from google.genai import types

PRIORITIES = ("low", "normal", "high", "urgent")


async def open_support_ticket(
    subject: str,
    customer_question: str,
    summary: str,
    tool_context: ToolContext,
    sources: list[str] | None = None,
    priority: str = "normal",
) -> dict:
    """Writes a support-ticket summary as a markdown artifact for a human agent.
    Only call it when the user agrees to escalate or explicitly asks for a ticket.

    Args:
      subject: One-line ticket title.
      customer_question: The customer's request, in their own words.
      summary: What was already answered (with sources) and what remains open.
      sources: Document titles consulted.
      priority: One of low, normal, high, urgent.

    Returns:
      The ticket id and the artifact file name.
    """
    ticket_id = f"TCK-{uuid.uuid4().hex[:8].upper()}"
    priority = priority if priority in PRIORITIES else "normal"
    now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    sources_md = "\n".join(f"- {s}" for s in sources or []) or "- (none)"
    body = (
        f"# {ticket_id} — {subject}\n\n"
        f"| Field | Value |\n|---|---|\n| Priority | {priority} |\n| Created | {now} |\n"
        f"| Session | {tool_context.session.id} |\n| Status | open — awaiting human agent |\n\n"
        f"## Customer request\n\n{customer_question}\n\n"
        f"## Summary\n\n{summary}\n\n"
        f"## Sources consulted\n\n{sources_md}\n"
    )
    filename = f"ticket-{ticket_id}.md"
    await tool_context.save_artifact(filename, types.Part.from_bytes(data=body.encode(), mime_type="text/markdown"))
    return {"status": "ok", "ticket_id": ticket_id, "artifact": filename}
