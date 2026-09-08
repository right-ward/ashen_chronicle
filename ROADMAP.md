# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

### v0.48.1: procedural content and emerging world
- Added deterministic procedural generation primitives for regional themes, climates, prosperity, danger, population, resources, and tags.
- Added deterministic location characteristics derived from surrounding regional context, including location kinds, population, resources, danger, and tags.
- Added deterministic generated location names, context-driven factions, and NPCs using reusable name/content pools.
- Populated generated settlements and other populated locations with runtime-valid NPCs assigned to context-matched generated factions while preserving authored entities.
- Made generated entity population idempotent across repeated campaign bootstrap calls and covered deterministic generation with focused tests.
- Generated deterministic faction relationships from shared resources, danger, prosperity, and environmental differences, recording rivalry, alliance, trade, influence, or dependency in persistent faction memory.
- Integrated relationship generation into generated-world bootstrap and covered deterministic, idempotent, valid, and serialization-preserving relationship state.
- Integrated authored campaign NPCs and factions as deterministic anchors in generated regions without replacing their authored identities, faction assignments, locations, or quest references.
- Allowed authored factions to participate in the same deterministic generated relationship system as generated factions, while retaining authored-authored relationships unchanged.
- Covered authored anchor placement, authored quest reference integrity, cross-authored/generated relationships, and deterministic integration with focused tests.
- Hardened gameplay and persistence with negative Insight XP handling, persistent deterministic event RNG, bounded history, save-size/decompression limits, mod path confinement, and save serialization without cloning the full GameState.
- Generated persistent quests from faction relationships and dangerous locations, using existing quest objectives and NPC/faction runtime references.
- Generated deterministic travel events from the same world relationships and dangerous locations, reusing the existing event trigger/cooldown/history flow.
- Added consequence-driven world evolution: completing generated opportunities can pacify dangerous locations, record persistent faction memory, and create deterministic follow-up opportunities without regenerating the world graph.
- Covered generated opportunities, valid references, evolution, serialization, and time-driven consequence propagation with focused tests.

## Next

0.49.x milestone

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
