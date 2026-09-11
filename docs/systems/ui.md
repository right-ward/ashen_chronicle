# UI System

The shipped frontend is Bevy. The UI layer is built around renderer-neutral presentation views and semantic interaction events so gameplay systems do not depend on GUI details.

## Screen architecture

The Bevy frontend uses dedicated screen flows for start, save selection, character creation, gameplay, navigation, records, combat, developer console, quit, and death. `NavigationState` tracks the active screen and return destination so nested views can return to the correct parent.

Lifecycle screens are owned by `bevy_lifecycle.rs`. Gameplay and world navigation are owned by `bevy_gameplay.rs`. Character, inventory, quest, meditation, history, and journal flows are owned by `bevy_records.rs`. Combat is owned by `bevy_combat.rs`, and the developer console is owned by `bevy_console.rs`.

## Presentation boundary

`presentation.rs` contains frontend-independent view models. These models carry domain data such as strings, numbers, identifiers, and collections without depending on Bevy or any terminal UI library.

Bevy presentation adapters turn those views into UI nodes. Shared node construction, visual theme constants, screen roots, panels, labels, choice buttons, gauges, navigation state, and semantic input routing live in `bevy_presentation.rs`.

## Input

Native Bevy keyboard and button interaction is translated into the semantic `InputEvent` model in `input.rs`. Gameplay and screen systems consume these semantic events rather than Bevy-specific keyboard types except where text entry requires native `KeyboardInput` text data.

Arrow keys, Home/End, Page Up/Page Down, Enter, Escape, Tab, Backspace, Delete, and selected number/vim-style shortcuts remain available where each screen supports them. UI buttons produce the same semantic confirmation events as keyboard input.

## Gameplay and results

The gameplay dashboard presents the current world context, recent history, player health, available actions, and short-lived action messages. World navigation is a dedicated state within the gameplay flow.

Combat presents player/enemy status, encounter events, action choices, and result details through Bevy nodes while the authoritative combat system continues to own resolution and state mutation.

Record screens present character data, inventory details, quests, meditation choices/results, history entries/details, and journal editing. Results stay on-screen until the user advances or returns to the parent flow.

## Developer console

The developer console uses renderer-neutral `ConsoleView` data. Command editing, history navigation, completion candidates, scrolling, and command execution state remain in the game console modules; `bevy_console.rs` is responsible for graphical presentation and input handling.

The console opens from the gameplay flow and closes back to gameplay without a terminal-mode transition.

## Visual design

The shared Bevy presentation layer uses a restrained dark theme, panels, labels, selected-choice indicators, and health gauges. Text remains the primary presentation medium, with optional world/location artwork represented by view data where available.

## Design direction

Keep screen responsibilities separate from gameplay mechanics. New UI work should reuse frontend-neutral view models and semantic input events rather than introducing renderer-specific dependencies into gameplay rules. Avoid maintaining parallel terminal and graphical implementations.
