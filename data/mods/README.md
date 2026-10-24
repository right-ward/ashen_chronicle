# Mod folder

Put each mod in its own subfolder with a `manifest.json` file. The manifest points to a content file, usually `content.json`, in the same folder.

A manifest looks like this:

```json
{
  "id": "example.mod",
  "name": "Example Mod",
  "version": "0.1.0",
  "priority": 0,
  "enabled": true,
  "content_file": "content.json"
}
```

Mod content uses the same structure as `data/base_content.json`. Stable IDs are used for replacement, and location names are used for encounter and atmosphere replacement.

## Mod-owned art

A mod may include an `assets/` directory beside its manifest and content file:

```
data/mods/example_mod/
├── manifest.json
├── content.json
└── assets/
    ├── locations/
    │   └── ruined_keep.png
    └── icons/
        └── cult_mark.png
```

Use a `mod:` asset reference in content when the image belongs to a particular mod. For example:

```json
{
  "scene_image": "mod:example_mod/locations/ruined_keep.png"
}
```

The reference resolves to `data/mods/example_mod/assets/locations/ruined_keep.png`. It uses the actual mod folder name, not the manifest ID. References without `mod:` resolve under the selected game's `data/assets/` directory, for example `icons/ui/back.png`.

Only relative PNG paths are supported. Absolute paths and `..` traversal are rejected. Art is read from the selected game data folder at runtime, not compiled into the executable. On Android, bundled defaults are copied into the game folder and the chosen shared folder is synchronized with the app's local data folder.
