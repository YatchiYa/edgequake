"""Locked provider profiles for SPEC-047."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional


@dataclass(frozen=True)
class BenchProfile:
    profile_id: str
    query_mode: str
    llm_provider: str
    llm_model: str
    vision_provider: str
    vision_model: str
    embedding_provider: str
    embedding_model: str
    embedding_dim: int
    pdf_parser_backend: str  # vision | edgeparse
    extractor: str  # gpt-4o | mistral
    description: str
    # LightRAG-style multimodal second pass: "i"/"t"/"e" chars (SPEC-047 Phase A).
    process_options: Optional[str] = None
    # Require VLM_PROCESS_ENABLE on the API host when process_options includes "i".
    requires_vlm_process: bool = False


# SPEC-047 locked stack: Mistral Small (LLM+vision) + mistral-embed, Postgres storage
_LLM = "mistral-small-latest"
_VISION = "mistral-small-latest"
_EMBED = "mistral-embed"


PROFILES: dict[str, BenchProfile] = {
    "P0_primary": BenchProfile(
        profile_id="P0_primary",
        query_mode="hybrid",
        llm_provider="mistral",
        llm_model=_LLM,
        vision_provider="mistral",
        vision_model=_VISION,
        embedding_provider="mistral",
        embedding_model=_EMBED,
        embedding_dim=1024,
        pdf_parser_backend="vision",
        extractor="mistral",
        description="Headline: hybrid + Small vision + mistral-embed + Small judge (Postgres)",
    ),
    "P0_mm_ite": BenchProfile(
        profile_id="P0_mm_ite",
        query_mode="hybrid",
        llm_provider="mistral",
        llm_model=_LLM,
        vision_provider="mistral",
        vision_model=_VISION,
        embedding_provider="mistral",
        embedding_model=_EMBED,
        embedding_dim=1024,
        pdf_parser_backend="vision",
        extractor="mistral",
        description="P0 + multimodal analyze process_options=ite (images/tables/equations)",
        process_options="ite",
        requires_vlm_process=True,
    ),
    "P0_official_extractor": BenchProfile(
        profile_id="P0_official_extractor",
        query_mode="hybrid",
        llm_provider="mistral",
        llm_model=_LLM,
        vision_provider="mistral",
        vision_model=_VISION,
        embedding_provider="mistral",
        embedding_model=_EMBED,
        embedding_dim=1024,
        pdf_parser_backend="vision",
        extractor="gpt-4o",
        description="P0 with official GPT-4o short-answer extractor",
    ),
    "P0_mistral_judge": BenchProfile(
        profile_id="P0_mistral_judge",
        query_mode="hybrid",
        llm_provider="mistral",
        llm_model=_LLM,
        vision_provider="mistral",
        vision_model=_VISION,
        embedding_provider="mistral",
        embedding_model=_EMBED,
        embedding_dim=1024,
        pdf_parser_backend="vision",
        extractor="mistral",
        description="Alias of P0_primary",
    ),
    "P1_naive": BenchProfile(
        profile_id="P1_naive",
        query_mode="naive",
        llm_provider="mistral",
        llm_model=_LLM,
        vision_provider="mistral",
        vision_model=_VISION,
        embedding_provider="mistral",
        embedding_model=_EMBED,
        embedding_dim=1024,
        pdf_parser_backend="vision",
        extractor="mistral",
        description="Naive vector-only ablation",
    ),
    # SPEC-047 / 019 Q2.1: Mix RRF vs locked hybrid round-robin ablation
    "P1_mix_rrf": BenchProfile(
        profile_id="P1_mix_rrf",
        query_mode="mix",
        llm_provider="mistral",
        llm_model=_LLM,
        vision_provider="mistral",
        vision_model=_VISION,
        embedding_provider="mistral",
        embedding_model=_EMBED,
        embedding_dim=1024,
        pdf_parser_backend="vision",
        extractor="mistral",
        description="Ablation: Mix + RRF fusion (EQ production default) vs P0 hybrid",
        process_options="ite",
        requires_vlm_process=True,
    ),
    "P5_text_parse": BenchProfile(
        profile_id="P5_text_parse",
        query_mode="hybrid",
        llm_provider="mistral",
        llm_model=_LLM,
        vision_provider="mistral",
        vision_model=_VISION,
        embedding_provider="mistral",
        embedding_model=_EMBED,
        embedding_dim=1024,
        pdf_parser_backend="edgeparse",
        extractor="mistral",
        description="EdgeParse (no vision) ablation — same Small LLM/embed",
    ),
}


def get_profile(name: Optional[str] = None) -> BenchProfile:
    key = name or "P0_primary"
    if key not in PROFILES:
        raise SystemExit(f"Unknown profile {key}. Choose from: {', '.join(PROFILES)}")
    return PROFILES[key]


BANNER = (
    "EdgeQuake RAG adaptation of MMLongBench-Doc; not comparable to the LVLM "
    "leaderboard without caveats. Official LVLM GPT-4o F1≈44.9% is a difficulty "
    "reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres)."
)
