# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

### v0.44.0: dedicated combat encounter presentation
- Replaced combat's generic action/log presentation with a dedicated combat encounter screen.
- Combat now presents the player and enemy together with current turn, encounter location, enemy power, available actions, and recent combat events.
- Player and enemy health remain displayed through the existing LineGauge presentation while combat is active.
- Combat results for victory, defeat, and fleeing are presented on the encounter screen before returning to normal gameplay or the death flow.
- Preserved existing combat rules, history recording, rewards, quest updates, and threat cleanup behavior.

## Next

### v0.45.0 - 0.46.0

- Continue modularizing and improving the existing TUI presentation before the longer-term GUI transition.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
