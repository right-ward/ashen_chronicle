# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

### v0.47.0: procedural world foundation
- Added deterministic procedural world generation with explicit region/location/extra-edge configuration and a connected generated world graph.
- Added deterministic authored-content placement on generated worlds while preserving authored location identities, metadata, exits, and gameplay relationships.
- Persisted the generated world's seed and generation configuration as world-level metadata with backward-compatible defaults for older saves.
- Switched new-game initialization to generated worlds and preserved generated structure plus runtime mutations across save/load without re-seeding generated locations.
- Added focused generation, content-placement, save/load, and initialization coverage for the new world lifecycle.

## Next

### v0.48.0 planning

- Define the next gameplay/content milestone before implementation begins.
- Track the future interaction-intent layer separately if GUI work requires inputs more abstract than the current semantic `InputEvent` controls.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
