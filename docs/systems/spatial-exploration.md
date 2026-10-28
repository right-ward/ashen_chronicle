# Spatial Exploration

## Responsibility boundaries

`spatial.rs` owns renderer-neutral local-map data, terrain walkability, stable point-of-interest identity, deterministic generation, map exits, discovery state, and the character's logical exploration session. It must not depend on Bevy rendering or UI types.

`bevy_spatial.rs` adapts that model to the Bevy frontend. It owns angled projection, terrain and wall-face rendering, depth ordering, movement input from keyboard and touch controls, collision checks, camera follow, contextual interaction feedback, and synchronization with the active game state. The saved world remains authoritative for persistent maps; the active character owns the exact continuous position.

## Coordinate conventions

- Grid coordinates are continuous during movement. A tile's center is represented by `(x + 0.5, y + 0.5)`.
- Grid/world coordinates are authoritative for movement, collision, discovery, and persistence. Projected screen-space coordinates are presentation-only.
- The fixed three-quarter projection maps the two grid axes onto opposing diagonal screen directions. Input directions are converted through the inverse projection so keyboard and touch share the same movement rules.
- Sprite transforms are positioned from the ground-contact point plus a visual vertical offset. Actor depth follows the continuous ground position instead of snapping to a whole tile.
- Floor, ash, and rubble diamonds are placed on a dedicated background depth layer so ground surfaces cannot draw over actors. Wall tops are sorted from their ground-contact line; wall faces are sorted from their front/base edge. This lets the wall occlude a character behind it while a character moving beyond the wall's base renders in front. When an actor and wall top share the same ground depth, the actor wins the tie so a wall's entire top diamond does not hide a character in an adjacent lateral position.
- Wall tops extend upward beyond the floor diamond. Camera bounds account for that asymmetric vertical extent rather than expanding both vertical edges equally.

## Movement and interactions

Movement uses a small ground-space collision footprint and resolves the two grid axes separately, allowing movement to slide along a blocked edge. Walls and static prop footprints block movement; floor, ash, and rubble remain walkable. Pillars occupy one cell and tables occupy two adjacent cells in one grid row. Prop footprints are stored with the persistent local map, and the same walkability query is used for interaction line-of-sight checks.

Point-of-interest interaction requires both a projected-distance range check and a clear path through walkable terrain. Interaction range and line of sight are distinct checks; future combat should not treat either as a substitute for attack collision or target eligibility.

## Physical map exits

A local map's `exits` are derived from its world location's existing connected-location IDs. Each passage stores a stable target location ID rather than relying on a name or checking the destination's danger flag. Passage placement is deterministic, passages are positioned on edge tiles, and generated walkable corridors connect them to the map's entry area.

A contextual passage action appears only when the character is within interaction range and has a clear approach. Pressing the contextual interaction key uses the nearest available passage, or examines the point of interest when no passage is in range. Directly activating a passage action applies the same range and approach validation.

For now, using a physical passage returns to the normal gameplay screen, matching the existing `Leave` action. It does not yet transfer the character to the destination location because spatial maps are not implemented for every connected location. The destination ID is nevertheless retained in persistent map state so passage use can later enter a supported destination, with a suitable fallback for locations that do not yet have spatial maps.

Older saved local maps without an `exits` field remain deserializable. On entry, missing/empty exit data is initialized from the current world location's connections.

## Visual assets

The first spatial pixel-art set lives under the selected game's `data/assets/spatial/` directory and includes wood-floor, ash, rubble, and wall-top tiles, separate left/right wall-face sprites, a character sprite, a passage marker, and dedicated pillar/table props. These PNGs are loaded from disk at runtime through the same validated asset loader used for UI icons and mod art; they are not compiled into the executable. Android seeds the selected game folder from bundled defaults and synchronizes its `data/assets/` directory with the chosen shared folder. The wood floor is a first-pass treatment for the Old Shrine prototype, not a claim that every location should use wood.

The interface uses named PNG icons from `data/assets/icons/ui/` for actions, menu controls, conditions, records, combat choices, and the sun/moon treatment. Text separators and ordinary labels use simple ASCII punctuation instead of decorative glyphs that may be missing from the default font. Mods may provide their own images under `data/mods/<folder>/assets/` and reference them with `mod:<folder>/<relative-path>.png`.

## Spatial HUD

The local exploration screen keeps a compact character-status panel in the header. It reads the active character's name, level, current/maximum health, and current conditions directly from authoritative game state. The shared health-gauge primitive supplies both the blood-fill indicator and numeric values. The conditions line names active conditions and explicitly says when none are present. The location title and contextual feedback remain separate from this panel, and the Leave/action controls and Android-only virtual joystick retain their existing roles.

This is the first HUD layer, not the complete reference layout. Combat telegraphs, enemy vitality, ability controls, a minimap/compass, and local ally orders are not implemented by this change.

## Camera behavior

The camera uses a viewport-scaled dead zone: ordinary small player movements do not move the camera, while crossing the dead-zone edge shifts the target by only the excess distance. A smoothed follow approaches that target, and map bounds keep the viewport inside the projected map where possible. Maps smaller than the viewport are centered.

This is not yet full room-aware camera behavior. Local maps currently have no explicit room graph, per-room bounds, or door-to-room transitions. Physical edge passages are world-exit markers, not an interior room system. Room-aware camera behavior should only be introduced alongside a defined room/door model, then tested for smooth transitions and for avoiding views outside playable spaces.

## Validation status and next gates

Automated unit tests cover deterministic map generation, map serialization and backward-compatible defaults, reachable physical exits, projection offsets, inverse input mapping, basic continuous movement, blocked-terrain collision, interaction range/approach, dead-zone behavior, map-bound camera constraints, equal-depth wall-top ordering, and UI icon fallbacks.

These tests are source-level coverage. They do not establish successful end-to-end play on desktop or Android; interactive testing and on-device verification are intentionally left to the repository owner.

The current prototype covers a limited generated pillar/table prop set plus wall tiles and physical edge passages. Before broad art production, validate a representative authored room containing intersecting walls, a doorway, a pillar, a table, and a player moving in front of and behind those objects. That next gate should prove projection alignment, ground-based depth sorting, deliberate collision footprints, reachable interactions, and stable camera framing. Full room/door topology and authoring tools remain future work.
