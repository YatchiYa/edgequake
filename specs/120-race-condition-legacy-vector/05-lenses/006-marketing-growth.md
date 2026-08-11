# Lens 006 — Marketing & Growth

## Narrative

EdgeQuake concurrent ingest is **safe by default** after provenance uniqueness shipped. A race that briefly surfaced as hard failures is closed without asking customers to serialize uploads.

## Talking points (post-ship)

- Concurrent workspace ingest remains supported.
- Provenance uniqueness retained (no data-model rollback).
- Upgrade from 0.23 → 0.24.x no longer requires “lower concurrency” as a standing workaround for this class.

## Avoid

- Claiming “zero duplicate entities historically” (completeness debt remains under SPEC-083).
- Framing migration 143/144 as a mistake — they correctly made identity enforceable; absorb completes the story.
