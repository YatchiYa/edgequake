# Langfuse Agent Skill (vendored)

Vendored from [langfuse/skills](https://github.com/langfuse/skills) for EdgeQuake SPEC-124.

- Upstream: `https://github.com/langfuse/skills` → `skills/langfuse`
- Primary entry: [SKILL.md](./SKILL.md)
- Instrumentation guide: [references/instrumentation.md](./references/instrumentation.md)
- Live best practices (always fetch fresh when implementing):
  https://langfuse.com/docs/observability/best-practices

## Env (operator)

```bash
export LANGFUSE_PUBLIC_KEY=pk-lf-...
export LANGFUSE_SECRET_KEY=sk-lf-...
export LANGFUSE_BASE_URL=https://cloud.langfuse.com  # or US / self-hosted
# Alias accepted by EdgeQuake: LANGFUSE_HOST=$LANGFUSE_BASE_URL
```

Do not paste secrets into chat. Keys live under Langfuse project Settings → API Keys.
