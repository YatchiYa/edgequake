# 05 — E2E / contract test matrix

> Gates for Waves A–F. Prefer wiremock / httpmock against Ollama JSON shapes over live GPU. Live smoke optional under `#[ignore]` + `OLLAMA_HOST`.

## Legend

| Tag | Meaning |
|-----|---------|
| C | Compile-time / source contract |
| U | Unit (no network) |
| M | Mocked HTTP (show/tags/chat) |
| L | Live Ollama (optional / ignored by default) |
| A | EdgeQuake API integration |

## Matrix

| ID | Wave | Tag | Assert |
|----|------|-----|--------|
| T-113-01 | A | U | `capabilities_include_thinking(["completion","thinking"])` ⇒ true |
| T-113-02 | A | U | `capabilities_include_thinking(["completion","vision"])` ⇒ false |
| T-113-03 | A | U | Auto + `ThinkingSupport::Yes` ⇒ `think: true` |
| T-113-04 | A | U | Auto + `ThinkingSupport::No` ⇒ `think` omitted |
| T-113-05 | A | U | Auto + `ThinkingSupport::Unknown` ⇒ `think` omitted |
| T-113-06 | A | U | Model id `qwen3-vl:8b` + support `No` ⇒ omit (name must not override) |
| T-113-07 | A | U | Model id `vl-instruct-8b` + support `Yes` ⇒ Auto think allowed (alias ≠ capability) |
| T-113-08 | A | U | Explicit `reasoning_effort=none` ⇒ omit regardless of Yes |
| T-113-09 | A | U | Explicit `high` + support `No` ⇒ omit + does not panic |
| T-113-10 | A | M | Mock `/api/show` without thinking; assert outbound `/api/chat` JSON has no `think` |
| T-113-11 | A | M | Mock `/api/show` with thinking; Auto chat includes `"think":true` |
| T-113-12 | A | M | Mock show 500 / timeout → chat still 200 from mock chat; no `think` |
| T-113-13 | B | U | Cache: second lookup does not re-hit show (counter) within TTL |
| T-113-14 | B | U | Different hosts do not share cache entries |
| T-113-15 | B | U | `legacy_name` mode: `qwen3-vl` Auto sends think (rollback path) |
| T-113-16 | C | C | Default path source does **not** call substring list as SSOT (or `is_thinking_model` only under legacy) |
| T-113-17 | C | U | Discovery parse helper == resolver parse helper (same fixtures) |
| T-113-18 | C | U | `reasoning_capabilities("ollama","qwen3-vl:8b")` does not alone force think send |
| T-113-19 | D | A | Models search `supports_thinking` matches tags mock capabilities |
| T-113-20 | D | A | Query/chat with Ollama mock VL model succeeds (no 502 from think) |
| T-113-21 | E | C | Regression: existing thinking response parse tests still pass |
| T-113-22 | E | M | Stream + non-stream both honor capability gate |
| T-113-23 | F | L | Optional live: real `qwen3` with thinking vs VL without — document in measurements/ |
| T-113-24 | A | U | `#369` fixture: name contains `qwen3`, caps lack thinking → omit |

## Fixtures (canonical JSON)

### Show — thinking model

```json
{
  "capabilities": ["completion", "tools", "thinking"],
  "details": { "family": "qwen3" }
}
```

### Show — VL / non-thinking (issue class)

```json
{
  "capabilities": ["completion", "vision"],
  "details": { "family": "qwen3" }
}
```

### Chat assertion helper

```text
  deserialize outbound body
  if support != Yes or effort in {none,off,…}:
      assert !body.contains_key("think")
  else if Auto and support == Yes:
      assert body["think"] == true
```

## Suggested commands

```bash
# In edgequake-llm
cargo test -p edgequake-llm ollama -- --nocapture
cargo test -p edgequake-llm --test e2e_ollama_think_capability -- --nocapture   # proposed

# After dep bump in EdgeQuake
cargo test -p edgequake-api --lib models_search
cargo test -p edgequake-api --test spec113_ollama_think_gate -- --nocapture     # proposed
```

## Proof artifacts

Write under [`measurements/`](measurements/):

| File | Content |
|------|---------|
| `e2e113-gates.txt` | Command + pass/fail summary |
| `e2e113-wiremock-vl.txt` | Outbound chat JSON for VL fixture |
| `e2e113-live-optional.txt` | Live curl show+chat if run |

## Exit criteria

- T-113-03..12, 16, 22, 24 green on CI  
- T-113-23 optional  
- #369 repro script fails on 0.10.4 heuristic path, passes on fixed crate (documented)
