# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

### v0.48.0: procedural content and emerging world
- Added deterministic procedural generation primitives for regional themes, climates, prosperity, danger, population, resources, and tags.
- Added deterministic location characteristics derived from surrounding regional context, including location kinds, population, resources, danger, and tags.
- Added deterministic generated location names, context-driven factions, and NPCs using reusable name/content pools.
- Populated generated settlements and other populated locations with runtime-valid NPCs assigned to context-matched generated factions while preserving authored entities.
- Made generated entity population idempotent across repeated campaign bootstrap calls and covered deterministic generation with focused tests.
- Generated deterministic faction relationships from shared resources, danger, prosperity, and environmental differences, recording rivalry, alliance, trade, influence, or dependency in persistent faction memory.
- Integrated relationship generation into generated-world bootstrap and covered deterministic, idempotent, valid, and serialization-preserving relationship state.

## Next

### v0.48.0 in progress

- Integrate authored content as stable anchors within the generated ecosystem.
- Generate emergent quests and initial world events from generated world state.
- Establish ongoing world evolution from generated state.
- Strengthen end-to-end procedural generation, integration, persistence, and evolution coverage.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
