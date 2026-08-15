# Lens 002 — Full Stack Developer

## Stake

One observability crate owns exporters; API exposes status; WebUI presents — no FE parsing of secrets.

## Backend checklist

- [ ] `LangfuseConfig::from_env()` pure + tested
- [ ] HTTP OTLP exporter alongside gRPC
- [ ] Health + `GET /settings/langfuse`
- [ ] Generation spans wired
- [ ] `trace_id` on query responses when active

## Frontend checklist

- [ ] Card with `data-testid`s
- [ ] Open link only when `enabled && ui_url`
- [ ] Copy env snippet (no secrets from server — placeholders)
- [ ] Query “Open trace” when `trace_id` + configured

## Anti-patterns

- Parsing `LANGFUSE_*` in React
- PATCH secrets endpoint
- Blocking request path on export flush

## Cross-refs

- Impl: [../07-implementation-plan.md](../07-implementation-plan.md)
- UX: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
- Code as-is: [../03-code-as-is.md](../03-code-as-is.md)
