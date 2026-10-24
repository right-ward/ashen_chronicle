# UI icon assets

The PNGs in this directory are the graphical frontend's shared icons. They are loaded at runtime from the selected game's `data/assets/icons/ui/` directory. Copying a replacement PNG over a named file changes that icon without recompiling the game.

Use transparent PNG artwork with a 32×32 source canvas and nearest-neighbor sampling for a consistent pixel-art style. If a mod needs a unique icon, it can place the PNG under its own `assets/` directory and reference it with `mod:<mod-folder>/<relative-path>.png`.

Missing icons are logged and omitted; they are not replaced with Unicode glyphs or placeholder text.
