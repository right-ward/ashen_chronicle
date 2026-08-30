# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

### v0.43.0: inventory and character presentation
- Replaced the plain inventory dump with a dedicated Inventory screen that lets the player select held items and view each item's description and available art.
- Replaced the plain character sheet with a dedicated Character screen containing General, Reputation, and Journal tabs.
- General shows core character stats, health, experience, effective attributes, and active conditions.
- Reputation shows faction standing and remembered faction dealings; Journal shows recorded character notes.
- Reworked meditation to choose the next named time-of-day stopping point instead of entering a numeric duration, while preserving healing, condition updates, history, and automatic saving.
- Preserved the existing screen flow so backing out of the dedicated screens returns to normal gameplay.

## Next

### v0.44.0 - 0.46.0

- Continue modularizing and improving the existing TUI presentation before the longer-term GUI transition.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
