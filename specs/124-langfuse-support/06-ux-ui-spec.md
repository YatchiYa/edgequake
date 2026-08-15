# 06 — UX / UI Spec

## Primary surface: Settings → Langfuse card

Placement: [`settings/page.tsx`](../../edgequake_webui/src/app/(dashboard)/settings/page.tsx), after Provider / Attribution cards.

```ascii
  ┌─ Langfuse Observability ─────────────────────────────┐
  │  Status: Not configured | Enabled | Misconfigured     │
  │                                                       │
  │  Base URL: https://cloud.langfuse.com                 │
  │  Public key: ✓ set | — not set                        │
  │  Secret key: ✓ set | — not set                        │
  │  OTEL build: ✓ feature otel | — rebuild required      │
  │                                                       │
  │  [ Copy env snippet ]                                 │
  │  [ Open in Langfuse ]   ← only if enabled+base_url    │
  └───────────────────────────────────────────────────────┘
```

### States

| State | Open button | Snippet | Message |
|-------|-------------|---------|---------|
| Unconfigured | Hidden | Shown | Configure via env |
| Enabled | Shown → `ui_url` | Shown (readonly) | Traces exporting |
| Keys without `otel` feature | Hidden | Shown | Unusual — otel is default; rebuild without `--no-default-features` |
| Partial keys | Hidden | Shown | Both keys required |

Patterns reused: ProviderStatus `config_requirements`, Swagger “Open in …” + `ExternalLink`, `target="_blank"` `rel="noopener noreferrer"`.

## Secondary: Health / dashboard

- `/health.operational.observability.langfuse_enabled` + `langfuse_base_url`
- Optional SystemStatus chip when enabled (non-blocking discovery)

## Per-trace link (query chrome)

When query/stream JSON includes `trace_id` **and** Langfuse configured:

```ascii
  [ Open trace in Langfuse ] → {base}/trace/{trace_id}
```

If not configured: no button (LAW-124-6).

## Config edit

v1: **env-only**. Card documents:

```bash
export LANGFUSE_PUBLIC_KEY=pk-lf-...
export LANGFUSE_SECRET_KEY=sk-lf-...
export LANGFUSE_BASE_URL=https://cloud.langfuse.com
# otel is on by default — restart the server after setting keys
```

No password inputs for Langfuse keys. No workspace-scoped override.

## Accessibility / copy

- Button label: “Open in Langfuse” / “Open trace in Langfuse”
- `data-testid="langfuse-settings-card"`, `langfuse-open-link`, `langfuse-open-trace`

## Cross-refs

- Front lens: [05-lenses/005-front-designer.md](05-lenses/005-front-designer.md)
- UX lens: [05-lenses/004-ux-ui.md](05-lenses/004-ux-ui.md)
- Acceptance: [09-acceptance.md](09-acceptance.md)
