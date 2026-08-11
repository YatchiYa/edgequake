# Geometry results (SPEC-115)

- UTC: `2026-08-10T14:22:48.154011+00:00`
- Sample: `zz_test_docs/academic_papers/lighrag_2410.05779v3.pymupdf.gold.md`
- chars=61353 utf8_bytes=61547 tiktoken=14156
- PDF bytes=1123301 (adaptive-if-wrong-key→600); text adaptive pin→800/66
- Chunker: **real LightRAG F** (`chunking_by_token_size`)

| Pin | N | min | avg | max |
|-----|--:|----:|----:|----:|
| F@1200/100 | 13 | 955 | 1181.1 | 1200 |
| F@800/66 | 20 | 209 | 770.4 | 801 |
| F@600/50 | 26 | 405 | 592.3 | 600 |

**H-C1 (F geometry):** N_product/N_fair = 20/13 = **1.538**
