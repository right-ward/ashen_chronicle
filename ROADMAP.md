# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

- v0.35.0: Gameplay Action Architecture Cleanup VII
- `game/actions.rs` now focuses on travel and meditation gameplay actions.
- Inventory display, quest-log display, and journal writing now live in `game/records.rs` as a cohesive player-record concern.
- Character-sheet dispatch now goes directly through `game/character.rs` instead of being wrapped by the action layer.
- Gameplay behavior, save compatibility, and screen flow remain unchanged.

## Next

### v0.36.0 — Gameplay Action Architecture Cleanup VIII

- Continue decomposing `game/actions.rs` only where a clear responsibility boundary improves maintainability.
- Prefer extracting cohesive gameplay concerns over splitting functions or files by size alone.
- Preserve existing gameplay behavior, save compatibility, and screen flow.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major architectural work should preserve backward compatibility where practical and include focused tests for behavior affected by the refactor.
