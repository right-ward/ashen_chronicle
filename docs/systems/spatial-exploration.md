# Spatial Exploration

## Responsibility boundaries

`spatial.rs` owns renderer-neutral local-map data, terrain walkability, stable point-of-interest identity, discovery state, deterministic generation, and the character's logical exploration session. It must not depend on Bevy rendering or UI types.

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

## Camera behavior

The camera uses a viewport-scaled dead zone: ordinary small player movements do not move the camera, while crossing the dead-zone edge shifts the target by only the excess distance. A smoothed follow approaches that target, and map bounds keep the viewport inside the projected map where possible. Maps smaller than the viewport are centered.

This is not yet full room-aware camera behavior. Local maps currently have no explicit room graph, doorway transitions, or per-room bounds. Those features should be introduced only alongside a defined room/door model, then tested for smooth transitions and for avoiding views outside playable spaces.

## Validation status and next gates

Automated unit tests cover projection offsets, inverse input mapping, basic continuous movement, blocked-terrain collision, interaction range/approach, dead-zone behavior, and map-bound camera constraints. These tests are source-level coverage; they do not establish successful desktop or Android end-to-end behavior.

Before broad art production, validate a small representative room containing intersecting walls, a doorway, a pillar, a table, and a player moving in front of and behind those objects. That test should prove projection alignment, ground-based depth sorting, deliberate collision footprints, reachable interactions, and stable camera framing. The current terrain-only prototype does not yet satisfy that complete gate.
