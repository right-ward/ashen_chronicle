# UI System

The shipped frontend is Bevy. The UI layer is built around renderer-neutral presentation views and semantic interaction events so gameplay systems do not depend on GUI details.

## Screen architecture

The Bevy frontend uses dedicated screen flows for start, save selection, character creation, gameplay, navigation, secondary navigation, pause, Options, NPC dialogue, remains recovery, records, combat, developer console, quit, and death. `NavigationState` tracks the active screen and return destination so nested views can return to the correct parent.

Lifecycle screens are owned by `bevy_lifecycle.rs`. Gameplay and world navigation are owned by `bevy_gameplay.rs`. NPC dialogue and remains recovery are owned by `bevy_interactions.rs`. Character, inventory, quest, meditation, history, and journal flows are owned by `bevy_records.rs`. Combat is owned by `bevy_combat.rs`, and the developer console is owned by `bevy_console.rs`.

## Presentation boundary

`presentation.rs` contains frontend-independent view models. These models carry domain data such as strings, numbers, identifiers, and collections without depending on Bevy or any terminal UI library.

Bevy presentation adapters turn those views into UI nodes. Shared node construction, semantic visual theme tokens, screen roots, surfaces, overlays, labels, action/menu controls, health gauges, condition indicators, contextual messages, navigation state, and semantic input routing live in `bevy_presentation.rs`.

The v0.51 graphical foundation keeps legacy panel helpers for existing screens while adding meaning-oriented primitives for the redesigned interface. New action and menu controls expose explicit interaction-state styling, health gauges keep their display values synchronized with their component state, and responsive spacing/touch-target tokens are centralized so new screens do not invent per-screen sizing rules. The dedicated Options screen reuses these primitives rather than introducing a separate menu architecture.

## Input

Native Bevy keyboard, touch, and button interaction is translated into the semantic `InputEvent` model in `input.rs`. Gameplay and screen systems consume these semantic events rather than Bevy-specific keyboard types except where text entry requires native `KeyboardInput` text data.

Android touch positions are logical window coordinates while Bevy UI layout hit-testing uses physical coordinates, so the shared touch handling converts touch positions with the window scale factor before testing UI bounds. This conversion is shared by custom choice and scroll hit-testing.

Lifecycle character-creation fields use Bevy 0.19.1's `EditableText` with real `InputFocus`, tab navigation, cursor/edit handling, and native IME routing. The shared presentation layer still gives the fields explicit touch targets so touch input can assign native focus. The custom IME-to-keyboard bridge remains only for the developer console, whose input model is still renderer-specific. Android IME dismissal clears native lifecycle focus before semantic Back navigation resumes.

Arrow keys, Home/End, Page Up/Page Down, Enter, Escape, Tab, Backspace, Delete, and selected number/vim-style shortcuts remain available where each screen supports them. UI buttons produce the same semantic confirmation events as keyboard input.

## Gameplay and results

The gameplay surface is world-first: location art/context occupies the dominant area, the 12-point world clock drives an obvious sun/moon sky treatment, and atmosphere can move into a short-lived contextual message. A compact top-right HUD carries health and active-condition indicators beside the secondary-navigation control. Primary actions remain below the world presentation, with contextual Investigate/Search Remains actions added only when the authoritative game state makes them available. Recent history is no longer a permanent gameplay panel; full history remains in its dedicated record screen. World navigation is a dedicated state within the gameplay flow.

NPC dialogue presents people at the current location, availability, faction and memory information, quest offering/turn-in choices, and conversation results. Remains recovery presents available corpses, recovered items, discovered hidden items, and recovery notes while keeping the authoritative corpse mutation in the game legacy module.

Combat presents player/enemy status, encounter events, action choices, and result details through Bevy nodes while the authoritative combat system continues to own resolution and state mutation.

Record screens present character data, inventory details, quests, meditation choices/results, history entries/details, and journal editing. Results stay on-screen until the user advances or returns to the parent flow.

## Developer console

The developer console uses renderer-neutral `ConsoleView` data. Command editing, history navigation, completion candidates, scrolling, and command execution state remain in the game console modules; `bevy_console.rs` is responsible for graphical presentation and input handling.

The console opens from the gameplay flow and closes back to gameplay without a terminal-mode transition.

## Visual design

The shared Bevy presentation layer uses a restrained dark theme with semantic surface, border, hover, pressed, selected, disabled, and overlay tokens. Legacy panels remain available for existing screens, while `spawn_surface` and `spawn_overlay` provide the non-dashboard structural primitives used by the v0.51 graphical redesign.

Meaning-oriented helpers include compact icon+text action buttons, compact menu buttons, health gauges with overlaid current/max values, condition indicators, and contextual messages. Action icons use the muted grayscale palette, while health gauges use a blood fill with a darker offset shadow layer. Shared touch targets retain a 48px logical minimum while surrounding spacing and typography use viewport-relative units.

The secondary gameplay navigation is implemented as a centered overlay with a soft dimming layer so the underlying gameplay remains visible. The overlay is a navigation layer rather than a replacement gameplay screen, and its dedicated-system entries reuse the same shared action-button primitives. Options is a dedicated, tabbed information screen reached from that secondary navigation; its current Game Data tab exposes the configured filesystem root without mixing that setting into the gameplay HUD. On desktop, selecting another root persists the setting for the next launch. On Android, the same control opens the existing Storage Access Framework picker and keeps the selected shared folder as the user-facing editable-storage location.

Text remains the primary presentation medium, with optional world/location artwork represented by view data where available.

## v0.51 graphical foundation

Issue #278 establishes the reusable presentation primitives for the redesigned gameplay HUD and secondary navigation. Issue #279 applies those primitives to the gameplay surface with a world-first composition, responsive primary/contextual actions, dynamic time-of-day sky treatment, and the compact player HUD. Issue #280 supplies the secondary-navigation layer, and #227 now adds the dedicated tabbed Options screen on top of it. The pause menu remains #226.

## Design direction

Keep screen responsibilities separate from gameplay mechanics. New UI work should reuse frontend-neutral view models and semantic input events rather than introducing renderer-specific dependencies into gameplay rules. Avoid maintaining parallel terminal and graphical implementations.
