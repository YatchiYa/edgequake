"""Locked provider profiles for SPEC-047."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional

# ---------------------------------------------------------------------------
# SSOT model pins (DRY) — query LLM ≠ vision model may diverge (025 / FP3).
# ---------------------------------------------------------------------------
QUERY_LLM_MODEL = "mistral-small-latest"
VISION_MODEL_LOCKED_SMALL = "mistral-small-latest"
# Official Mistral Medium 3.5 API id (docs.mistral.ai · edgequake-llm 0.10.1).
VISION_MODEL_STRONG = "mistral-medium-3-5"
EMBEDDING_MODEL = "mistral-embed"
EMBEDDING_DIM = 1024

# Back-compat aliases used by older scripts / docs
_LLM = QUERY_LLM_MODEL
_VISION = VISION_MODEL_LOCKED_SMALL
_EMBED = EMBEDDING_MODEL


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

    def uses_stronger_vision(self) -> bool:
        """True when Pass A/B vision is stronger than the locked Small Acc pin."""
        return self.vision_model != VISION_MODEL_LOCKED_SMALL

    def is_split_llm_vision(self) -> bool:
        """True when query LLM and vision model differ (intended W1 ablation)."""
        return self.llm_model != self.vision_model


def _mistral_postgres_profile(
    *,
    profile_id: str,
    description: str,
    query_mode: str = "hybrid",
    vision_model: str = VISION_MODEL_LOCKED_SMALL,
    pdf_parser_backend: str = "vision",
    extractor: str = "mistral",
    process_options: Optional[str] = None,
    requires_vlm_process: bool = False,
) -> BenchProfile:
    """Factory for the locked Mistral + mistral-embed Postgres stack (DRY)."""
    return BenchProfile(
        profile_id=profile_id,
        query_mode=query_mode,
        llm_provider="mistral",
        llm_model=QUERY_LLM_MODEL,
        vision_provider="mistral",
        vision_model=vision_model,
        embedding_provider="mistral",
        embedding_model=EMBEDDING_MODEL,
        embedding_dim=EMBEDDING_DIM,
        pdf_parser_backend=pdf_parser_backend,
        extractor=extractor,
        description=description,
        process_options=process_options,
        requires_vlm_process=requires_vlm_process,
    )


# SPEC-047 locked stack: Mistral Small (LLM+vision) + mistral-embed, Postgres storage
PROFILES: dict[str, BenchProfile] = {
    "P0_primary": _mistral_postgres_profile(
        profile_id="P0_primary",
        description="Headline: hybrid + Small vision + mistral-embed + Small judge (Postgres)",
    ),
    "P0_mm_ite": _mistral_postgres_profile(
        profile_id="P0_mm_ite",
        description="P0 + multimodal analyze process_options=ite (images/tables/equations)",
        process_options="ite",
        requires_vlm_process=True,
    ),
    # 025 / EQ-047-W1-vision: one causal change — Medium Pass A/B, Small query LLM.
    "P0_mm_ite_vision_medium": _mistral_postgres_profile(
        profile_id="P0_mm_ite_vision_medium",
        description=(
            "P0_mm_ite ablation: Small query LLM + mistral-medium-3-5 vision (Pass A/B only)"
        ),
        vision_model=VISION_MODEL_STRONG,
        process_options="ite",
        requires_vlm_process=True,
    ),
    "P0_official_extractor": _mistral_postgres_profile(
        profile_id="P0_official_extractor",
        description="P0 with official GPT-4o short-answer extractor",
        extractor="gpt-4o",
    ),
    "P0_mistral_judge": _mistral_postgres_profile(
        profile_id="P0_mistral_judge",
        description="Alias of P0_primary",
    ),
    "P1_naive": _mistral_postgres_profile(
        profile_id="P1_naive",
        description="Naive vector-only ablation",
        query_mode="naive",
    ),
    # SPEC-047 / 019 Q2.1: Mix RRF vs locked hybrid round-robin ablation
    "P1_mix_rrf": _mistral_postgres_profile(
        profile_id="P1_mix_rrf",
        description="Ablation: Mix + RRF fusion (EQ production default) vs P0 hybrid",
        query_mode="mix",
        process_options="ite",
        requires_vlm_process=True,
    ),
    "P5_text_parse": _mistral_postgres_profile(
        profile_id="P5_text_parse",
        description="EdgeParse (no vision) ablation — same Small LLM/embed",
        pdf_parser_backend="edgeparse",
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
    "reference only. Provider stack: mistral-small-latest + mistral-embed (Postgres). "
    "W1 ablation: P0_mm_ite_vision_medium pins mistral-medium-3-5 for vision only."
)
