# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

**v0.51.23 — release complete.** Android text-input and graphical gameplay stabilization are complete. The developer console has explicit input ownership and a Close control.

**v0.52.x — Phase 1: location banner prototype.** Ashen Gate now uses a full-width pixel-art PNG banner with nearest-neighbor sampling; other locations and banner-load failures retain the ASCII fallback.

## Next

**v0.52.x — Phase 2: spatial exploration prototype**

Keep the first implementation deliberately small and validate one complete exploration loop before expanding the system:

- Define the minimum tile/grid representation, camera and coordinate rules, movement, interaction, and state/persistence boundary.
- Connect the existing Explore action to one dangerous location's spatial view.
- Support movement and one meaningful point of interest or encounter, then return to the existing gameplay screen with world state intact.
- Keep the current world model, quests, time, narrative, and location-level actions authoritative; spatial exploration supplements rather than replaces them.
- Align the environment art and layout with the incremental pixel-art direction in issue #229, with desktop and Android landscape behavior in mind.

Related design issues: [#228 — hybrid world exploration](https://github.com/right-ward/ashen_chronicle/issues/228), [#229 — pixel-art visual direction](https://github.com/right-ward/ashen_chronicle/issues/229), and [#252 — future spatial combat design](https://github.com/right-ward/ashen_chronicle/issues/252). Keep real-time combat and audio as separate design/implementation steps rather than prerequisites for the first exploration loop.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
