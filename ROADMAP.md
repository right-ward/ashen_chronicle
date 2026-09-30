# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state
v0.52.0 development — graphical frontend regression fixes and start-screen Options access completed in this iteration

Startup regression fixed: the graphical frontend no longer aborts on Bevy UI health-gauge query validation.

## Next

v0.51.x milestone (#262) — integration/regression/polish pass

v0.52.0 graphical frontend integration fixes

Completed in this iteration:
- #291 — persist the resolved desktop game-data root after first-launch selection
- #292 — constrain the gameplay sky to the world presentation area
- #293 — keep shared graphical choices/options content within its parent bounds
- #294 — keep event metadata out of transient gameplay messages
- #295 — restore the dedicated graphical quit presentation and wire choice activation correctly
- #298 — restore user-facing graphical text rendering
- #299 — display the contextual quit-confirmation question
- #300 — restore persisted desktop game-data root startup resolution
- #301 — restore compact Game Data options controls
- #302 — add Options to the graphical start screen
- #278 — shared graphical UI foundation
- #279 — world-first gameplay HUD redesign
- #227 — dedicated tabbed Options screen and configurable game data root
- Android configuration persistence mirror — private app config with shared game-folder synchronization
- #226 — gameplay pause menu with Resume, lifecycle actions, Options, and Quit confirmation

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
