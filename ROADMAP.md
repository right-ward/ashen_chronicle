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

### v0.43.1: inventory and meditation presentation refinement

- Inventory item selection now uses a clean list screen with no item art or details shown before selection.
- Selected inventory items open a dedicated detail screen containing the item's description and available art.
- Meditation now keeps the time-of-day selection on a dedicated presentation screen and shows the completed meditation, elapsed portions, recovery, and resulting time on a dedicated result screen.
- Unsafe meditation attempts also use the dedicated Meditation screen instead of the action log.

### v0.43.0: inventory and character presentation
- Replaced the plain inventory dump with a dedicated Inventory screen that lets the player select held items and view each item's description and available art.
- Replaced the plain character sheet with a dedicated Character screen containing General, Reputation, and Journal tabs.
- General shows core character stats, health, experience, effective attributes, and active conditions.
- Reputation shows faction standing and remembered faction dealings; Journal shows recorded character notes.
- Reworked meditation to choose the next named time-of-day stopping point instead of entering a numeric duration, while preserving healing, condition updates, history, and automatic saving.
- Preserved the existing screen flow so backing out of the dedicated screens returns to normal gameplay.

## Next

### v0.45.0 - 0.46.0

- Continue modularizing and improving the existing TUI presentation before the longer-term GUI transition.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
