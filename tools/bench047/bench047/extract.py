"""Short-answer extraction (official GPT-4o or Mistral judge).

SPEC-047 / 026 W4-extract: English pin + list/number normalize so soft Acc
does not zero on format (bullets vs JSON list, unit fluff).
"""

from __future__ import annotations

import json
import os
import re
from typing import Optional

EXTRACT_PROMPT = """You are an answer extractor. Given a question and a long model analysis, extract the short final answer only.
Rules:
- Output ONLY the short answer (or "Not answerable" if the analysis concludes it cannot be answered from the document).
- Write the answer in English only.
- Preserve numbers and spelling carefully.
- If the gold-style answer is a bare number, prefer the bare number (omit units unless the question explicitly asks for units).
- If the answer is a list of items, output a JSON array of strings (e.g. ["Alice","Bob"]) — never markdown bullets or numbered lists.
- Do not explain. Do not wrap in quotes unless the answer itself is a quote.
"""

_BULLET_RE = re.compile(r"^\s*(?:[-*•]|\d+[.)])\s+")
_FLUFF_PREFIX_RE = re.compile(
    r"^(?:the\s+answer\s+is|answer\s*:|final\s+answer\s*:|short\s+answer\s*:)\s*",
    re.IGNORECASE,
)


def normalize_short_answer(answer: str) -> str:
    """Deterministic post-process for soft-score hygiene (026 W4-extract).

    - Strip common extractor fluff prefixes
    - Convert bullet / numbered lists → JSON array
    - Collapse whitespace
    Does NOT invent answers; does NOT ban \"Not answerable\".
    """
    text = (answer or "").strip()
    if not text:
        return text
    text = _FLUFF_PREFIX_RE.sub("", text).strip()
    if text.lower() in {"not answerable", "n/a", "na"}:
        return "Not answerable"

    lines = [ln.strip() for ln in text.splitlines() if ln.strip()]
    if len(lines) >= 2 and all(_BULLET_RE.match(ln) for ln in lines):
        items = [_BULLET_RE.sub("", ln).strip().strip('"').strip("'") for ln in lines]
        items = [i for i in items if i]
        if items:
            return json.dumps(items, ensure_ascii=False)

    # Single-line "a, b, c" list when clearly enumerating short tokens
    if (
        "," in text
        and "\n" not in text
        and not text.startswith("[")
        and len(text) < 200
        and text.count(",") >= 1
    ):
        parts = [p.strip().strip('"').strip("'") for p in text.split(",")]
        if (
            len(parts) >= 2
            and all(1 <= len(p) <= 40 for p in parts)
            and all(not p.endswith(".") for p in parts[:-1])
        ):
            # Only normalize when parts look like list items (no long prose clauses)
            if all(len(p.split()) <= 4 for p in parts):
                return json.dumps(parts, ensure_ascii=False)

    # Singleton JSON array → scalar (026 W4): ["MMMU"] → MMMU so soft Acc
    # matches bare-string gold. Multi-item lists stay JSON arrays.
    if text.startswith("["):
        try:
            parsed = json.loads(text)
        except json.JSONDecodeError:
            parsed = None
        if (
            isinstance(parsed, list)
            and len(parsed) == 1
            and isinstance(parsed[0], (str, int, float))
            and not isinstance(parsed[0], bool)
        ):
            return str(parsed[0]).strip()

    return text


def extract_answer_detailed(
    question: str,
    long_answer: str,
    *,
    extractor: str = "gpt-4o",
    model: Optional[str] = None,
) -> dict[str, str]:
    """Extract short answer; return both raw extractor output and normalized pred.

    Protocol: store ``pred_raw`` so Acc Δ can separate W4 normalize from content.
    """
    extractor = (extractor or "gpt-4o").lower()
    if extractor in {"gpt-4o", "openai", "official"}:
        raw = _openai_extract(question, long_answer, model or "gpt-4o")
    elif extractor in {
        "mistral",
        "mistral-small",
        "mistral-small-latest",
        "mistral-large",
        "mistral-large-latest",
    }:
        raw = _mistral_extract(question, long_answer, model or "mistral-small-latest")
    else:
        raise ValueError(f"Unknown extractor: {extractor}")
    return {"pred_raw": raw, "pred": normalize_short_answer(raw)}


def extract_answer(
    question: str,
    long_answer: str,
    *,
    extractor: str = "gpt-4o",
    model: Optional[str] = None,
) -> str:
    return extract_answer_detailed(
        question, long_answer, extractor=extractor, model=model
    )["pred"]


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
