# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

### v0.44.0: dedicated combat encounter presentation
- Replaced combat's generic action/log presentation with a dedicated combat encounter screen.
- Combat now presents the player and enemy together with current turn, encounter location, enemy power, available actions, and recent combat events.
- Player and enemy health remain displayed through the existing LineGauge presentation while combat is active.
- Combat results for victory, defeat, and fleeing are presented on the encounter screen before returning to normal gameplay or the death flow.
- Preserved existing combat rules, history recording, rewards, quest updates, and threat cleanup behavior.

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

## Next

### v0.45.0 - 0.46.0

- Migrate the remaining existing screens onto the presentation boundary using the shared UI primitives.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
