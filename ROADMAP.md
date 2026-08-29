# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

- v0.39.0: UI and input reliability — in progress
- Unified base-content and mod discovery around a single resolved `data/` root so the loader cannot silently combine files from different roots.
- Added explicit data-root candidate diagnostics and warnings for missing, unreadable, or malformed external content.
- Strengthened campaign seeding so existing worlds reconcile newly loaded locations, metadata, exits, and persistent campaign entities instead of relying on first-creation state only.
- Exposed the content loading report through the content module for developer diagnostics and future tooling.
- CI reliability: automatic rustfmt commits are retained, while formatting no longer gates test execution; CI also runs when workflow, Cargo, or data files change.
- Input handling: UI and developer-console key processing now accepts only `KeyEventKind::Press` events, preventing non-press terminal events from being interpreted as duplicate input.

## Next

### v0.39.0 — remaining

- Fix in-game ASCII art so leading spaces are preserved instead of being flattened. (#57) — implementation queued
- Fix transparent/stacked game screens so each screen is cleared and rendered independently. (#60)

### v0.40.0

- Continue modularizing and improving the existing TUI presentation before the longer-term GUI transition.

## Completed in v0.39.0

- Developer-console output and completion menus support bounded scrolling and keep overflowing content navigable, including wrapped output lines.
- Developer-console implementation is split into focused entrypoint, UI/state, and command modules for easier maintenance.
- Added the `logkeys [true|false]` developer-console command. When enabled, it buffers terminal key events outside the developer console with timestamps, event kinds, key codes, and modifiers; console input itself is excluded.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
