# Changelog

## Unreleased

### Fixed

- `chat.stream` now calls `/chat/completions/stream` and returns an enumerator of decoded SSE events instead of a single response. `query.stream` also decodes SSE events. Responses are buffered; events are not delivered incrementally over the network.
