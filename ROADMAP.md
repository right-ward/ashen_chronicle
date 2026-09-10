# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state
### v0.49.x: Ratatui to Bevy migration (in progress)
- Established the Bevy application foundation as an opt-in graphical runtime while retaining the existing Ratatui frontend for the incremental migration.
- Added a dedicated Bevy application module and kept engine/bootstrap concerns separate from gameplay state and rules.
- Added a reusable Bevy presentation layer with shared screen/panel/label primitives, choice buttons, health-style gauges, centralized visual theme constants, semantic navigation state, and a semantic input queue.
- Added Bevy keyboard translation for the existing frontend-neutral input events and Bevy UI interaction translation without exposing Bevy types to gameplay systems.
- Migrated lifecycle flows for starting, loading, character creation, death, inheritance, and quit confirmation to Bevy.
- Migrated the primary gameplay/world dashboard and world navigation/travel flow to Bevy while continuing to reuse authoritative gameplay rules.
- Migrated character, reputation, journal, inventory, quest, meditation, and history screens to Bevy, including nested detail/result flows and journal entry editing.
=======
- Persisted runtime-generated event definitions separately from authored campaign events so procedural event progression survives save/load.
- Removed per-trigger cloning of the complete campaign event vector while preserving deterministic event selection and RNG sequencing.
- Routed event-driven condition application through the shared condition refresh semantics to prevent duplicate same-named conditions.
- Recorded processed procedural world-evolution transitions in structured event history so resolved evolution quests are not reconciled repeatedly on later time advances.


## Next

Continue v0.49.x with combat migration, then migrate the developer console before removing the remaining Ratatui/crossterm runtime dependencies and completing the platform/documentation integration.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
