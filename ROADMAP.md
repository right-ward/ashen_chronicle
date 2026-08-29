# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

- v0.39.0: UI polish and patching and `logkeys` dev command
- Developer-console output and completion menus support bounded scrolling and keep overflowing content navigable, including wrapped output lines.
- Developer-console implementation is split into focused entrypoint, UI/state, and command modules for easier maintenance.
- Added the `logkeys [true|false]` developer-console command. When enabled, it buffers terminal key events outside the developer console with timestamps, event kinds, key codes, and modifiers; console input itself is excluded.
- Fixed in-game ASCII art flattening by preserving leading whitespace in menu, location, and journal renderers. (#57)
- Fixed transparent/stacked game screens by clearing the full terminal frame before rendering standalone menu and developer-console screens. (#60)
### v0.39.1: patch console and actions layout
- Restored the in-game action menu to the final dashboard panel instead of drawing it as a detached overlay.
- Restored clean developer-console screen lifecycle: the console opens on a cleared screen and returns to the preserved game alternate screen without stacking frames.

## Next

### v0.40.0

- Continue modularizing and improving the existing TUI presentation before the longer-term GUI transition.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
