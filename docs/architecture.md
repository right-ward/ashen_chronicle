# The Ashen Chronicle — Architecture

## Purpose

This document defines the structural boundaries of the game. It describes where responsibilities live, how systems depend on one another, and the rules that keep the codebase modular.

## Architectural goals

The architecture is built around four principles:

- World simulation is independent from presentation.
- Gameplay content is data-driven rather than embedded in engine logic.
- Persistent world state is separate from character-specific state where possible.
- Systems communicate through explicit responsibilities instead of reaching into one another's internals.

The project should prefer clear responsibility boundaries over splitting modules merely to reduce line counts.

## High-level layers

```text
Application / Game Flow
        │
        ▼
Runtime & Dispatch
        │
        ├── Gameplay Actions
        ├── Gameplay Interactions
        ├── Character Progression
        ├── Legacy / Death
        ├── Combat
        ├── Lifecycle / Screens
        └── World / Bootstrap
        │
        ▼
Core State & Models
        │
        ├── World state
        ├── Character state
        ├── Entities
        └── Historical state
        │
        ├───────────────┐
        ▼               ▼
Content / Events    Persistence
```

Content loading supplies definitions to the runtime but should not own live world state. Presentation reads state and produces output but should not implement game rules. Persistence serializes and restores state but should not decide gameplay outcomes.

## Current module direction

The current codebase has been progressively decomposed from large modules into responsibility-based modules. The intended structure is approximately:

```text
src/
├── main.rs
├── game.rs                 # top-level game entry / compatibility facade
├── game/
│   ├── runtime.rs          # gameplay loop and turn flow
│   ├── dispatcher.rs       # GameAction dispatch
│   ├── actions.rs          # core gameplay actions and action-specific logic
│   ├── character.rs        # character progression and character-sheet presentation
│   ├── interactions.rs     # NPC dialogue, quest interaction, faction memory/reputation
│   ├── legacy.rs           # death, corpses, previous-life recovery, legacy item presentation
│   ├── lifecycle.rs        # start/load/creation/quit/death lifecycle flows
│   ├── combat.rs           # combat encounter processing
│   ├── console.rs          # developer-console lifecycle and terminal integration
│   ├── console_ui.rs       # console interaction state, view construction, and terminal rendering
│   └── world.rs            # world bootstrap and loaded-state validation
├── presentation.rs         # frontend-independent presentation/view models
├── content.rs              # content module facade
├── content/
│   ├── definitions.rs      # schemas and validation definitions
│   ├── loader.rs           # base/mod loading and merging
│   └── seeding.rs          # content-to-world translation
├── events.rs               # event runtime
├── model.rs                # shared game-state and entity models
├── persistence.rs          # save/load and migrations
├── input.rs                # frontend-neutral interaction events
├── ui.rs                   # terminal UI facade and state bridge
├── ui_impl.rs              # ratatui/crossterm implementation
└── ui_components.rs        # reusable terminal UI rendering primitives
```

The exact module list may evolve, but new modules should represent meaningful responsibilities rather than arbitrary slices of large files.

## Game flow

`main.rs` starts the application. `game.rs` provides the top-level game entry point. The runtime owns the main loop and coordinates turn lifecycle, while the dispatcher maps player-selected actions to their implementations.

Gameplay actions operate on the model and relevant systems. Character progression owns experience gain, level advancement, and character-sheet presentation. Gameplay interactions own NPC dialogue, quest offering/turn-in, faction memory/reputation updates, and NPC availability. Legacy gameplay owns character death, corpse creation, corpse recovery, and previous-life item recovery. Combat is isolated from general action handling. Lifecycle logic owns start/load/creation/quit/death flows. World/bootstrap logic owns world initialization and validation. Presentation renders the current state and contextual results. The developer console owns its command/session lifecycle while exposing renderer-neutral console data to its terminal renderer.

This keeps the main runtime readable without duplicating state-management logic across screen and action code.

## State ownership

The model represents the authoritative simulation state. Systems should mutate state through explicit functions belonging to the appropriate owner.

World-level state includes persistent locations, factions, world history, event cooldowns, corpses, placed items, and other changes that survive character death.

Character-level state includes attributes, experience, conditions, inventory, active quests, and other properties belonging to the current life.

When a character dies and a world is inherited, character-specific state is discarded or intentionally reconstructed while persistent world state remains.

## Entity identity

Every persistent entity uses a stable unique ID. References should use IDs rather than display names.

Names, descriptions, and other presentation fields are not identity. This allows content to be renamed without breaking relationships and makes save compatibility and mod merging more predictable.

## Content architecture

Gameplay definitions are loaded from structured content rather than hardcoded throughout runtime code.

The content layer is responsible for:

- Definitions and schemas.
- Content validation.
- Base content loading.
- Mod discovery and loading.
- Merging by stable identifiers and keys.
- Translation of definitions into initial world state.

The runtime consumes loaded content and should not need to know whether a definition came from the base pack or a mod.

See [`systems/content.md`](systems/content.md) for content-specific details.

## Event architecture

Events are data-driven and executed by a reusable runtime. Definitions can specify triggers, weights, chance gates, conditions, effects, and cooldowns. Event execution can produce persistent world changes and structured history records.

The event system should remain independent of individual hardcoded travel or quest branches.

See [`systems/events.md`](systems/events.md) for details.

## Persistence architecture

Persistence serializes the authoritative world/character state and restores it into a valid runtime state. Save compatibility and migrations belong to the persistence boundary.

Campaign content itself is runtime data and should not be redundantly embedded in save files when it can be safely reloaded from the current content definitions.

## Presentation architecture

Presentation consumes authoritative state and produces frontend-independent view data before any terminal- or GUI-specific rendering occurs. The root `presentation.rs` module contains shared view models expressed only through domain-neutral owned data such as strings, scalars, and collections; it does not depend on ratatui, crossterm, or gameplay actions. Screen and gameplay modules are responsible for constructing these models from authoritative state, while frontend renderers decide how the models are visually represented.

Lifecycle screens use `ScreenView` and `ChoiceView`, death details use `DeathView`, corpse recovery uses `RemainsView` and `RemainsResultView`, and the developer console is rendered from `ConsoleView`. These models keep the data required by the renderer out of terminal-specific implementations.

The terminal UI is split between the `ui.rs` facade, the `ui_impl.rs` ratatui/crossterm implementation, and `ui_components.rs` reusable terminal rendering primitives. Components such as compact-layout detection, bottom-panel sizing, panel construction, scrolling text, message panels, health gauges, frame clearing, and shared layout spacing belong in `ui_components.rs` so screen migrations can reuse consistent behavior without copying renderer details.

The terminal interface uses ratatui and supports responsive layouts for narrow and wide terminals. Lifecycle screens remain separate from the gameplay dashboard so start, load, character creation, quit, and death do not unnecessarily render gameplay underneath them.

The `input.rs` boundary translates terminal keyboard values into semantic `InputEvent` values before game-facing interaction code consumes them. A graphical frontend can provide equivalent events without exposing keyboard or crossterm details upstream.

Character-sheet presentation is owned by the character module because it is directly tied to character progression state rather than the general action dispatcher.

Legacy mechanics remain independent from lifecycle screens: `legacy.rs` owns death/corpse state changes and recovery data construction, while `lifecycle.rs` coordinates the death flow itself.

See [`systems/ui.md`](systems/ui.md) for details.

## Dependency rules

A system should depend on abstractions or shared models appropriate to its responsibility, not on unrelated implementation details.

In particular:

- Presentation should not implement simulation rules.
- Persistence should not decide gameplay outcomes.
- Content loading should not directly own runtime character state.
- World/bootstrap code should not depend on gameplay action implementations merely to perform world initialization.
- Actions should not duplicate combat, interaction, progression, legacy, presentation, or persistence logic that already has a dedicated owner.
- Gameplay interactions may use action-owned turn/progression helpers where those helpers are still shared gameplay infrastructure, but interaction-specific rules belong in `interactions.rs`.
- Legacy mechanics should remain independent from lifecycle screens and own only death, corpse, and previous-life recovery responsibilities.
- Character progression should remain independent from world/bootstrap and persistence implementation details.
- Shared models should remain focused on state and domain representation rather than becoming a catch-all service module.
- Terminal screens should use shared UI primitives rather than duplicate generic panel, gauge, scrolling, and responsive-layout behavior.

## Compatibility and refactoring

Refactoring should preserve gameplay behavior, save compatibility, and screen flow unless the change explicitly intends to alter them.

When a module becomes large, first identify cohesive responsibilities and move them behind clear interfaces. Do not split code solely because a file has many lines.

Architecture changes should include focused tests around affected behavior and should avoid introducing parallel state systems.

## Documentation boundaries

`ROADMAP.md` describes current and upcoming milestones.

`DEVELOPMENT_PLAN.md` describes development strategy, priorities, and implementation rules.

This file describes structural architecture and responsibility boundaries.

The files under `docs/systems/` describe individual systems in greater detail.

Historical milestone records remain in the dedicated history documents rather than being repeated throughout the active documentation.
