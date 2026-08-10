# LENS — Database (SPEC-015V)

- **No migration** — keys live in `workspaces.metadata` JSONB.
- **Doc snapshot** — store resolved extract + prompt hashes/text under document metadata at ingest (`vision_extract` object).
- **Types:** bools JSON true/false; prompts JSON strings; absent key = default.
