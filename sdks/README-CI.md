# SDK CI (SPEC-083 D-48)

GitHub Actions **only** loads workflows from the repository root `.github/workflows/`.

Nested files under `sdks/*/.github/workflows/` are **not executed** by GHA. They remain as reference copies of the historical per-SDK recipes.

**Executable SSOT**: [`.github/workflows/sdk-ci.yml`](../.github/workflows/sdk-ci.yml) (Python / TypeScript / Rust matrix).

Other SDK languages (Go, Java, …) can be added to that workflow or as sibling `sdk-*.yml` files at the root.
