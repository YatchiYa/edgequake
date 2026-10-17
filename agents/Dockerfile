# EdgeQuake ADK agents (custom chat UI + ADK API). Build context: agents/
FROM python:3.12-slim
COPY --from=ghcr.io/astral-sh/uv:0.8.17 /uv /usr/local/bin/uv
WORKDIR /app
ENV UV_COMPILE_BYTECODE=1 UV_LINK_MODE=copy UV_PROJECT_ENVIRONMENT=/app/.venv \
    MPLBACKEND=Agg MPLCONFIGDIR=/tmp/matplotlib PYTHONUNBUFFERED=1
COPY pyproject.toml uv.lock ./
RUN uv sync --frozen --no-dev --no-install-project
COPY . .
RUN uv sync --frozen --no-dev && useradd -r -u 10001 agents && mkdir -p /data && chown -R agents /app /data
USER agents
ENV PATH=/app/.venv/bin:$PATH EQ_UI_HOST=0.0.0.0 EQ_UI_PORT=8765 EQ_DATA_DIR=/data
# /data: api_key, agents.db, sessions.db, artifacts/ (mount a volume). /v1 needs AGENTS_INTERNAL_TOKEN.
EXPOSE 8765
CMD ["python", "scripts/container_start.py"]
