//! Loading moddable PNG assets from the selected game data folder.
//!
//! References such as `icons/ui/back.png` resolve under `data/assets/`.
//! References prefixed with `mod:` resolve under a specific mod's `assets/`
//! directory, for example `mod:ashen_expansion/locations/ruin.png`.
//! Images are read from disk instead of being compiled into the executable.

use std::fs;
use std::path::{Component, Path, PathBuf};

use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::{Assets, Handle, Image};

fn validate_relative_path(path: &Path) -> Result<(), String> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || !path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(format!("asset reference must be a safe relative path: {}", path.display()));
    }
    Ok(())
}

fn resolve_roots(reference: &str) -> Result<(PathBuf, PathBuf), String> {
    let data_dir = std::env::current_dir()
        .map_err(|error| format!("could not resolve game data directory: {error}"))?
        .join("data");

    if let Some(relative) = reference.strip_prefix("mod:") {
        let relative_path = Path::new(relative);
        validate_relative_path(relative_path)?;
        let mut components = relative_path.components();
        let Some(Component::Normal(folder)) = components.next() else {
            return Err("mod asset reference must include a mod folder".to_string());
        };
        let folder = Path::new(folder);
        let remaining = relative_path
            .strip_prefix(folder)
            .map_err(|_| "mod asset reference must include a file path after the mod folder".to_string())?;
        if remaining.as_os_str().is_empty() {
            return Err("mod asset reference must include a file path after the mod folder".to_string());
        }
        validate_relative_path(remaining)?;
        let root = data_dir.join("mods").join(folder).join("assets");
        Ok((root.clone(), root.join(remaining)))
    } else {
        let relative_path = Path::new(reference);
        validate_relative_path(relative_path)?;
        let root = data_dir.join("assets");
        Ok((root.clone(), root.join(relative_path)))
    }
}

/// Load a PNG from the selected data folder, constraining file resolution to the
/// intended asset root so malformed references and symlink escapes are rejected.
pub(crate) fn load_image_from_data(
    images: &mut Assets<Image>,
    reference: &str,
) -> Result<Handle<Image>, String> {
    let (root, path) = resolve_roots(reference)?;
    let canonical_root = fs::canonicalize(&root)
        .map_err(|error| format!("asset root {} is unavailable: {error}", root.display()))?;
    let canonical_path = fs::canonicalize(&path)
        .map_err(|error| format!("asset {} could not be found: {error}", path.display()))?;
    if !canonical_path.starts_with(&canonical_root) || !canonical_path.is_file() {
        return Err(format!("asset path escapes its asset root: {}", path.display()));
    }

    let bytes = fs::read(&canonical_path)
        .map_err(|error| format!("could not read asset {}: {error}", canonical_path.display()))?;
    let mut image = Image::from_buffer(
        &bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::nearest(),
        RenderAssetUsages::default(),
    )
    .map_err(|error| format!("could not decode PNG asset {}: {error}", canonical_path.display()))?;
    image.sampler = ImageSampler::nearest();
    Ok(images.add(image))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_traversal_and_absolute_asset_paths() {
        assert!(validate_relative_path(Path::new("../secret.png")).is_err());
        assert!(validate_relative_path(Path::new("icons/../../secret.png")).is_err());
        assert!(validate_relative_path(Path::new("/tmp/secret.png")).is_err());
        assert!(resolve_roots("mod:../secret/icon.png").is_err());
        assert!(resolve_roots("mod:example/../../secret.png").is_err());
    }

    #[test]
    fn separates_shared_assets_from_mod_scoped_assets() {
        let (global_root, global_file) =
            resolve_roots("icons/ui/back.png").expect("global asset reference should be valid");
        assert!(global_root.ends_with("data/assets"));
        assert!(global_file.ends_with("data/assets/icons/ui/back.png"));

        let (mod_root, mod_file) = resolve_roots("mod:ashen_expansion/locations/ruin.png")
            .expect("mod asset reference should be valid");
        assert!(mod_root.ends_with("data/mods/ashen_expansion/assets"));
        assert!(mod_file.ends_with("data/mods/ashen_expansion/assets/locations/ruin.png"));
    }
}
