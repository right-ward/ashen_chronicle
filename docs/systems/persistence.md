# Persistence System

The persistence system stores the world and character state while keeping inherited worlds compatible across character deaths and later versions.

## Save format

Saves use the existing JSON payload format compressed with gzip and are stored under the game root's `saves/` directory. Character-specific filenames follow the form `ashen_chronicle_save_<character>.json.gz` after filename sanitization.

Legacy uncompressed `ashen_chronicle_save.json` saves remain loadable. Save migration uses defaulted fields and explicit migration handling when new progression or time data is introduced.

## Game data root configuration

The desktop game-data root can be changed from the dedicated Options screen. Desktop display mode is stored separately in the user's OS configuration directory so fullscreen/windowed preset selection survives launches without becoming part of game data or saves. The selected path is stored in the user's OS configuration directory rather than inside the selected game root, so changing roots does not lose the setting itself. The new root is applied on the next launch; the running session continues using its already-initialized root.

On Android, the user-facing game folder is managed through the Storage Access Framework. Rust keeps its normal app-scoped filesystem root and the Android activity mirrors editable `data/mods/` and `saves/` content to the selected shared folder. Android also keeps a private `config.json` in app data and mirrors it to the selected game folder. The shared configuration is imported when that folder is available, while the SAF permission/URI remains private Android app state rather than relying on the shared file as the permission grant.

## World and character boundaries

Character-specific state includes progression, conditions, and the personal quest log. World-persistent state includes faction memories, completed quest deeds, corpses, history, event cooldowns, and world time.

When a character dies, the next character can inherit the world without inheriting the dead character's personal quest log or faction reputation. Corpse contents and other persistent traces remain available to later lives.

## Saving and loading

Saving is tied to safe meditation rather than quitting. The start and load flows explicitly select compatible saves instead of silently loading a save. Character-specific save discovery preserves the selected save path.

Loading validates runtime references and reports broken or inconsistent save data clearly.

## Compatibility

Persistence changes should preserve older saves where practical. New fields should have safe defaults or explicit migration paths, and malformed compressed data should produce a stable invalid-data error rather than crashing through an unrelated failure.

## Design direction

Keep the save payload focused on persistent game state. Runtime-only campaign content should be rehydrated from the current content definitions instead of duplicated inside save files.
