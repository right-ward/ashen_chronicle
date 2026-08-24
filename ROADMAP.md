# The Ashen Chronicle

## Roadmap

The roadmap tracks current and upcoming development. Detailed completed milestone history is kept in [`docs/roadmap-history.md`](docs/roadmap-history.md).

## Current state

- v0.36.0: Echoes of the Ashen Road expansion
- Added the Resonant Forge and Hollow Caravan as the expansion's opening locations.
- Expanded the road through Broken Stage, Library of Loops, Silence Spire, Endless Corridor, and the Pit.
- Added expansion encounters, NPCs, faction-linked quests, atmospheres, item art, and location-specific travel events.
- Adapted the old music/metal concepts around an in-world traveling musical tradition, haunted performances, resonance, memory, repetition, and silence rather than modern-world imagery.
- Delivered the expansion through the existing stable-ID mod/content system without changing the gameplay engine or save format.
- Kept the deeper locations in a companion content pack so they can extend and override the opening expansion content through the same loader rules.

## Next

### v0.37.0 — Quest Depth and World Consequences

- Expand the quest system beyond single-item turn-ins with explicit objective state and more meaningful world consequences.
- Use the existing event, history, faction, and NPC-memory systems so quest outcomes can affect the persistent world.
- Preserve save compatibility and keep new quest data validated before entering runtime state.

## Longer-term direction

Continue modularizing the codebase by responsibility rather than by file size alone. Keep runtime state, content loading, gameplay actions, presentation, persistence, and event processing independently understandable and testable.

Major gameplay work should remain data-driven where practical, preserve backward compatibility, and include focused tests for behavior affected by the change.
