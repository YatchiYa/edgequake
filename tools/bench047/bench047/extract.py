"""Short-answer extraction (official GPT-4o or Mistral judge)."""

from __future__ import annotations

import os
from typing import Optional

EXTRACT_PROMPT = """You are an answer extractor. Given a question and a long model analysis, extract the short final answer only.
Rules:
- Output ONLY the short answer (or "Not answerable" if the analysis concludes it cannot be answered from the document).
- Preserve numbers, units, and spelling carefully.
- Do not explain.
"""


def extract_answer(
    question: str,
    long_answer: str,
    *,
    extractor: str = "gpt-4o",
    model: Optional[str] = None,
) -> str:
    extractor = (extractor or "gpt-4o").lower()
    if extractor in {"gpt-4o", "openai", "official"}:
        return _openai_extract(question, long_answer, model or "gpt-4o")
    if extractor in {"mistral", "mistral-small", "mistral-small-latest", "mistral-large", "mistral-large-latest"}:
        return _mistral_extract(question, long_answer, model or "mistral-small-latest")
    raise ValueError(f"Unknown extractor: {extractor}")


def _openai_extract(question: str, long_answer: str, model: str) -> str:
    from openai import OpenAI

    key = os.environ.get("OPENAI_API_KEY")
    if not key:
        raise RuntimeError("OPENAI_API_KEY required for official extractor")
    client = OpenAI(api_key=key)
    resp = client.chat.completions.create(
        model=model,
        messages=[
            {"role": "system", "content": EXTRACT_PROMPT},
            {
                "role": "user",
                "content": f"Question: {question}\n\nAnalysis:\n{long_answer}\n\nShort answer:",
            },
        ],
        temperature=0.0,
        max_tokens=256,
    )
    return (resp.choices[0].message.content or "").strip()


def _mistral_extract(question: str, long_answer: str, model: str) -> str:
    import httpx

    key = os.environ.get("MISTRAL_API_KEY")
    if not key:
        raise RuntimeError("MISTRAL_API_KEY required for mistral extractor")
    r = httpx.post(
        "https://api.mistral.ai/v1/chat/completions",
        headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"},
        json={
            "model": model,
            "temperature": 0.0,
            "max_tokens": 256,
            "messages": [
                {"role": "system", "content": EXTRACT_PROMPT},
                {
                    "role": "user",
                    "content": f"Question: {question}\n\nAnalysis:\n{long_answer}\n\nShort answer:",
                },
            ],
        },
        timeout=120.0,
    )
    r.raise_for_status()
    return (r.json()["choices"][0]["message"]["content"] or "").strip()
