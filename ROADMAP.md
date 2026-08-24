# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

- v0.37.0: Developer console foundation
- Added an overlay developer console with scrollable output, command history, and a `/` shortcut from the normal gameplay menu.
- Added command access to world navigation, content/mod inspection, quests, factions, NPCs, inventory, character stats, conditions, time, history, save, and content reload.
- Added Tab autocomplete with arrow-key navigation, Enter selection, Esc cancellation, and stable runtime entity IDs with names/titles shown as hints.
- Kept the normal gameplay renderer and save format unchanged.

## Next

### v0.38.0 — Quest Depth and World Consequences

- Expand the quest system beyond single-item turn-ins with explicit objective state and more meaningful world consequences.
- Use the existing event, history, faction, and NPC-memory systems so quest outcomes can affect the persistent world.
- Preserve save compatibility and keep new quest data validated before entering runtime state.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
