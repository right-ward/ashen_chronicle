use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(not(target_os = "android"))]
use crate::desktop_storage;

pub const GAME_DIRECTORY_NAME: &str = "The Ashen Chronicle";
pub const DATA_DIRECTORY_NAME: &str = "data";
pub const MODS_DIRECTORY_NAME: &str = "mods";
pub const SAVES_DIRECTORY_NAME: &str = "saves";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GamePaths {
    pub root: PathBuf,
    pub data_dir: PathBuf,
    pub mods_dir: PathBuf,
    pub saves_dir: PathBuf,
    pub bundled_data_dir: Option<PathBuf>,
    pub using_android_fallback: bool,
}

impl GamePaths {
    pub fn initialize() -> io::Result<Self> {
        let bundled_data_dir = discover_bundled_data_dir();
        let (preferred_root, fallback_root) = platform_roots();

        #[cfg(not(target_os = "android"))]
        let preferred_root = desktop_storage::resolve_root(&preferred_root)?;

        match prepare_root(&preferred_root, bundled_data_dir.as_deref()) {
            Ok(paths) => {
                std::env::set_current_dir(&paths.root)?;
                Ok(paths)
            }
            Err(preferred_error) => {
                let Some(fallback) = fallback_root else {
                    return Err(preferred_error);
                };
                let mut paths = prepare_root(&fallback, bundled_data_dir.as_deref())?;
                paths.using_android_fallback = cfg!(target_os = "android");
                std::env::set_current_dir(&paths.root)?;
                Ok(paths)
            }
        }
    }

    pub fn base_content_path(&self) -> PathBuf {
        self.data_dir.join("base_content.json")
    }
}

fn prepare_root(root: &Path, bundled_data_dir: Option<&Path>) -> io::Result<GamePaths> {
    ensure_directory(root)?;
    let data_dir = root.join(DATA_DIRECTORY_NAME);
    let mods_dir = data_dir.join(MODS_DIRECTORY_NAME);
    let saves_dir = root.join(SAVES_DIRECTORY_NAME);
    ensure_directory(&data_dir)?;
    ensure_directory(&mods_dir)?;
    ensure_directory(&saves_dir)?;

    let paths = GamePaths {
        root: root.to_path_buf(),
        data_dir,
        mods_dir,
        saves_dir,
        bundled_data_dir: bundled_data_dir.map(Path::to_path_buf),
        using_android_fallback: false,
    };
    paths.sync_bundled_data(bundled_data_dir)?;
    Ok(paths)
}

impl GamePaths {
    fn sync_bundled_data(&self, bundled_data_dir: Option<&Path>) -> io::Result<()> {
        let Some(bundled_data_dir) = bundled_data_dir else {
            return Ok(());
        };

        let bundled_base = bundled_data_dir.join("base_content.json");
        if bundled_base.is_file() {
            let target = self.base_content_path();
            if target.is_file() {
                set_writable(&target)?;
            }
            fs::copy(&bundled_base, &target)?;
            set_read_only(&target)?;
        }

        let bundled_mods = bundled_data_dir.join(MODS_DIRECTORY_NAME);
        if bundled_mods.is_dir() {
            sync_directory_without_overwriting(&bundled_mods, &self.mods_dir)?;
        }
        Ok(())
    }
}

fn platform_roots() -> (PathBuf, Option<PathBuf>) {
    #[cfg(target_os = "android")]
    {
        let external_root = std::env::var_os("EXTERNAL_STORAGE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/storage/emulated/0"));
        let root = external_root
            .join("Android")
            .join("data")
            .join("com.rightward.ashenchronicle")
            .join("files")
            .join("Documents")
            .join(GAME_DIRECTORY_NAME);
        (root, None)
    }

    #[cfg(not(target_os = "android"))]
    {
        let preferred = dirs::document_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(GAME_DIRECTORY_NAME);
        (preferred, None)
    }
}

fn discover_bundled_data_dir() -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(current_dir.join("data"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("data"));
            if let Some(parent) = dir.parent() {
                candidates.push(parent.join("data"));
            }
        }
    }

    candidates.into_iter().find(|path| path.is_dir())
}

fn ensure_directory(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        Ok(())
    } else {
        fs::create_dir_all(path)
    }
}

fn sync_directory_without_overwriting(source: &Path, destination: &Path) -> io::Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            ensure_directory(&destination_path)?;
            sync_directory_without_overwriting(&source_path, &destination_path)?;
        } else if source_path.is_file() && !destination_path.exists() {
            fs::copy(&source_path, &destination_path)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn set_writable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(permissions.mode() | 0o200);
    fs::set_permissions(path, permissions)
}

#[cfg(not(unix))]
fn set_writable(path: &Path) -> io::Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_readonly(false);
    fs::set_permissions(path, permissions)
}

fn set_read_only(path: &Path) -> io::Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be valid")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("ashen_chronicle_game_paths_{}", stamp));
        fs::create_dir_all(&path).expect("temporary directory should be created");
        path
    }

    #[test]
    fn game_paths_use_expected_directory_names() {
        let root = PathBuf::from("/tmp/chronicle");
        let paths = GamePaths {
            root: root.clone(),
            data_dir: root.join(DATA_DIRECTORY_NAME),
            mods_dir: root.join(DATA_DIRECTORY_NAME).join(MODS_DIRECTORY_NAME),
            saves_dir: root.join(SAVES_DIRECTORY_NAME),
            bundled_data_dir: None,
            using_android_fallback: false,
        };

        assert_eq!(
            paths.base_content_path(),
            root.join("data/base_content.json")
        );
        assert_eq!(paths.mods_dir, root.join("data/mods"));
        assert_eq!(paths.saves_dir, root.join("saves"));
    }

    #[test]
    fn sync_does_not_overwrite_existing_user_files() {
        let root = temp_dir();
        let source = root.join("bundled");
        let destination = root.join("user");
        fs::create_dir_all(source.join("nested")).expect("source should exist");
        fs::create_dir_all(&destination).expect("destination should exist");
        fs::write(source.join("nested/content.json"), "bundled").expect("source file should exist");
        fs::write(destination.join("nested_marker"), "marker").expect("marker should exist");

        sync_directory_without_overwriting(&source, &destination).expect("sync should succeed");
        assert_eq!(
            fs::read_to_string(destination.join("nested/content.json"))
                .expect("copied file should exist"),
            "bundled"
        );
        assert_eq!(
            fs::read_to_string(destination.join("nested_marker")).expect("marker should exist"),
            "marker"
        );
        fs::remove_dir_all(root).expect("temporary directory should be removed");
    }
}
