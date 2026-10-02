# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state
v0.51.11 development — unified gameplay pause/navigation menu

Startup regression fixed: the graphical frontend no longer aborts on Bevy UI health-gauge query validation.

## Next

v0.51.x — continue the graphical frontend toward the planned real-time spatial combat work.

Completed in this iteration:
- #319 — add an Android long-press gesture for the developer console
- #317 — unify the gameplay burger menu and pause screen, including Android Back navigation
- #314 — restore Start screen Options navigation
- #316 — fix Load screen Back navigation and migrate its controls to the current shared button style
- #315 — migrate legacy graphical buttons to the current shared button style
- #313 — make Android Quit safely terminate the activity/process so relaunch starts cleanly
- #298 — restore user-facing graphical text rendering
- #299 — display the contextual quit-confirmation question
- #300 — restore persisted desktop game-data root startup resolution
- #301 — restore compact Game Data options controls
- #302 — add Options to the graphical start screen
- Android build regression — keep desktop-only game root resolution out of Android compilation
- #305 — keep conversation time and character turn progression consistent
- #306 — keep remains-search time and turn progression consistent
- #307 — preserve pause-menu selection when returning from Options
- #308 — remove stale gameplay navigation bridge mapping
- #309 — keep world view construction renderer-neutral
- #310 — centralize semantic choice-button activation
- #311 — make the gameplay menu button satisfy the shared touch-target contract

Previous v0.51.3 regression fixes:
- #291 — persist the resolved desktop game-data root after first-launch selection
- #292 — constrain the gameplay sky to the world presentation area
- #293 — keep shared graphical choices/options content within its parent bounds
- #294 — keep event metadata out of transient gameplay messages
- #295 — restore the dedicated graphical quit presentation and wire choice activation correctly
- #278 — shared graphical UI foundation
- #279 — world-first gameplay HUD redesign
- #227 — dedicated tabbed Options screen and configurable game data root
- Android configuration persistence mirror — private app config with shared game-folder synchronization
- #226 — gameplay pause menu with Resume, lifecycle actions, Options, and Quit confirmation

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
