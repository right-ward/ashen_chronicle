# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

### v0.41.0: dedicated quest representation

- Replaced the log-style quest review with a dedicated player-facing fullscreen quest screen.
- Quest index entries clearly distinguish `ACTIVE`, `READY`, and `COMPLETED` states.
- Quest details show the description, objective progress, and reward without mixing into the gameplay journal.
- Returning from the quest screen restores the normal gameplay dashboard without changing quest or gameplay state.
- Kept existing hidden-quest filtering, objective tracking, completion behavior, and save compatibility unchanged.

## Next

### v0.42.0 - 0.46.0

- Continue modularizing and improving the existing TUI presentation before the longer-term GUI transition.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
