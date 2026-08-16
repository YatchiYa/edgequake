# Lens 004 — UX / UI

## Stake

Users must see **per-file** truth. A single hung transfer must not look like “all uploads broken.”

## Vocabulary

| State | Copy |
|-------|------|
| pending (queued behind cap 3) | Waiting… |
| uploading / admit | Transferring / Saving to workspace… |
| success + track | Queued / Processing (SPEC-122 chips) |
| error / timeout | Upload failed — retry this file |

## Rules

1. Never imply whole-batch failure when one file errors.
2. After admit, do not keep “Transferring files” as the only headline if transfer is done — prefer “Transfer complete / Processing.”
3. Link FAQ: multi-PDF slow convert ≠ failed upload (SPEC-122).

## Cross-refs

- UX spec: [../06-ux-ui-spec.md](../06-ux-ui-spec.md)
- Front: [005-front-designer.md](005-front-designer.md)
