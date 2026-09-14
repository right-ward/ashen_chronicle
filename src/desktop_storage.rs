use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};

pub fn resolve_root(preferred_root: &Path) -> io::Result<PathBuf> {
    if preferred_root.exists() {
        return match MessageDialog::new()
            .set_level(MessageLevel::Info)
            .set_title("The Ashen Chronicle storage")
            .set_description(format!(
                "An existing game root was found at:\n\n{}\n\nYes: use it\nNo: recreate it\nCancel: choose another folder",
                preferred_root.display()
            ))
            .set_buttons(MessageButtons::YesNoCancel)
            .show()
        {
            MessageDialogResult::Yes => Ok(preferred_root.to_path_buf()),
            MessageDialogResult::No => {
                fs::remove_dir_all(preferred_root)?;
                Ok(preferred_root.to_path_buf())
            }
            _ => pick_alternate_root(preferred_root),
        };
    }

    match MessageDialog::new()
        .set_level(MessageLevel::Info)
        .set_title("The Ashen Chronicle storage")
        .set_description(format!(
            "The default game root does not exist yet:\n\n{}\n\nYes: create it\nNo: choose another folder",
            preferred_root.display()
        ))
        .set_buttons(MessageButtons::YesNo)
        .show()
    {
        MessageDialogResult::Yes => Ok(preferred_root.to_path_buf()),
        _ => pick_alternate_root(preferred_root),
    }
}

fn pick_alternate_root(preferred_root: &Path) -> io::Result<PathBuf> {
    let start = preferred_root
        .parent()
        .filter(|path| path.is_dir())
        .map(Path::to_path_buf)
        .or_else(dirs::document_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    FileDialog::new()
        .set_title("Choose The Ashen Chronicle game root")
        .set_directory(start)
        .pick_folder()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Interrupted, "game root selection was cancelled"))
}
