# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

- v0.29.0: Gameplay Action Architecture Cleanup
- v0.29.0 is the current completed milestone.
- Time-display formatting now has its own gameplay time module instead of living inside `game/actions.rs`.
- Gameplay behavior, save compatibility, and screen flow remain unchanged.

## Next

### v0.30.0 — Gameplay Action Architecture Cleanup II

- Continue decomposing `game/actions.rs` only where a clear responsibility boundary improves maintainability.
- Avoid splitting files purely to reduce line counts.
- Preserve existing gameplay behavior, save compatibility, and screen flow.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major architectural work should preserve backward compatibility where practical and include focused tests for behavior affected by the refactor.
