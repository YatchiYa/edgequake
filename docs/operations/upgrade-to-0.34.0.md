---
title: "Upgrade to EdgeQuake v0.34.0"
description: "Upgrade notes for EdgeQuake v0.34.0: a plain-English visual tour of the web app. Schema stays 169. Migrate is a no-op from 0.33.0."
---

# Upgrade to EdgeQuake v0.34.0

> **From:** v0.33.0 · **To:** v0.34.0 · **Schema:** **169** (unchanged)

This release adds a picture tour of the website. Query, storage, and the database schema do not change. If you are already on **169**, `edgequake migrate` has nothing to apply. Roll the API and WebUI images.

**demo.edgequake.com:** this cut installs the 0.34.0 API and WebUI. Password auth remains the demo default. The screens in [the visual tour](../../tutorial/README.md) were taken on this demo before the cut. Those screens do not change in 0.34.0. The version in the corner of the pictures reads **0.33.0** because that is the build that was running when they were taken.

## Highlights

| Area | What changed |
|------|--------------|
| Docs | [tutorial/README.md](../../tutorial/README.md), linked from the README as "Click through the app" |
| Schema | Stays **169** |
| Migrate | No-op from 0.33.0 |
| Acc | Attested existing medical-mid pack (`valid: true`, 2026-08-15). Retrieval was not re-scored |

## Upgrade path

```mermaid
%%{init: {"theme":"base","themeVariables":{"primaryColor":"#E0E7FF","primaryBorderColor":"#6366F1","primaryTextColor":"#1E1B4B","secondaryColor":"#D1FAE5","secondaryBorderColor":"#10B981","secondaryTextColor":"#064E3B","tertiaryColor":"#FEF3C7","tertiaryBorderColor":"#F59E0B","tertiaryTextColor":"#6B7A90","lineColor":"#7A889C","clusterBkg":"rgba(99,102,241,0.07)","clusterBorder":"#A5B4FC","noteBkgColor":"#FEF9C3","noteTextColor":"#422006","textColor":"#6B7A90","titleColor":"#6B7A90","signalColor":"#7A889C","signalTextColor":"#6B7A90","loopTextColor":"#6B7A90","edgeLabelBackground":"#F1F5F9","actorLineColor":"#94A3B8"}}}%%
%% eq-theme:v1
flowchart TB
    A["Running 0.33.0 on schema 169"] --> B["Confirm edgequake migrate reports nothing pending"]
    B --> C["Start the 0.34.0 API and WebUI"]
    C --> D["Open /health and confirm version 0.34.0"]
%% eq-classes
classDef eqStore fill:#D1FAE5,stroke:#10B981,color:#064E3B
class B eqStore
```

Read the chart from top to bottom. There is no new migration.

## Operator steps

1. You are on **v0.33.0** and schema **169**. If you are older, follow [upgrade-to-0.33.0.md](upgrade-to-0.33.0.md) first.
2. `edgequake migrate` against the existing database. It should report that **169** is already applied.
3. Deploy images `ghcr.io/raphaelmansuy/edgequake:0.34.0` and `ghcr.io/raphaelmansuy/edgequake-frontend:0.34.0`.
4. `GET /health` returns `"version": "0.34.0"`.

No new environment variables.
