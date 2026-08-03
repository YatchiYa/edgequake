"""Provider / judge pins for SPEC-001.

Defaults (headline ``ACC_E2OCC_086_v1`` — SPEC-086 E2-occ Mix law):
  LLM + vision = mistral / mistral-small-latest
  Embed        = mistral / mistral-embed @ 1024-d
  Judge LLM    = same as SUT LLM (overridable independently)
  Mix law      = round_robin · bfs · retrieval · occ_sort · Fact L2

All roles are parameters (CLI + env). Scorecard ``pins`` records full lineage.
"""

from __future__ import annotations

import os
from dataclasses import dataclass
from typing import Any

# ---------------------------------------------------------------------------
# Defaults — locked headline stack
# ---------------------------------------------------------------------------

# Headline Acc / publication profile id (SPEC-086 E2-occ Mix law).
PROFILE_ID_DEFAULT = "ACC_E2OCC_086_v1"

DEFAULT_LLM_PROVIDER = "mistral"
DEFAULT_LLM_MODEL = "mistral-small-latest"
DEFAULT_VISION_PROVIDER = "mistral"
DEFAULT_VISION_MODEL = "mistral-small-latest"
DEFAULT_EMBEDDING_PROVIDER = "mistral"
DEFAULT_EMBEDDING_MODEL = "mistral-embed"
DEFAULT_EMBEDDING_DIM = 1024
DEFAULT_LLM_BASE_URL = "https://api.mistral.ai/v1"

# Judge LLM defaults to SUT LLM.
# Metric-side embed for Acc's 25% cosine term: default mistral-embed (aligned with
# SUT). Paper Table-2 parity uses BAAI/bge-large-en-v1.5 via --judge-embedding-model.
DEFAULT_JUDGE_EMBEDDING_MODEL = "mistral-embed"
PAPER_JUDGE_EMBEDDING_MODEL = "BAAI/bge-large-en-v1.5"

CHUNK_SIZE = 1200
# Legacy default; publishable runs use fair_pins.retrieve_topk() (=30).
RETRIEVE_TOPK = 30

DEFAULT_QUERY_CONCURRENCY = 8
MAX_QUERY_CONCURRENCY = 64
# Judgment is I/O-bound (Mistral API). Default higher; cap for rate limits.
DEFAULT_EVAL_CONCURRENCY = 16
MAX_EVAL_CONCURRENCY = 64

# Larger same-family stacks (Acc lift / publishable ablations).
LARGE_MISTRAL_MODEL = "mistral-large-latest"
MEDIUM_MISTRAL_MODEL = "mistral-medium-latest"

# Backward-compat module aliases (resolved from active pins at access time via
# the module-level constants below — kept as the *default* literals so
# ``from profiles import LLM_MODEL`` still means the headline default).
PROFILE_ID = PROFILE_ID_DEFAULT
LLM_PROVIDER = DEFAULT_LLM_PROVIDER
LLM_MODEL = DEFAULT_LLM_MODEL
VISION_PROVIDER = DEFAULT_VISION_PROVIDER
VISION_MODEL = DEFAULT_VISION_MODEL
EMBEDDING_PROVIDER = DEFAULT_EMBEDDING_PROVIDER
EMBEDDING_MODEL = DEFAULT_EMBEDDING_MODEL
EMBEDDING_DIM = DEFAULT_EMBEDDING_DIM
MISTRAL_BASE_URL = DEFAULT_LLM_BASE_URL


@dataclass(frozen=True)
class ProviderPins:
    """Resolved SUT + judge stack for one bench001 run."""

    profile_id: str
    llm_provider: str
    llm_model: str
    vision_provider: str
    vision_model: str
    embedding_provider: str
    embedding_model: str
    embedding_dim: int
    llm_base_url: str
    judge_provider: str
    judge_model: str
    judge_base_url: str
    judge_embedding_model: str

    @classmethod
    def defaults(cls) -> ProviderPins:
        return cls(
            profile_id=PROFILE_ID_DEFAULT,
            llm_provider=DEFAULT_LLM_PROVIDER,
            llm_model=DEFAULT_LLM_MODEL,
            vision_provider=DEFAULT_VISION_PROVIDER,
            vision_model=DEFAULT_VISION_MODEL,
            embedding_provider=DEFAULT_EMBEDDING_PROVIDER,
            embedding_model=DEFAULT_EMBEDDING_MODEL,
            embedding_dim=DEFAULT_EMBEDDING_DIM,
            llm_base_url=DEFAULT_LLM_BASE_URL,
            judge_provider=DEFAULT_LLM_PROVIDER,
            judge_model=DEFAULT_LLM_MODEL,
            judge_base_url=DEFAULT_LLM_BASE_URL,
            judge_embedding_model=DEFAULT_JUDGE_EMBEDDING_MODEL,
        )

    def lineage(self) -> dict[str, str]:
        """Compact model lineage for scorecard / SUMMARY."""
        return {
            "sut_llm": f"{self.llm_provider}/{self.llm_model}",
            "sut_vision": f"{self.vision_provider}/{self.vision_model}",
            "sut_embed": f"{self.embedding_provider}/{self.embedding_model}@{self.embedding_dim}d",
            "judge_llm": f"{self.judge_provider}/{self.judge_model}",
            "judge_metric_embed": self.judge_embedding_model,
            "llm_base_url": self.llm_base_url,
            "judge_base_url": self.judge_base_url,
        }

    def to_pin_fields(self) -> dict[str, Any]:
        return {
            "profile_id": self.profile_id,
            "llm_provider": self.llm_provider,
            "llm_model": self.llm_model,
            "vision_provider": self.vision_provider,
            "vision_model": self.vision_model,
            "embedding_provider": self.embedding_provider,
            "embedding_model": self.embedding_model,
            "embedding_dim": self.embedding_dim,
            "llm_base_url": self.llm_base_url,
            "judge_provider": self.judge_provider,
            "judge_model": self.judge_model,
            "judge_base_url": self.judge_base_url,
            "judge_embedding_model": self.judge_embedding_model,
            "lineage": self.lineage(),
        }


_ACTIVE: ProviderPins | None = None


def set_active_pins(pins: ProviderPins) -> None:
    global _ACTIVE
    _ACTIVE = pins


def active_pins() -> ProviderPins:
    return _ACTIVE if _ACTIVE is not None else ProviderPins.defaults()


def _env(*names: str, default: str) -> str:
    for name in names:
        raw = os.environ.get(name)
        if raw is not None and str(raw).strip():
            return str(raw).strip()
    return default


def _env_int(*names: str, default: int) -> int:
    for name in names:
        raw = os.environ.get(name)
        if raw is None or not str(raw).strip():
            continue
        try:
            return int(raw)
        except ValueError:
            continue
    return default


def resolve_pins(
    *,
    llm_provider: str | None = None,
    llm_model: str | None = None,
    vision_provider: str | None = None,
    vision_model: str | None = None,
    embedding_provider: str | None = None,
    embedding_model: str | None = None,
    embedding_dim: int | None = None,
    llm_base_url: str | None = None,
    judge_provider: str | None = None,
    judge_model: str | None = None,
    judge_base_url: str | None = None,
    judge_embedding_model: str | None = None,
    profile_id: str | None = None,
) -> ProviderPins:
    """CLI override > BENCH001_* > EDGEQUAKE_* / MISTRAL_* > defaults."""
    base = ProviderPins.defaults()

    llm_p = llm_provider or _env(
        "BENCH001_LLM_PROVIDER", "EDGEQUAKE_LLM_PROVIDER", default=base.llm_provider
    )
    llm_m = llm_model or _env(
        "BENCH001_LLM_MODEL", "EDGEQUAKE_LLM_MODEL", "MISTRAL_MODEL", default=base.llm_model
    )
    vis_p = vision_provider or _env(
        "BENCH001_VISION_PROVIDER",
        "EDGEQUAKE_VISION_PROVIDER",
        default=base.vision_provider,
    )
    vis_m = vision_model or _env(
        "BENCH001_VISION_MODEL", "EDGEQUAKE_VISION_MODEL", default=base.vision_model
    )
    emb_p = embedding_provider or _env(
        "BENCH001_EMBEDDING_PROVIDER",
        "EDGEQUAKE_EMBEDDING_PROVIDER",
        default=base.embedding_provider,
    )
    emb_m = embedding_model or _env(
        "BENCH001_EMBEDDING_MODEL",
        "MISTRAL_EMBEDDING_MODEL",
        "EDGEQUAKE_EMBEDDING_MODEL",
        default=base.embedding_model,
    )
    emb_d = (
        embedding_dim
        if embedding_dim is not None
        else _env_int(
            "BENCH001_EMBEDDING_DIM",
            "EDGEQUAKE_EMBEDDING_DIM",
            default=base.embedding_dim,
        )
    )
    if llm_base_url:
        base_url = llm_base_url
    elif os.environ.get("BENCH001_LLM_BASE_URL"):
        base_url = os.environ["BENCH001_LLM_BASE_URL"].strip()
    elif llm_p == "mistral":
        base_url = DEFAULT_LLM_BASE_URL
    else:
        base_url = _env("OPENAI_BASE_URL", default="https://api.openai.com/v1")

    jud_p = judge_provider or _env("BENCH001_JUDGE_PROVIDER", default=llm_p)
    jud_m = judge_model or _env("BENCH001_JUDGE_MODEL", default=llm_m)
    if judge_base_url:
        jud_url = judge_base_url
    elif os.environ.get("BENCH001_JUDGE_BASE_URL"):
        jud_url = os.environ["BENCH001_JUDGE_BASE_URL"].strip()
    elif jud_p == llm_p:
        jud_url = base_url
    elif jud_p == "mistral":
        jud_url = DEFAULT_LLM_BASE_URL
    else:
        jud_url = _env("OPENAI_BASE_URL", default="https://api.openai.com/v1")
    jud_emb = judge_embedding_model or _env(
        "BENCH001_JUDGE_EMBEDDING_MODEL", default=base.judge_embedding_model
    )

    pid = profile_id or _env("BENCH001_PROFILE_ID", default=base.profile_id)
    # Auto-tag profile when stack leaves the headline Mistral defaults.
    if pid == PROFILE_ID_DEFAULT and (
        llm_p != DEFAULT_LLM_PROVIDER
        or llm_m != DEFAULT_LLM_MODEL
        or emb_p != DEFAULT_EMBEDDING_PROVIDER
        or emb_m != DEFAULT_EMBEDDING_MODEL
    ):
        pid = f"P0_custom_{llm_p}_{emb_p}"
    from .fair_pins import resolve_publish_profile_id

    pid = resolve_publish_profile_id(pid)

    return ProviderPins(
        profile_id=pid,
        llm_provider=llm_p,
        llm_model=llm_m,
        vision_provider=vis_p,
        vision_model=vis_m,
        embedding_provider=emb_p,
        embedding_model=emb_m,
        embedding_dim=int(emb_d),
        llm_base_url=base_url,
        judge_provider=jud_p,
        judge_model=jud_m,
        judge_base_url=jud_url,
        judge_embedding_model=jud_emb,
    )


def mistral_api_key() -> str | None:
    """Return a usable Mistral API key, ignoring agent-injected placeholders."""
    for env_name in ("MISTRAL_API_KEY", "LLM_API_KEY"):
        val = (os.environ.get(env_name) or "").strip()
        if not val:
            continue
        # Cursor/agent sandboxes sometimes inject LLM_API_KEY=FAKE… placeholders.
        if val.upper().startswith("FAKE"):
            continue
        return val
    return None


def sut_api_key(pins: ProviderPins | None = None) -> str | None:
    """API key for SUT LLM/embed calls."""
    p = pins or active_pins()
    if p.llm_provider == "mistral" or p.embedding_provider == "mistral":
        return mistral_api_key() or os.environ.get("OPENAI_API_KEY")
    return (
        os.environ.get("OPENAI_API_KEY")
        or os.environ.get("LLM_API_KEY")
        or mistral_api_key()
    )


def judge_api_key(pins: ProviderPins | None = None) -> str | None:
    p = pins or active_pins()
    if p.judge_provider == "mistral":
        return mistral_api_key() or os.environ.get("OPENAI_API_KEY")
    return (
        os.environ.get("OPENAI_API_KEY")
        or os.environ.get("LLM_API_KEY")
        or mistral_api_key()
    )


def _clamp_concurrency(raw: str | int | None, *, default: int, cap: int) -> int:
    if raw is None:
        n = default
    else:
        try:
            n = int(raw)
        except (TypeError, ValueError):
            n = default
    return max(1, min(n, cap))


def query_concurrency(override: int | None = None) -> int:
    if override is not None:
        return _clamp_concurrency(override, default=DEFAULT_QUERY_CONCURRENCY, cap=MAX_QUERY_CONCURRENCY)
    return _clamp_concurrency(
        os.environ.get("BENCH001_QUERY_CONCURRENCY"),
        default=DEFAULT_QUERY_CONCURRENCY,
        cap=MAX_QUERY_CONCURRENCY,
    )


def eval_concurrency(override: int | None = None) -> int:
    if override is not None:
        return _clamp_concurrency(override, default=DEFAULT_EVAL_CONCURRENCY, cap=MAX_EVAL_CONCURRENCY)
    return _clamp_concurrency(
        os.environ.get("BENCH001_EVAL_CONCURRENCY"),
        default=DEFAULT_EVAL_CONCURRENCY,
        cap=MAX_EVAL_CONCURRENCY,
    )


def pin_block(
    *,
    fixture_id: str,
    judge: str,
    git_sha: str,
    dataset_id: str,
    dataset_revision: str,
    pins: ProviderPins | None = None,
) -> dict[str, Any]:
    """Scorecard pins — full model lineage for reproducibility."""
    from .fair_pins import publish_pin_fields, retrieve_topk
    from .judge_tune import judge_tune_pin_fields

    p = pins or active_pins()
    out: dict[str, Any] = {
        "edgequake_git_sha": git_sha,
        "dataset_id": dataset_id,
        "dataset_revision": dataset_revision,
        "fixture_id": fixture_id,
        **p.to_pin_fields(),
        **judge_tune_pin_fields(),
        **publish_pin_fields(),
        "eq_query_mode": "mix",
        "lr_query_mode": "mix",
        "chunk_size": CHUNK_SIZE,
        "retrieve_topk": retrieve_topk(),
        "query_concurrency": query_concurrency(),
        "eval_concurrency": eval_concurrency(),
        "judge": judge,
    }
    return out


def pins_as_env(pins: ProviderPins | None = None) -> dict[str, str]:
    """Env exports for subprocess / doctor display (SUT stack)."""
    p = pins or active_pins()
    return {
        "EDGEQUAKE_LLM_PROVIDER": p.llm_provider,
        "EDGEQUAKE_LLM_MODEL": p.llm_model,
        "EDGEQUAKE_VISION_PROVIDER": p.vision_provider,
        "EDGEQUAKE_VISION_MODEL": p.vision_model,
        "EDGEQUAKE_EMBEDDING_PROVIDER": p.embedding_provider,
        "MISTRAL_EMBEDDING_MODEL": p.embedding_model,
        "EDGEQUAKE_EMBEDDING_MODEL": p.embedding_model,
        "BENCH001_JUDGE_PROVIDER": p.judge_provider,
        "BENCH001_JUDGE_MODEL": p.judge_model,
        "BENCH001_JUDGE_BASE_URL": p.judge_base_url,
        "BENCH001_LLM_BASE_URL": p.llm_base_url,
    }
