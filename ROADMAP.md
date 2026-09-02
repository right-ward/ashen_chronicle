# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

### v0.46.0: presentation foundation
- Added frontend-independent presentation/view models for character, item, location, threat, history, faction, NPC, quest, and combatant data.
- Kept presentation models free of ratatui/crossterm dependencies and gameplay rules so later frontends can consume the same data.
- Added shared terminal UI primitives for compact-layout detection, panel rendering, scrolling text, message panels, health gauges, frame clearing, and common layout behavior.
- Migrated the World/Chronicle screen to build a frontend-independent `WorldView` before terminal rendering.
- Reused shared UI primitives for the World screen's responsive layout, panels, and health gauge while preserving gameplay behavior and menu dispatch.
- Migrated History, Navigation, and Talk flows to build frontend-independent presentation views before terminal-specific display.
- Preserved existing history ordering, navigation choices, NPC availability, dialogue options, quest interactions, and time advancement behavior.
- Migrated Quest Log, Inventory, and Meditation flows to build frontend-independent presentation views before terminal-specific display.
- Preserved existing quest filtering/details, inventory selection/details and item art, meditation safety checks, target selection, recovery, save behavior, and result presentation.
- Migrated the Character Sheet and Combat flows to build frontend-independent presentation views before terminal rendering.
- Preserved character progression, condition/reputation/journal presentation, combat actions, events, result states, rewards, history recording, and threat cleanup behavior.
- Added a frontend-neutral `InputEvent` boundary and routed gameplay-facing keyboard interactions through semantic events instead of direct `crossterm::KeyCode` handling.
- Preserved existing keyboard controls, menu navigation, combat selection, history navigation, and developer-console editing/completion behavior.
- Added focused presentation-boundary tests covering renderer-neutral view composition and representative Character Sheet view-model generation.
- Migrated lifecycle/start/load/character-creation/quit/death flows to frontend-independent screen and choice views.
- Migrated corpse/legacy recovery flow to frontend-independent remains and recovery result views while preserving recovery rules and item behavior.
- Migrated developer-console rendering to consume a renderer-neutral `ConsoleView` while retaining semantic `InputEvent` handling.
- Completed issue #146, removing the remaining lifecycle, legacy/recovery, and developer-console presentation coupling from gameplay-facing flow modules.
- Completed the final architecture audit: direct ratatui/crossterm usage remains confined to renderer and input-adapter layers, presentation models remain renderer-neutral, and no additional v0.46.0 migration blocker was found.

## Next

### v0.47.0 planning

- Define the next gameplay/content milestone before implementation begins.
- Track the future interaction-intent layer separately if GUI work requires inputs more abstract than the current semantic `InputEvent` controls.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
