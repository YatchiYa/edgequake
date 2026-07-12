# SPEC-048 E2E

## PDF corpus (figure extraction / SPEC-049)

| File | Paper | Pages |
|------|-------|------:|
| `ideas_2607.08758v1.pdf` | Ideas Have Genomes (arXiv 2607.08758) | 22 |
| `hierar_2607.02980v1.pdf` | Hierarchical Sparse Attention (arXiv 2607.02980) | 27 |
| `lighrad_2410.05779v3.pdf` | LightRAG (arXiv 2410.05779) | 16 |

```bash
cd edgequake
cargo test -p edgequake-api --test e2e_spec049_visual_regions -- --test-threads=1
```

Text-native tables do not require `-table-` PNG crops (see SPEC-049 first principles).

## Screenshots

See [screenshots/ANALYSIS.md](./screenshots/ANALYSIS.md) for visual review of S01–S06.

## Run

```bash
cd edgequake_webui
pnpm exec playwright test e2e/spec048-ingestion-progress.spec.ts --project=chromium
```

Artifacts write to `screenshots/*.png` + `ANALYSIS.md` (appended by the suite).

## Backend contracts

```bash
cd edgequake
cargo test -p edgequake-api --test contract_spec048_progress
cargo test -p edgequake-api --lib progress_facade
```
