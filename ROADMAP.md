# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

### v0.42.0: world navigation presentation

- Replaced the direct Travel menu flow with a dedicated fullscreen World Navigation screen.
- Shows the player's current location, region, description, danger state, and available routes in one focused view.
- Reuses existing location scene art on the navigation screen when it is available.
- Route choices identify dangerous destinations and include a short destination description.
- Keeps the existing travel effects, including time advancement, conditions, threats, quest synchronization, history, and travel events.
- Preserved the existing Back behavior so leaving navigation returns to the normal gameplay dashboard without changing state.

## Next

### v0.43.0 - 0.46.0

- Continue modularizing and improving the existing TUI presentation before the longer-term GUI transition.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
