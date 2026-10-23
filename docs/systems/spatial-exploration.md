# Spatial Exploration

## Responsibility boundaries

`spatial.rs` owns renderer-neutral local-map data, terrain walkability, stable point-of-interest identity, deterministic generation, map exits, discovery state, and the character's logical exploration session. It must not depend on Bevy rendering or UI types.

`bevy_spatial.rs` adapts that model to the Bevy frontend. It owns angled projection, terrain and wall-face rendering, depth ordering, movement input from keyboard and touch controls, collision checks, camera follow, contextual interaction feedback, and synchronization with the active game state. The saved world remains authoritative for persistent maps; the active character owns the exact continuous position.

## Coordinate conventions

- Grid coordinates are continuous during movement. A tile's center is represented by `(x + 0.5, y + 0.5)`.
- Grid/world coordinates are authoritative for movement, collision, discovery, and persistence. Projected screen-space coordinates are presentation-only.
- The fixed three-quarter projection maps the two grid axes onto opposing diagonal screen directions. Input directions are converted through the inverse projection so keyboard and touch share the same movement rules.
- Sprite transforms are positioned from the ground-contact point plus a visual vertical offset. Depth ordering uses the ground position, not the sprite's upper bounds.
- Wall tops extend upward beyond the floor diamond. Camera bounds account for that asymmetric vertical extent rather than expanding both vertical edges equally.

## Movement and interactions

Movement uses a small ground-space collision footprint and resolves the two grid axes separately, allowing movement to slide along a blocked edge. Walls block movement; floor, ash, and rubble remain walkable. This terrain-only collision model is a prototype and does not yet define authored collision footprints for decorative props.

Point-of-interest interaction requires both a projected-distance range check and a clear path through walkable terrain. Interaction range and line of sight are distinct checks; future combat should not treat either as a substitute for attack collision or target eligibility.

## Physical map exits

A local map's `exits` are derived from its world location's existing connected-location IDs. Each passage stores a stable target location ID rather than relying on a name or checking the destination's danger flag. Passage placement is deterministic, passages are positioned on edge tiles, and generated walkable corridors connect them to the map's entry area.

A contextual passage action appears only when the character is within interaction range and has a clear approach. Pressing the contextual interaction key uses the nearest available passage, or examines the point of interest when no passage is in range. Directly activating a passage action applies the same range and approach validation.

For now, using a physical passage returns to the normal gameplay screen, matching the existing `Leave` action. It does not yet transfer the character to the destination location because spatial maps are not implemented for every connected location. The destination ID is nevertheless retained in persistent map state so passage use can later enter a supported destination, with a suitable fallback for locations that do not yet have spatial maps.

Older saved local maps without an `exits` field remain deserializable. On entry, missing/empty exit data is initialized from the current world location's connections.

## Visual assets

The first spatial pixel-art set lives under `data/assets/spatial/` and includes separate wood-floor, ash, rubble, and wall-top tiles, a character sprite, and a passage marker. The images are embedded at build time and decoded with nearest-neighbor sampling so desktop and Android share the same assets without relying on runtime asset-directory discovery. The wood floor is a first-pass visual treatment for the Old Shrine prototype, not a claim that every location should use wood.

UI icon labels pass through a centralized ASCII fallback before reaching Bevy text rendering. This avoids missing-glyph rectangles when the bundled default font cannot render decorative Unicode symbols. The fallback is intentionally functional; it does not replace a future dedicated icon-sprite system or a broader font/art-direction decision.

## Camera behavior

The camera uses a viewport-scaled dead zone: ordinary small player movements do not move the camera, while crossing the dead-zone edge shifts the target by only the excess distance. A smoothed follow approaches that target, and map bounds keep the viewport inside the projected map where possible. Maps smaller than the viewport are centered.

This is not yet full room-aware camera behavior. Local maps currently have no explicit room graph, per-room bounds, or door-to-room transitions. Physical edge passages are world-exit markers, not an interior room system. Room-aware camera behavior should only be introduced alongside a defined room/door model, then tested for smooth transitions and for avoiding views outside playable spaces.

## Validation status and next gates

Automated unit tests cover deterministic map generation, map serialization and backward-compatible defaults, reachable physical exits, projection offsets, inverse input mapping, basic continuous movement, blocked-terrain collision, interaction range/approach, dead-zone behavior, map-bound camera constraints, and UI icon fallbacks.

These tests are source-level coverage. They do not establish successful end-to-end play on desktop or Android; interactive testing and on-device verification are intentionally left to the repository owner.

Before broad art production, validate a representative room containing intersecting walls, a doorway, a pillar, a table, and a player moving in front of and behind those objects. That test should prove projection alignment, ground-based depth sorting, deliberate collision footprints, reachable interactions, and stable camera framing. The current terrain-only prototype does not yet satisfy that broader room/prop validation gate.
