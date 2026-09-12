# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state
### v0.49.x: Bevy graphical frontend (terminal removal complete; final verification pending)
- Established Bevy as the default and shipped graphical runtime while keeping gameplay rules and presentation models independent of the frontend.
- Added a reusable Bevy presentation layer with shared screen/panel/label primitives, choice buttons, health-style gauges, centralized visual theme constants, semantic navigation state, and a semantic input queue.
- Added Bevy keyboard translation for the frontend-neutral input events and Bevy UI interaction translation without exposing Bevy types to gameplay systems.
- Migrated lifecycle flows for starting, loading, character creation, death, inheritance, and quit confirmation to Bevy.
- Migrated the primary gameplay/world dashboard and world navigation/travel flow to Bevy while continuing to reuse authoritative gameplay rules.
- Migrated NPC dialogue and remains recovery into dedicated Bevy interaction screens while keeping dialogue, quest interaction, corpse recovery, and related state changes in the authoritative game modules.
- Migrated character, reputation, journal, inventory, quest, meditation, and history screens to Bevy, including nested detail/result flows and journal entry editing.
- Migrated combat presentation and interaction to Bevy while keeping combat resolution, state changes, outcomes, and reward handling in the authoritative game combat system.
- Migrated the developer console frontend to Bevy while retaining renderer-neutral command state, completion, history, scrolling, save, output, and close behavior.
- Removed Ratatui/crossterm dependencies and terminal-only screen, rendering, input, and game-loop modules from the shipped application.
- Persisted runtime-generated event definitions separately from authored campaign events so procedural event progression survives save/load.
- Removed per-trigger cloning of the complete campaign event vector while preserving deterministic event selection and RNG sequencing.
- Routed event-driven condition application through the shared condition refresh semantics to prevent duplicate same-named conditions.
- Recorded processed procedural world-evolution transitions in structured event history so resolved evolution quests are not reconciled repeatedly on later time advances.
- Removed migration-only `InputEvent::Other` and unused shared `ScrollViewport` compatibility types from the frontend boundary.

## Next

Complete CI validation of the Bevy application and supported platform targets, verify save compatibility and full gameplay/navigation flow through the normal release workflow, then finalize the v0.49.0 release metadata and historical roadmap entry.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
