# Spatial exploration art

These are first-pass original pixel-art assets for the spatial exploration prototype. PNG files are read from the selected game's `data/assets/spatial/` directory at runtime; they are not compiled into the executable.

- `terrain_floor.png`: worn wooden floor.
- `terrain_ash.png`: ash-covered ground.
- `terrain_rubble.png`: broken stone ground.
- `terrain_wall_top.png`: weathered masonry tops.
- `wall_face_left.png` and `wall_face_right.png`: textured isometric wall faces.
- `player_ash_warden.png`: placeholder player character sprite.
- `passage.png`: physical edge-passage marker.
- `prop_pillar.png`: small stone pillar; 32×40 pixels, one-cell collision footprint.
- `prop_table.png`: weathered wooden table; 72×48 pixels, two-cell collision footprint.

The current art uses nearest-neighbor sampling. Its dimensions are matched to the prototype's 48×24 floor diamonds, 16×24 character sprite, 24×36 wall faces, and the footprint/depth conventions for the pillar and table. These are a starting point, not final art direction.

Copyright (c) 2026 right-ward. Licensed under CC BY-NC-ND 4.0, as documented in `LICENSE-CONTENT`.
