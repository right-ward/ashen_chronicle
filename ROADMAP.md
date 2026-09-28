# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state
v0.51.0 development — shared graphical UI foundation complete

## Next

v0.51.x milestone (#262)

Completed in this iteration:
- #278 — shared graphical UI foundation
- #279 — world-first gameplay HUD redesign
- #227 — dedicated tabbed Options screen and configurable game data root
- Android configuration persistence mirror — private app config with shared game-folder synchronization

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
