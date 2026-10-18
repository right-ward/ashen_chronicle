# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

**v0.51.23 — release complete.** Android text-input and graphical gameplay stabilization are complete. The developer console has explicit input ownership and a Close control.

**v0.52.x — location banner prototype complete.** Ashen Gate uses a full-width pixel-art PNG banner with nearest-neighbor sampling; other locations and banner-load failures retain the ASCII fallback.

## Active milestone

**v0.52.x — Procedural spatial exploration prototype** ([tracking issue #353](https://github.com/right-ward/ashen_chronicle/issues/353))

The first implementation slice is tracked in [#354 — spatial state and deterministic local-map generation](https://github.com/right-ward/ashen_chronicle/issues/354), [#355 — continuous movement and discovery](https://github.com/right-ward/ashen_chronicle/issues/355), and [#356 — exploration persistence and inheritance](https://github.com/right-ward/ashen_chronicle/issues/356).

The current focus is a renderer-neutral, bounded local-map model with deterministic generation and a guaranteed reachable point of interest. The playable Bevy view, movement, interaction, and persistence integration follow in their respective tasks.

Keep the existing world model, quests, time, narrative, and location-level actions authoritative. Spatial exploration supplements rather than replaces them. Real-time spatial combat and audio remain outside this milestone.

Related design issues: [#228 — long-term hybrid exploration](https://github.com/right-ward/ashen_chronicle/issues/228), [#229 — incremental pixel-art direction](https://github.com/right-ward/ashen_chronicle/issues/229), and [#252 — spatial combat design](https://github.com/right-ward/ashen_chronicle/issues/252).

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
