# LENS — Front Designer (SPEC-101)

## Visual SSOT — ServerDefaultsCard

```ascii
┌─────────────────────────────────────────────┐
│ Using server defaults                       │
│ LLM        ollama/gemma4:latest             │
│ Embedding  openai/text-embedding-3-small    │
│ Vision     ollama/gemma4:latest             │
│ Source: environment / server config         │
│ [ Customize models ]                        │
└─────────────────────────────────────────────┘
```

- Monospace for IDs  
- Muted source line  
- Skeleton 3 lines while `/models` loads (no null flash)  
- `data-testid="server-defaults-card"`

## Wizard chrome

- Max width ~560px dialog (comfortable reading)  
- Thin progress bar under header  
- Step counter `Step 2 of 5` + SR live region  
- Primary = Next/Create; secondary = Back; ghost = Cancel (hidden on first-run)  
- First-run: `showCloseButton={false}`  
- Create: dirty cancel → AlertDialog  
- Preserve existing EdgeQuake tokens (no purple-onboarding theme)

## Density

Happy path: no badges/chips clusters. Advanced: single provider `<Select>` + Command list.

## Evidence QC

Capture models + review at 1440 / 768 / 375 into `specs/101-…/evidence/after-*.png` (not Percy).

## Reconfigure Workspace

- Same chrome as create; Finish label **Apply**
- Impact block on Review: muted warning surface, list changed fields + rebuild hints
- `data-testid="reconfigure-workspace-wizard"` / `wizard-reconfigure-impact`
- Capture `after-reconfigure-*.png` at 1440 / 768 / 375
