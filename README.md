# The Ashen Chronicle

A dark fantasy RPG built in Rust, focused on exploration, consequence, character death, and a world that remembers what happened. The shipped game uses a Bevy graphical frontend while keeping gameplay and presentation models independent of the engine.

The game is inspired by the atmosphere and themes of dark fantasy: dangerous roads, forgotten places, strange characters, difficult choices, and a world that does not simply reset when a character dies.

## Features

- Procedurally generated world
- Character creation and inheritance
- Exploration and travel
- Threats and encounters
- Turn-based combat
- World reset and inheritance options on death
- Persistent world changes
- History/event tracking
- NPC memory, reactions, and faction reputation
- Quest system
- Base content loading and mod support foundation
- Bevy graphical user interface
- Keyboard interaction and clickable choices
- Optional ASCII portraits, item art, and location scenes

## Core Concept

Death is not simply a game over.

When a character dies, their actions can leave traces behind. Their remains, deeds, consequences, and changes to the world can persist.

A later character may inherit that world and encounter what the previous character left behind.

The goal is to make the world feel persistent rather than disposable.

## Project Structure

```text
ashen_chronicle
├── data
│   ├── mods
│   │   ├── ashen_expansion
│   │   │   ├── content.json
│   │   │   └── manifest.json
│   │   ├── echoes_depth
│   │   │   ├── content.json
│   │   │   └── manifest.json
│   │   └── README.md
│   └── base_content.json
├── docs
│   ├── systems
│   │   ├── content.md
│   │   ├── events.md
│   │   ├── persistence.md
│   │   └── ui.md
│   ├── README.md
│   ├── architecture.md
│   ├── development-plan-history.md
│   └── roadmap-history.md
├── src
│   ├── content
│   │   ├── definitions.rs
│   │   ├── diagnostics.rs
│   │   ├── loader.rs
│   │   └── seeding.rs
│   ├── game
│   │   ├── actions.rs
│   │   ├── character.rs
│   │   ├── combat.rs
│   │   ├── console.rs
│   │   ├── console_commands.rs
│   │   ├── console_ui.rs
│   │   ├── history_screen.rs
│   │   ├── interactions.rs
│   │   ├── legacy.rs
│   │   ├── menu.rs
│   │   ├── navigation.rs
│   │   ├── quests.rs
│   │   ├── records.rs
│   │   ├── state_effects.rs
│   │   ├── time.rs
│   │   └── world.rs
│   ├── bevy_app.rs
│   ├── bevy_combat.rs
│   ├── bevy_console.rs
│   ├── bevy_gameplay.rs
│   ├── bevy_interactions.rs
│   ├── bevy_lifecycle.rs
│   ├── bevy_navigation_bridge.rs
│   ├── bevy_presentation.rs
│   ├── bevy_records.rs
│   ├── content.rs
│   ├── events.rs
│   ├── game.rs
│   ├── input.rs
│   ├── main.rs
│   ├── model.rs
│   ├── persistence.rs
│   ├── presentation.rs
│   ├── procedural.rs
│   ├── procedural_authored.rs
│   ├── procedural_characteristics.rs
│   ├── procedural_entities.rs
│   ├── procedural_opportunities.rs
│   ├── procedural_relationships.rs
│   ├── rng.rs
│   └── ui.rs
├── AGENTS.md
├── Cargo.lock
├── Cargo.toml
├── DEVELOPMENT_PLAN.md
├── LICENSE
├── README.md
└── ROADMAP.md
```

`main.rs` starts the Bevy application. `bevy_app.rs` configures the engine, while the other `bevy_*.rs` modules adapt lifecycle, gameplay, records, combat, console, and interaction flows to the shared presentation layer.

`game.rs` contains the gameplay module façade and core gameplay modules.

`model.rs` contains the core game data structures and world model.

`persistence.rs` handles saving and loading the world and preserves compatibility with older save formats.

`presentation.rs` contains frontend-independent view models, while `input.rs` contains frontend-neutral semantic interaction events.

## Saves

Save files are stored as gzip-compressed JSON using a character-specific filename such as `ashen_chronicle_save_Ash Walker.json.gz`. Existing `ashen_chronicle_save.json` saves from earlier versions remain readable.

## Building

The project uses Rust, Cargo, and Bevy.

Build the project with:

```sh
cargo build -r
```

Run the game with:

```sh
cargo run -r
```

or run the built release directly:

```sh
./ashen_chronicle
```

Run the test suite with:

```sh
cargo test
```

## Design Philosophy

#### The world should remember.
Important actions should have consequences that can survive beyond a single character.

#### Death should matter.
Character death is part of the game's progression rather than merely a failure state.

#### Systems should interact.
Quests, factions, NPCs, locations, combat, inventory, history, and world state should gradually become interconnected rather than existing as isolated mechanics.

#### Content should eventually be data-driven.
As the game grows, adding content should require less modification of the underlying engine.

#### Presentation should enhance the world.
The Bevy frontend is intended to improve readability and platform reach without replacing the text-first narrative and system design.

#### Keep the project maintainable.
Development should proceed incrementally, with the roadmap tracking completed milestones and semantic versioning tracking releases.

## Versioning

The project follows semantic versioning:

MAJOR.MINOR.PATCH

Development releases remain below `1.0.0` while the core systems and content are still being established.

## License

See [`LICENSE`](./LICENSE) for the project's license information.

---

The world remembers what you leave behind.
