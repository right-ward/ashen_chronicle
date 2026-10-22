# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

**v0.51.23 — release complete.** Android text-input and graphical gameplay stabilization are complete. The developer console has explicit input ownership and a Close control.

**v0.52.x — location banner prototype complete.** Ashen Gate uses a full-width pixel-art PNG banner with nearest-neighbor sampling; other locations and banner-load failures retain the ASCII fallback.

## Active milestone

**v0.52.x — Procedural spatial exploration prototype** ([tracking issue #353](https://github.com/right-ward/ashen_chronicle/issues/353))

The first implementation slice is tracked in [#354 — spatial state and deterministic local-map generation](https://github.com/right-ward/ashen_chronicle/issues/354), [#355 — continuous movement and discovery](https://github.com/right-ward/ashen_chronicle/issues/355), and [#356 — exploration persistence and inheritance](https://github.com/right-ward/ashen_chronicle/issues/356).

The renderer-neutral map model and deterministic generator are in place, alongside a first Bevy exploration view for dangerous locations, continuous keyboard/touch movement with blocked-terrain collision, angled tile projection, depth ordering, and a contextual point-of-interest interaction recorded in world history. World-persistent local maps now live in the saved world state; exact active exploration position/session live with the character. The current follow-up adds a viewport-scaled camera dead zone and map bounds that account for the upward extension of wall geometry. This is map-aware soft follow, not full room-aware transitions: explicit room/door topology and representative prop occlusion are not implemented yet. Desktop/Android end-to-end validation remains outstanding. Spatial controls now expose eight-way directional input for diagonal movement, and UI scroll panels consume desktop mouse-wheel input in addition to touch drags. Button actions now use Bevy's completed-click activation event rather than the button-down `Pressed` state, and touch-scroll gestures suppress activation of the button they began on. Spatial exploration uses an Android-only analog virtual joystick with a dead zone and pressure-scaled movement; desktop movement is keyboard-driven with no on-screen directional pad.

Keep the existing world model, quests, time, narrative, and location-level actions authoritative. Spatial exploration supplements rather than replaces them. Real-time spatial combat and audio remain outside this milestone.

Related design issues: [#228 — long-term hybrid exploration](https://github.com/right-ward/ashen_chronicle/issues/228), [#229 — incremental pixel-art direction](https://github.com/right-ward/ashen_chronicle/issues/229), and [#252 — spatial combat design](https://github.com/right-ward/ashen_chronicle/issues/252).

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
