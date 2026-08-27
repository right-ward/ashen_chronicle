# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

- v0.38.0: Content loading and world seeding hardening
- Unified base-content and mod discovery around a single resolved `data/` root so the loader cannot silently combine files from different roots.
- Added explicit data-root candidate diagnostics and warnings for missing, unreadable, or malformed external content.
- Strengthened campaign seeding so existing worlds reconcile newly loaded locations, metadata, exits, and persistent campaign entities instead of relying on first-creation state only.
- Exposed the content loading report through the content module for developer diagnostics and future tooling.
- CI reliability: automatic rustfmt commits are retained, while formatting no longer gates test execution; CI also runs when workflow, Cargo, or data files change.

## Next

### v0.39.0 — Quest Depth and World Consequences

- Expand the quest system beyond single-item turn-ins with explicit objective state and more meaningful world consequences.
- Use the existing event, history, faction, and NPC-memory systems so quest outcomes can affect the persistent world.
- Preserve save compatibility and keep new quest data validated before entering runtime state.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
