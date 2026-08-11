# Lens 007 — System Engineer

## Stake

Docker deployments are the #370 environment. PDF depends on native pdfium cache, multipart body limits across proxies, and outbound LLM/vision networking. JSON does not.

## Failure modes ranked

| Rank | Mode | Symptom | Mitigation |
|------|------|---------|------------|
| 1 | Reverse proxy `client_max_body_size` | 413 on PDF; JSON OK | Align proxy ≥ `EDGEQUAKE_MAX_UPLOAD_BYTES` |
| 2 | pdfium cache not writable | Startup fail or convert fail | `PDFIUM_AUTO_CACHE_DIR` 1777 / volume |
| 3 | Vision/Ollama unreachable from container | Admit OK, stuck/Failed converting | `OLLAMA_HOST` / API keys / host.docker.internal |
| 4 | Wrong endpoint (`/documents/upload` + PDF) | 400 unsupported | Docs + client use `/documents/pdf` |
| 5 | Missing workspace UUID | 400 Workspace ID required | FE header / API client |
| 6 | Corrupt / non-PDF bytes | Invalid PDF | Client validation |

## Runbook (operator)

```ascii
  1. curl health → storage + llm_provider
  2. Upload tiny JSON control → must 2xx
  3. curl -F file=@sample.pdf /api/v1/documents/pdf + workspace header
  4. If 413 → fix proxy body size
  5. If 400 unsupported → wrong route
  6. If admit then Failed → logs: pdfium / vision / PDF_CONVERSION_FAILED
  7. Confirm PDFIUM_AUTO_CACHE_DIR writable inside container
```

## Cross-refs

- Reproduction: [../10-reproduction.md](../10-reproduction.md)
- Docker: `edgequake/docker/Dockerfile`
