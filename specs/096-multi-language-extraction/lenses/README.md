# SPEC-096 — Lenses

Each lens cites laws from [00-first-principles.md](../00-first-principles.md) and findings from [01-finding-register.md](../01-finding-register.md).

| Lens | File | Primary question |
|------|------|------------------|
| Product Owner | [LENS-product-owner.md](LENS-product-owner.md) | What does “done” mean for non-English knowledge graphs? |
| UX | [LENS-ux.md](LENS-ux.md) | Can operators find and trust language config without clutter? |
| UI | [LENS-ui.md](LENS-ui.md) | Exact controls, states, testids, visual hierarchy |
| Front End | [LENS-frontend.md](LENS-frontend.md) | Types, API client, React Query, create + deeplink pages |
| Full Stack | [LENS-fullstack.md](LENS-fullstack.md) | FE→API→metadata→orchestrator→pipeline→prompt |
| Database | [LENS-database.md](LENS-database.md) | JSONB SSOT, no migration, compat |

## Reading order

1. Product Owner (acceptance / non-goals)  
2. UX → UI → Front End (surface)  
3. Full Stack (wire)  
4. Database (persistence constraints)  
