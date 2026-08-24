# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

- v0.33.0: Gameplay Action Architecture Cleanup V
- v0.33.0 is the current development milestone.
- `game/menu.rs` owns `GameAction`, menu entries, and main-menu construction.
- `game/actions.rs` now focuses on ordinary gameplay actions and shared time/condition helpers, plus inventory, quest, journaling, and character-sheet actions.
- Runtime menu construction and dispatch now depend on the dedicated menu layer rather than `game/actions.rs`.
- Gameplay behavior, save compatibility, and screen flow remain unchanged.

## Next

### v0.34.0 — Gameplay Action Architecture Cleanup VI

- Continue decomposing `game/actions.rs` only where a clear responsibility boundary improves maintainability.
- Prefer extracting cohesive gameplay concerns over splitting functions or files by size alone.
- Preserve existing gameplay behavior, save compatibility, and screen flow.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major architectural work should preserve backward compatibility where practical and include focused tests for behavior affected by the refactor.
