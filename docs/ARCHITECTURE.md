# Public Architecture

## Scope

This repository is the public platform shell. It contains the desktop client, application workflows, domain contracts, PostgreSQL persistence, import/export tooling, AI collaboration features, and model-facing interfaces.

The prediction engines, production parameter profiles, fixed fixtures, private research prompts, and regression assets are not distributed.

## Workspace

```text
src/                         TypeScript user interface
src-tauri/                   Tauri desktop boundary and local configuration
crates/domain/               Domain records and workflow contracts
crates/model-api/            Stable model request/response interface
crates/model-stub/           Non-executing public model entries
crates/application/          Application orchestration
crates/persistence-postgres/ PostgreSQL repositories and migrations
crates/research-gateway/     OpenAI-compatible research transport
crates/spreadsheet-io/       Excel import/export
crates/analysis-package/     Offline analysis package exchange
crates/analytics-engine/     Platform analytics
crates/review-engine/        Match review calculations
```

## Model boundary

`football-model-api` is the authoritative public boundary. `football-model-stub` registers the expected model identifiers so existing routing and UI entry points remain available, but every prediction call returns `ModelError::Unavailable`.

A private deployment must provide a compatible implementation of `PredictionModel` or adapt the application registry to an external `ModelProvider`. The public shell never loads private parameters or fixtures.

## Dependency direction

```text
UI -> Tauri commands -> application -> domain/persistence/model-api
                                      -> model-stub (public shell only)
```

Domain and persistence crates do not depend on a concrete prediction engine. The public stub depends only on `domain` and `model-api`.

## Runtime behavior

Non-model capabilities remain available. Prediction and model acceptance operations reach the preserved entry point and fail explicitly until an external provider is connected. The failure is intentional and must not be converted into a silent fallback or fabricated output.
