# The Ashen Chronicle

## Roadmap

## Pre-v0.8.0

- the Rust project exists
- the game name is finalized
- the design plan and roadmap are written
- the codebase has a clean folder structure

- save/load is implemented through a versioned JSON save file
- world state model
- entity IDs
- basic character creation
- text UI
- a simple travel/history/death loop is playable
- the recovery action is meditate, and it heals plus saves when the player is safe

- the player can move through a generated world
- the player can die
- the death screen offers world reset or world inheritance
- safe meditation and saving are tied to world danger
- a basic threat interaction exists
- threat resolution now has a real combat loop
- defeating a threat can clear danger and leave a trophy item behind
- item pickups now notify the player directly
- major action results now pause on screen so the text does not get skipped

- corpses are created when a character dies
- items drop into the corpse inventory instead of disappearing
- corpse remains persist in the world after inheritance
- locations show visible remains
- the player can search remains and recover items
- history records death, looting, and aftermath events
- danger can be cleared by defeating a threat, changing the world state
- faction reputation is tracked and changes when the player completes the shrine quest
- NPCs remember important events and react differently on later visits
- hidden quests stay out of the quest log until they are actually offered
- the first simple quest is active and rewards the player

- the region expanded from 3 to 8 connected locations
- three distinct dangerous locations now have different enemy profiles
- additional enemy types and location-specific trophies were added
- the item pool now contains multiple trophies and faction rewards
- the faction roster expanded to three factions
- the NPC roster expanded to five named NPCs with persistent memories
- the quest system now supports multiple active quest chains using the same offer/turn-in rules
- location arrival scenes provide distinct atmospheric events and descriptions
- campaign bootstrap now backfills new content into older saved worlds instead of requiring a reset
- campaign content definitions are centralized in the bootstrap layer to prepare for the later data-loading phase

## v0.8.1 Quest-system maintenance

- new characters no longer inherit the previous character's personal quest log
- completed quests are stored as persistent world deeds so later characters do not receive duplicate quests
- existing completed quest records are migrated into persistent world deeds during bootstrap/inheritance
- the main location description no longer prints a duplicate quest summary; Quest Log remains the dedicated quest view
- required quest items are consumed when a quest is successfully turned in
- fixed campaign quest bootstrapping so the shrine quest is not recreated on every load

## v0.8.2 Quest interaction and reputation maintenance

- quest offering and turn-in now use an explicit Talk action with NPC selection
- same-location quest completion now works immediately after the threat is defeated
- NPCs no longer automatically offer or complete quests merely because the player entered a location
- new characters start with zero faction reputation while persistent faction memories remain
- quest completion grants +5 faction reputation and the associated faction reward contributes another +5 while held
- inherited reward items restore their +5 faction contribution when the new character recovers them from a previous life

## v0.9.0 Data-loading foundation

- added a runtime-loaded campaign content file under `data/base_content.json`
- seeded the world from loaded content instead of hardcoded location bootstrap
- added validation for duplicate content IDs and broken content references
- moved NPC, faction, quest, atmosphere, and encounter definitions into the content pack
- added stable content IDs to the base campaign data
- kept existing save data compatible through defaulted quest fields

## v0.10.0 Mod loading foundation

- added mod discovery under `data/mods`
- loaded mod content after the base content pack
- merged content by stable IDs and location keys so mods can replace or extend data safely
- kept broken mod files from crashing the game by reporting load warnings
- moved quest identity handling onto stable content IDs in the runtime paths

## v0.11.0 Optional visuals foundation

- added optional ASCII portraits for NPCs
- added optional location scene art
- added optional item illustrations for rewards and trophies
- kept visuals fully optional so the game still works as text-only when assets are missing

## v0.12.0 Polish and balance pass

- simplified the main status screen to remove repeated world-mode and history-count clutter
- removed repeated location and inventory ASCII art from the always-visible screens while retaining contextual visuals during interaction and item acquisition
- tightened combat output so attacks and damage resolve in one readable sequence instead of pausing after every hit
- added explicit combat victory and quest reward summaries
- added clearer save-load validation warnings for broken runtime references and inconsistent completed quest deeds
- adjusted dangerous encounters to reduce early-game damage spikes while preserving different threat durability
- reduced repetitive travel/location presentation by keeping the recurring status display focused on actionable information

## v0.12.1 Quit-screen polish

- replaced the plain quit confirmation with a randomized atmospheric exit screen
- added four original dark farewell variants with contextual ASCII art
- kept quitting non-destructive: `N`, `No`, or Enter returns to the game without changing state

## v0.13.0 Gameplay polish and balance completion

- added an atmospheric end-of-life screen with a concise summary of death location, remembered deeds, faction standing, and items left on the corpse
- made previous-life traces discoverable through corpse recovery and NPC memories without restoring the dead character's quest log
- added a small set of randomized, non-quest travel events to make repeated journeys less predictable without changing important quest outcomes
- added explicit corpse/legacy feedback so inherited equipment is distinguished from information that must be rediscovered
- retained the existing faction reputation split and prevented new characters from inheriting old reputation

## v0.14.1 Character progression, NPC availability, and Time foundation

- added Might, Insight, and Endurance attributes with level-based progression
- added experience gains from combat, quests, and discovery, with player-chosen level improvements
- added a character sheet for progression and condition visibility
- added Wounded, Exhausted, and Well-rested conditions with gameplay effects
- added a persistent world time cycle with named day/night portions, day count, and an east-to-west ASCII sun/moon track
- replaced the time track with a compact two-line celestial cycle and a clear east-to-west indicator line
- kept the sun and moon readable with plain Unicode symbols while hiding the internal time variable from player-facing text
- improved unavailable NPC feedback so it explains whether it is too early or too late and hints when to check again
- made travel, combat, searching, journaling, talking, and meditation advance hidden time portions
- made meditation duration player-selected and tied healing directly to time spent
- added time-sensitive NPC availability and travel atmosphere changes
- kept progression and conditions character-specific while world time persists through inheritance
- added v1 save migration for the new progression/time fields
- added small time-sensitive travel variation to exercise the new systems

## v0.14.2 Location scene art presentation

- rendered optional location scene art during arrival scenes so the player sees the place before the descriptive text
- kept scene art fully optional and text-only fallback intact when no art is defined
- reused the existing atmosphere and NPC scene flow so visual content layers cleanly onto the text systems

## v0.16.0 Keyboard-driven interaction pass

- replaced the blocking number-entry menus with keyboard-driven selection using arrows, Enter, Esc, and number shortcuts
- added raw-mode text input so prompts work directly inside the TUI without falling back to scrollback interaction
- changed pause handling to a single-key confirmation instead of Enter-only input
- updated the on-screen control hint to match the new keyboard flow

## v0.17.0 Responsive ratatui renderer

- migrated the terminal renderer to ratatui
- added width/height-aware compact rendering for narrow vertical screens
- kept the existing gameplay flows working inside the new screen shell
- bumped the project version to v0.17.0

## v0.17.1 Mobile portrait UI cleanup

- expanded compact-mode detection so tall, narrow terminals no longer get forced into the desktop split layout
- made prompt overlays use more of the available screen space on compact terminals
- added scrolling window behavior for menu overlays so longer option lists stay readable on smaller screens

## v0.17.2 Monochrome prompt cleanup

- switched the UI styling to monochrome gray/white borders and highlights
- moved the pause prompt out of the center overlay so result text stays visible
- reduced the choice popup footprint so it does not bury the rest of the screen as aggressively

## v0.17.3 Docked prompt layout cleanup

- moved prompt and confirmation dialogs into a reserved bottom panel instead of drawing them over the rest of the screen
- kept the main dashboard visible while choices, pause prompts, and quit confirmations are active
- tightened the prompt layout so messages and results stay readable behind the prompt flow

## v0.17.4 Turn-based result cleanup

- removed the duplicate top-of-screen game-state summary so the header no longer repeats the Status panel
- changed the Messages panel into a short-lived Result panel that is cleared when the player starts a new choice
- kept action outcomes visible only for the current turn instead of letting old text pile up indefinitely
- tightened the header layout to give landscape screens more usable space

## v0.17.5 Main-screen cleanup

- removed the empty top header box instead of leaving it as a hollow frame
- removed debug-style location exits and people listings from the main dashboard
- removed the character name and faction reputation from the main dashboard
- moved faction reputation into the character sheet
- moved location arrival art and atmosphere into the Location panel instead of the Result panel
- fixed the quit confirmation prompt so it no longer prints a stray `>` on separate lines

## v0.17.6 UI polish and health gauges

- replaced plain-text player HP rendering with a ratatui `LineGauge`
- added an enemy health `LineGauge` during combat
- used blood/dark-red player health and dark-purple enemy health fills while retaining the monochrome UI elsewhere
- kept combat health in the dashboard state so it updates while combat choices are rendered
- normalized panel rendering and collapsed border overlap across compact and wide layouts
- removed redundant combat HP text output and simplified the result display
- bumped the project version to v0.17.6

## v0.18.0 Event system foundation

- added a reusable event runtime in `src/events.rs`
- added stable event IDs, triggers, weights, chance gates, conditions, effects, and cooldowns to campaign content
- migrated the four existing travel events into `data/base_content.json`
- persisted event cooldowns as part of the world state with serde defaults for older saves
- added content validation for event IDs, triggers, weights, effects, chance ranges, and condition references
- added unit tests covering event conditions and cooldown eligibility
- removed the old hardcoded random travel-event branch from `game.rs`

## v0.18.1 Event validation and persistence test coverage

- reject invalid individual event definitions instead of loading them into the runtime
- emit warnings identifying each rejected event and the exact validation reasons while continuing to load valid content
- reject duplicate event IDs without overwriting previously accepted events
- added save/load coverage for persistent event cooldowns
- added world-inheritance coverage confirming event cooldown state persists with the inherited world
- added content filtering tests for invalid events and duplicate event IDs

## v0.19.0 Event & world memory integration

- cached campaign content in runtime state so event processing no longer reloads content for every trigger
- added structured history entries for narrative and event records, including event ID, location, and outcome data
- recorded executed events automatically in persistent world history
- added history-aware event conditions through prior event IDs
- validated prior-event references and rejected invalid event definitions without preventing valid content from loading
- rehydrated runtime campaign content after save loading without storing campaign content in save files
- added tests for structured event history, history-based conditions, runtime content rehydration, and existing persistence behavior

## v0.19.1 Save compression and character-specific filenames

- save files are gzip-compressed while retaining the existing JSON payload format
- save filenames now include a sanitized character name (`ashen_chronicle_save_<character>.json.gz`)
- legacy uncompressed `ashen_chronicle_save.json` files remain loadable
- startup discovers character-specific saves and preserves the selected save path
- added tests for compressed round-tripping, legacy loading, invalid gzip data, filename sanitization, and character-specific paths

## v0.20.0 State-aware event conditions

- added faction reputation conditions with minimum and maximum reputation thresholds
- added required inventory-item conditions
- added required active-condition checks
- validated referenced factions and invalid reputation ranges during content loading
- kept event conditions backward-compatible through serde defaults
- fixed mod event validation so a mod event may safely reference an existing base event in `prior_event_id` conditions
- added focused runtime and content-validation tests for reputation, inventory, condition, and cross-content event requirements
- added the `event.market-recognition` campaign event to demonstrate reputation-gated narrative content

## v0.20.1 Test-failure repair and runtime flow restoration

- restored the startup/load control flow that was accidentally truncated in v0.20.0
- restored campaign bootstrap for factions, NPCs, and quests using stable content references
- restored main-menu dispatch, loaded-state validation, NPC/faction/quest lookup helpers, and character creation/inheritance prompts
- fixed save-path handling when a character dies and a new character inherits or starts a new world
- removed the unused diagnostic macro and resolved the warnings reported by the v0.20.0 test run

## v0.20.2 Persistence test fixes

- normalized gzip decompression failures to `io::ErrorKind::InvalidData` so malformed compressed saves have a stable public error category
- treated filename components containing no alphanumeric characters as `unnamed` after sanitization
- removed two unused runtime helpers reported by the test build as dead code

## v0.22.0 Start and quit screen overhaul

- added a dedicated start screen instead of immediately entering new-game creation or save loading
- made `New Game`, `Load Game`, and `Quit` explicit start-screen choices
- only show `Load Game` when a compatible save or legacy save exists
- added a dedicated save-selection screen instead of automatic loading or Y/N load confirmation
- kept character/world creation behind the start-screen flow rather than making it the initial screen
- added dedicated full-screen menu rendering so start, load, creation, and quit flows do not render the gameplay dashboard underneath them
- changed quit handling to use the existing randomized farewell sentences as the actual choices instead of Y/N confirmation
- removed save-on-quit behavior; saving remains tied to safe meditation
- made death-screen quitting non-destructive as well

## v0.22.1 Dark start-screen art variants

- added four randomized dark start-screen ASCII variants
- paired each start-screen artwork variant with an approved atmospheric sentence
- kept start-screen art limited to the start flow; quit and death artwork remain unchanged for their own patches

## v0.22.2 Dark quit-screen art variants

- expanded the quit-screen artwork pool with five additional dark ASCII variants
- added matching farewell choices for gate, extinguished shrine, empty road, graveyard, and final-look imagery
- kept quit behavior non-destructive; selecting the leave choice exits without saving
- left the death-screen artwork untouched for the dedicated death-screen patch

## v0.22.3 Dark death-screen art variants

- added five new dark death-screen ASCII variants
- added distinct death-themed sentences for the new variants
- retained the existing three death-screen variants for a total of eight randomized variants
- kept the existing death summary, inheritance flow, and non-destructive quit behavior unchanged

## v0.23.0 Game Screen Architecture

- extracted start, load, character creation, quit, and death screen logic from `game.rs` into `game/screens.rs`
- reduced `game.rs` responsibilities to game orchestration, state management, and gameplay logic
- kept all gameplay behavior and screen flows unchanged
- no gameplay mechanics or data structures were changed

### Architectural Direction

This begins the gradual decomposition of the large `game.rs` module by responsibility.

Future refactoring should continue only where there is a clear separation of responsibility rather than splitting files purely to reduce line counts.

```
src/
├── main.rs
├── game.rs              # game orchestration / main loop
├── game/
│   ├── combat.rs
│   ├── actions.rs
│   ├── screens.rs
│   ├── character.rs
│   └── world.rs
├── content.rs
├── content/
│   ├── loader.rs
│   ├── definitions.rs
│   └── seeding.rs
├── events.rs
├── model.rs
├── persistence.rs
└── ui.rs
```

## v0.24.0 Gameplay Action Architecture

- extracted main-menu action definitions and dispatch targets from `game.rs` into `game/actions.rs`
- moved travel, meditation, NPC interaction, quest handling, inventory, journal, corpse recovery, progression, time/condition helpers, and death handling into the action module
- extracted the combat encounter loop and combat-specific helpers into `game/combat.rs`
- reduced `game.rs` to orchestration, campaign bootstrap/validation, dashboard rendering, and location-scene presentation
- kept save compatibility and existing gameplay behavior unchanged
- kept the new modules dependent on the existing model, persistence, event, and UI systems instead of introducing parallel state systems

## v0.25.0 Content Architecture

- split the monolithic `content.rs` into `content/definitions.rs`, `content/loader.rs`, and `content/seeding.rs`
- kept `content.rs` as a compatibility-facing module facade and preserved the existing content-loading entry point
- moved campaign data definitions and content validation into the definitions module
- moved world seeding and content-to-world translation into the seeding module
- moved base/mod loading, merging, event filtering, path discovery, and content-loading tests into the loader module
- corrected the embedded fallback content path for the new loader module location
- preserved campaign content formats, mod behavior, event validation, and save compatibility
- bumped the project version to v0.25.0

## v0.25.1 Game Runtime Architecture Cleanup

- moved campaign bootstrap and loaded-state validation from `game.rs` into `game/world.rs`
- moved dashboard rendering and location-scene presentation from `game.rs` into `game/presentation.rs`
- kept `game.rs` focused on application flow, menu dispatch, and orchestration
- reused the cached campaign content during location-scene presentation instead of reloading the content pack on every newly visited location
- preserved gameplay behavior, save compatibility, and the existing screen flow
- bumped the project version to v0.25.1

## v0.26.0 Game Runtime Loop Architecture

- moved the main gameplay loop from `game.rs` into `game/runtime.rs`
- kept `game.rs` as the top-level game entry point and compatibility facade
- kept runtime orchestration dependent on the existing action, combat, screen, world, and presentation modules
- preserved gameplay behavior, save compatibility, and menu dispatch behavior
- bumped the project version to v0.26.0

## v0.27.0 Game Facade Cleanup

- removed the redundant loaded-state validation wrapper from `game.rs` and re-exported the world validation function directly
- kept `game.rs` as a minimal top-level game entry facade
- preserved gameplay behavior, save compatibility, and screen flow
- bumped the project version to v0.27.0

## v0.27.1 World Bootstrap Decoupling

- removed the `game/world.rs` dependency on `game/actions.rs`
- kept faction lookup inside the world/bootstrap responsibility
- preserved campaign bootstrap, save compatibility, and gameplay behavior
- bumped the project version to v0.27.1

## v0.28.0 Gameplay Action Dispatcher Architecture

- moved `GameAction` dispatch out of `game/runtime.rs` into `game/dispatcher.rs`
- kept the runtime loop focused on lifecycle handling, presentation, menu selection, and turn flow
- preserved existing action, combat, screen, world, save, and quit behavior
- bumped the project version to v0.28.0

## v0.29.0: Gameplay Action Architecture Cleanup I
- Time-display formatting now has its own gameplay time module instead of living inside `game/actions.rs`

## v0.30.0: Gameplay Interaction Architecture Cleanup II
- NPC selection, dialogue, quest offering/turn-in, faction memory, faction reputation, and NPC availability now live in `game/interactions.rs` instead of `game/actions.rs`

## v0.31.0: Gameplay Character Architecture Cleanup III
- Character progression and character-sheet presentation now live in `game/character.rs` instead of `game/actions.rs`.
- `game/actions.rs` retains thin compatibility entry points for progression while the implementation lives in the character module.

## v0.32.0: Gameplay Legacy Architecture Cleanup IV
- Character progression and character-sheet presentation live in `game/character.rs`.
- NPC dialogue, quest interaction, faction memory/reputation, and NPC availability live in `game/interactions.rs`.
- Death, corpse creation, corpse recovery, previous-life item recovery, and legacy item-gain presentation live in `game/legacy.rs`.
- `game/actions.rs` now focuses on ordinary gameplay actions, shared time/condition helpers, menu definitions, inventory/quest views, and journaling.

## v0.33.0: Gameplay Action Architecture Cleanup V
- `game/menu.rs` owns `GameAction`, menu entries, and main-menu construction.
- `game/actions.rs` now focuses on ordinary gameplay actions and shared time/condition helpers, plus inventory, quest, journaling, and character-sheet actions.
- Runtime menu construction and dispatch now depend on the dedicated menu layer rather than `game/actions.rs`.

## v0.34.0: Gameplay Action Architecture Cleanup VI
- `game/actions.rs` now focuses on player-facing gameplay actions rather than temporal state mutation.
- Temporal progression and condition lifecycle helpers now live in `game/state_effects.rs`.
- Travel, meditation, and journaling use the shared state-effects layer.

## v0.35.0: Gameplay Action Architecture Cleanup VII
- `game/actions.rs` now focuses on travel and meditation gameplay actions.
- Inventory display, quest-log display, and journal writing now live in `game/records.rs` as a cohesive player-record concern.
- Character-sheet dispatch now goes directly through `game/character.rs` instead of being wrapped by the action layer.

## v0.36.0: Echoes of the Ashen Road expansion
- Added the Resonant Forge and Hollow Caravan as the expansion's opening locations.
- Expanded the road through Broken Stage, Library of Loops, Silence Spire, Endless Corridor, and the Pit.
- Added expansion encounters, NPCs, faction-linked quests, atmospheres, item art, and location-specific travel events.
- Adapted the old music/metal concepts around an in-world traveling musical tradition, haunted performances, resonance, memory, repetition, and silence rather than modern-world imagery.
- Delivered the expansion through the existing stable-ID mod/content system without changing the gameplay engine or save format.
- Kept the deeper locations in a companion content pack so they can extend and override the opening expansion content through the same loader rules.

## v0.37.0: Developer console foundation
- Added an overlay developer console with scrollable output, command history, and a `/` shortcut from the normal gameplay menu.
- Added command access to world navigation, content/mod inspection, quests, factions, NPCs, inventory, character stats, conditions, time, history, save, and content reload.
- Added Tab autocomplete with arrow-key navigation, Enter selection, Esc cancellation, and stable runtime entity IDs with names/titles shown as hints.
### v0.37.1: Developer console stability patch
- Added an opaque, clean terminal handoff when leaving the developer console so the main gameplay layout is redrawn without stale console buffer contents.
### v0.37.2: Gameplay screen rendering fix
- Reworked the main gameplay menu to render through the shared UI terminal instead of creating a second terminal that overlaid the dashboard.
- Prevented menu choices, results, and journal updates from being visually mixed with the previous gameplay frame.

## v0.38.0: Content loading and world seeding hardening
- Unified base-content and mod discovery around a single resolved `data/` root so the loader cannot silently combine files from different roots.
- Added explicit data-root candidate diagnostics and warnings for missing, unreadable, or malformed external content.
- Strengthened campaign seeding so existing worlds reconcile newly loaded locations, metadata, exits, and persistent campaign entities instead of relying on first-creation state only.
- Exposed the content loading report through the content module for developer diagnostics and future tooling.
- CI reliability: automatic rustfmt commits are retained, while formatting no longer gates test execution; CI also runs when workflow, Cargo, or data files change.

## v0.39.0: UI patching and `logkeys` dev command
- Developer-console output and completion menus support bounded scrolling and keep overflowing content navigable, including wrapped output lines.
- Developer-console implementation is split into focused entrypoint, UI/state, and command modules for easier maintenance.
- Added the `logkeys [true|false]` developer-console command. When enabled, it buffers terminal key events outside the developer console with timestamps, event kinds, key codes, and modifiers; console input itself is excluded.
- Fixed in-game ASCII art flattening by preserving leading whitespace in menu, location, and journal renderers. (#57)
- Fixed transparent/stacked game screens by clearing the full terminal frame before rendering standalone menu and developer-console screens. (#60)
### v0.39.1: patch console and actions layout
- Restored the in-game action menu to the final dashboard panel instead of drawing it as a detached overlay.
- Restored clean developer-console screen lifecycle: the console opens on a cleared screen and returns to the preserved game alternate screen without stacking frames.

## v0.40.0: quest depth and world consequences

- Expanded quests from single-item turn-ins into explicit persisted objective state.
- Existing campaign quests now track visit, defeat, and item-acquisition objectives as a single quest chain.
- Objective progress updates through travel and combat and is visible in the quest log and NPC turn-in flow.
- Quest completion now records a structured quest history event and updates the existing faction and NPC memory systems, allowing later event conditions to react to completed deeds.
- Existing quest/save data is migrated through serde defaults and deterministic quest normalization without changing the save-file version.
- Loaded quest objectives are validated for empty targets, invalid requirements, and impossible progress before runtime use.

## v0.41.0: dedicated quest representation
- Replaced the log-style quest review with a dedicated player-facing fullscreen quest screen.
- Quest index entries clearly distinguish `ACTIVE`, `READY`, and `COMPLETED` states.
- Quest details show the description, objective progress, and reward without mixing into the gameplay journal.
- Returning from the quest screen restores the normal gameplay dashboard without changing quest or gameplay state.
- Kept existing hidden-quest filtering, objective tracking, completion behavior, and save compatibility unchanged.
### v0.41.1: quest navigation bugfix
- Fixed quest details so their Back action returns to the Quests screen instead of exiting directly to gameplay.
- Preserved the existing Quests screen Back action that returns to the normal gameplay dashboard.

## v0.42.0: world navigation presentation
- Replaced the direct Travel menu flow with a dedicated fullscreen World Navigation screen.
- Shows the player's current location, region, description, danger state, and available routes in one focused view.
- Reuses existing location scene art on the navigation screen when it is available.
- Route choices identify dangerous destinations and include a short destination description.
- Keeps the existing travel effects, including time advancement, conditions, threats, quest synchronization, history, and travel events.
- Preserved the existing Back behavior so leaving navigation returns to the normal gameplay dashboard without changing state.

## v0.43.0: inventory and character presentation
- Replaced the plain inventory dump with a dedicated Inventory screen that lets the player select held items and view each item's description and available art.
- Replaced the plain character sheet with a dedicated Character screen containing General, Reputation, and Journal tabs.
- General shows core character stats, health, experience, effective attributes, and active conditions.
- Reputation shows faction standing and remembered faction dealings; Journal shows recorded character notes.
- Reworked meditation to choose the next named time-of-day stopping point instead of entering a numeric duration, while preserving healing, condition updates, history, and automatic saving.
- Preserved the existing screen flow so backing out of the dedicated screens returns to normal gameplay.

