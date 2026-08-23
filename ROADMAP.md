# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

- v0.32.0: Gameplay Legacy Architecture Cleanup
- v0.32.0 is the current completed milestone.
- Character progression and character-sheet presentation live in `game/character.rs`.
- NPC dialogue, quest interaction, faction memory/reputation, and NPC availability live in `game/interactions.rs`.
- Death, corpse creation, corpse recovery, previous-life item recovery, and legacy item-gain presentation live in `game/legacy.rs`.
- `game/actions.rs` now focuses on ordinary gameplay actions, shared time/condition helpers, menu definitions, inventory/quest views, and journaling.
- Gameplay behavior, save compatibility, and screen flow remain unchanged.

## Next

### v0.33.0 — Gameplay Action Architecture Cleanup V

- Continue decomposing `game/actions.rs` only where a clear responsibility boundary improves maintainability.
- Avoid splitting files purely to reduce line counts.
- Preserve existing gameplay behavior, save compatibility, and screen flow.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major architectural work should preserve backward compatibility where practical and include focused tests for behavior affected by the refactor.
