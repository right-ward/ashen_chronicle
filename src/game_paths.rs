use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const GAME_DIRECTORY_NAME: &str = "The Ashen Chronicle";
const DATA_DIRECTORY_NAME: &str = "data";
const MODS_DIRECTORY_NAME: &str = "mods";
const SAVES_DIRECTORY_NAME: &str = "saves";

#[derive(Debug, Clone)]
pub struct GamePaths {
    root: PathBuf,
    data_dir: PathBuf,
    mods_dir: PathBuf,
    saves_dir: PathBuf,
    bundled_data_dir: Option<PathBuf>,
    using_android_fallback: bool,
}

impl GamePaths {
    pub fn initialize() -> io::Result<Self> {
        let bundled_data_dir = discover_bundled_data_dir();
        let roots = platform_roots();

        for (root, using_android_fallback) in roots {
            match prepare_root(&root, bundled_data_dir.as_deref()) {
                Ok(mut paths) => {
                    paths.bundled_data_dir = bundled_data_dir.clone();
                    paths.using_android_fallback = using_android_fallback;
                    std::env::set_current_dir(&paths.root)?;
                    return Ok(paths);
                }
                Err(_) => continue,
            }
        }

        Err(io::Error::new(
            io::ErrorKind::Other,
            "unable to initialize the game root",
        ))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn mods_dir(&self) -> &Path {
        &self.mods_dir
    }

    pub fn saves_dir(&self) -> &Path {
        &self.saves_dir
    }

    pub fn bundled_data_dir(&self) -> Option<&Path> {
        self.bundled_data_dir.as_deref()
    }

    pub fn using_android_fallback(&self) -> bool {
        self.using_android_fallback
    }
}

fn prepare_root(root: &Path, bundled_data_dir: Option<&Path>) -> io::Result<GamePaths> {
    let data_dir = root.join(DATA_DIRECTORY_NAME);
    let mods_dir = data_dir.join(MODS_DIRECTORY_NAME);
    let saves_dir = root.join(SAVES_DIRECTORY_NAME);

    fs::create_dir_all(&mods_dir)?;
    fs::create_dir_all(&saves_dir)?;

    let paths = GamePaths {
        root: root.to_path_buf(),
        data_dir,
        mods_dir,
        saves_dir,
        bundled_data_dir: None,
        using_android_fallback: false,
    };

    sync_bundled_data(&paths, bundled_data_dir)?;
    Ok(paths)
}

fn sync_bundled_data(paths: &GamePaths, bundled_data_dir: Option<&Path>) -> io::Result<()> {
    let Some(bundled_data_dir) = bundled_data_dir else {
        return Ok(());
    };

    let bundled_base = bundled_data_dir.join("base_content.json");
    let game_base = paths.root.join("base_content.json");
    if bundled_base.is_file() {
        fs::copy(&bundled_base, &game_base)?;
        set_read_only(&game_base)?;
    }

    let bundled_mods = bundled_data_dir.join(MODS_DIRECTORY_NAME);
    if bundled_mods.is_dir() {
        sync_directory_without_overwriting(&bundled_mods, &paths.mods_dir)?;
    }

    Ok(())
}

fn sync_directory_without_overwriting(source: &Path, destination: &Path) -> io::Result<()> {
    fs::create_dir_all(destination)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());

        if source_path.is_dir() {
            sync_directory_without_overwriting(&source_path, &destination_path)?;
        } else if source_path.is_file() && !destination_path.exists() {
            fs::copy(source_path, destination_path)?;
        }
    }

    Ok(())
}

fn set_read_only(path: &Path) -> io::Result<()> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions)
}

fn discover_bundled_data_dir() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("data"),
        std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|parent| parent.join("data")))
            .unwrap_or_default(),
        std::env::current_exe()
            .ok()
            .and_then(|path| {
                path.parent()
                    .and_then(Path::parent)
                    .map(|parent| parent.join("data"))
            })
            .unwrap_or_default(),
    ];

    candidates.into_iter().find(|path| path.is_dir())
}

fn platform_roots() -> Vec<(PathBuf, bool)> {
    #[cfg(target_os = "android")]
    {
        let shared = std::env::var_os("EXTERNAL_STORAGE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/storage/emulated/0"));
        let shared_root = shared
            .join("Documents")
            .join(GAME_DIRECTORY_NAME);
        let fallback_root = shared
            .join("Android")
            .join("data")
            .join("com.rightward.ashenchronicle")
            .join("files")
            .join("Documents")
            .join(GAME_DIRECTORY_NAME);

        vec![(shared_root, false), (fallback_root, true)]
    }

    #[cfg(not(target_os = "android"))]
    {
        let root = dirs::document_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(GAME_DIRECTORY_NAME);
        vec![(root, false)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        std::env::temp_dir().join(format!(
            "ashen-chronicle-game-paths-{}",
            std::process::id()
        ))
    }

    #[test]
    fn game_root_contains_expected_directories() {
        assert_eq!(GAME_DIRECTORY_NAME, "The Ashen Chronicle");
        assert_eq!(DATA_DIRECTORY_NAME, "data");
        assert_eq!(MODS_DIRECTORY_NAME, "mods");
        assert_eq!(SAVES_DIRECTORY_NAME, "saves");
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
            fs::read_to_string(destination.join("nested_marker"))
                .expect("existing file should remain"),
            "marker"
        );

        let _ = fs::remove_dir_all(root);
    }
}
