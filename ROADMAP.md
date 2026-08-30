# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

### v0.40.0: quest depth and world consequences

- Expanded quests from single-item turn-ins into explicit persisted objective state.
- Existing campaign quests now track visit, defeat, and item-acquisition objectives as a single quest chain.
- Objective progress updates through travel and combat and is visible in the quest log and NPC turn-in flow.
- Quest completion now records a structured quest history event and updates the existing faction and NPC memory systems, allowing later event conditions to react to completed deeds.
- Existing quest/save data is migrated through serde defaults and deterministic quest normalization without changing the save-file version.
- Loaded quest objectives are validated for empty targets, invalid requirements, and impossible progress before runtime use.

## Next

### v0.41.0 - 0.46.0

- Continue modularizing and improving the existing TUI presentation before the longer-term GUI transition.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
