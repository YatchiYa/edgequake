# Changelog

## Unreleased

### Fixed

- Structured JSON error responses now expose the server message through `APIError.Message` instead of the raw JSON body. Empty, plaintext and malformed error bodies retain the existing fallback.
