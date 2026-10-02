//! Desktop-only display mode configuration and persistence.

use std::fs;
use std::io::{self, Cursor};
use std::path::PathBuf;

use bevy::prelude::*;
use bevy::window::{MonitorSelection, VideoModeSelection, WindowMode};

const CONFIG_DIRECTORY_NAME: &str = "The Ashen Chronicle";
const CONFIG_FILE_NAME: &str = "display_mode";
const APP_ICON_BYTES: &[u8] = include_bytes!("../data/assets/icons/app/icon.png");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DisplayMode {
    Fullscreen,
    Windowed1920x1080,
    Windowed1280x720,
}

impl Default for DisplayMode {
    fn default() -> Self {
        Self::Windowed1280x720
    }
}

impl DisplayMode {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Fullscreen => "Fullscreen",
            Self::Windowed1920x1080 => "Windowed — 1920 × 1080",
            Self::Windowed1280x720 => "Windowed — 1280 × 720",
        }
    }

    fn storage_value(self) -> &'static str {
        match self {
            Self::Fullscreen => "fullscreen",
            Self::Windowed1920x1080 => "windowed-1920x1080",
            Self::Windowed1280x720 => "windowed-1280x720",
        }
    }

    fn from_storage_value(value: &str) -> Self {
        match value.trim() {
            "fullscreen" => Self::Fullscreen,
            "windowed-1920x1080" => Self::Windowed1920x1080,
            "windowed-1280x720" => Self::Windowed1280x720,
            _ => Self::default(),
        }
    }
}

pub(crate) fn load() -> DisplayMode {
    let Some(path) = config_path() else {
        return DisplayMode::default();
    };

    fs::read_to_string(path)
        .ok()
        .map(|value| DisplayMode::from_storage_value(&value))
        .unwrap_or_default()
}

pub(crate) fn save(mode: DisplayMode) -> io::Result<()> {
    let path = config_path().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "no desktop configuration directory is available",
        )
    })?;
    let parent = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "desktop display configuration has no parent directory",
        )
    })?;

    fs::create_dir_all(parent)?;
    fs::write(path, format!("{}\n", mode.storage_value()))
}

pub(crate) fn window_resolution(mode: DisplayMode) -> (u32, u32) {
    match mode {
        DisplayMode::Fullscreen => (1280, 720),
        DisplayMode::Windowed1920x1080 => (1920, 1080),
        DisplayMode::Windowed1280x720 => (1280, 720),
    }
}

pub(crate) fn window_mode(mode: DisplayMode) -> WindowMode {
    match mode {
        DisplayMode::Fullscreen => {
            WindowMode::Fullscreen(MonitorSelection::Primary, VideoModeSelection::Current)
        }
        DisplayMode::Windowed1920x1080 | DisplayMode::Windowed1280x720 => WindowMode::Windowed,
    }
}

pub(crate) fn apply(window: &mut Window, mode: DisplayMode) {
    let (width, height) = window_resolution(mode);
    window.resolution.set(width as f32, height as f32);
    window.resizable = true;
    window.mode = window_mode(mode);
}

pub(crate) fn apply_window_icon(
    mut applied: Local<bool>,
    primary_window: Single<Entity, With<bevy::window::PrimaryWindow>>,
) {
    if *applied {
        return;
    }

    let Some(icon) = load_window_icon() else {
        return;
    };

    bevy::winit::WINIT_WINDOWS.with(|windows| {
        let windows = windows.borrow();
        let Some(window) = windows.get_window(*primary_window) else {
            return;
        };

        window.set_window_icon(Some(icon));
        *applied = true;
    });
}

fn load_window_icon() -> Option<winit::window::Icon> {
    let decoder = png::Decoder::new(Cursor::new(APP_ICON_BYTES));
    let mut reader = decoder.read_info().ok()?;
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buffer).ok()?;
    let data = &buffer[..info.buffer_size()];

    let rgba = match info.color_type {
        png::ColorType::Rgba => data.to_vec(),
        png::ColorType::Rgb => data
            .chunks_exact(3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 255])
            .collect(),
        png::ColorType::Grayscale => data
            .iter()
            .flat_map(|&gray| [gray, gray, gray, 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => data
            .chunks_exact(2)
            .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
            .collect(),
        png::ColorType::Indexed => return None,
    };

    winit::window::Icon::from_rgba(rgba, info.width, info.height).ok()
}

fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|directory| directory.join(CONFIG_DIRECTORY_NAME).join(CONFIG_FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_modes_have_expected_resolutions() {
        assert_eq!(window_resolution(DisplayMode::Fullscreen), (1280, 720));
        assert_eq!(
            window_resolution(DisplayMode::Windowed1920x1080),
            (1920, 1080)
        );
        assert_eq!(
            window_resolution(DisplayMode::Windowed1280x720),
            (1280, 720)
        );
    }

    #[test]
    fn display_modes_have_stable_labels() {
        assert_eq!(DisplayMode::Fullscreen.label(), "Fullscreen");
        assert_eq!(
            DisplayMode::Windowed1920x1080.label(),
            "Windowed — 1920 × 1080"
        );
        assert_eq!(
            DisplayMode::Windowed1280x720.label(),
            "Windowed — 1280 × 720"
        );
    }

    #[test]
    fn storage_values_round_trip() {
        for mode in [
            DisplayMode::Fullscreen,
            DisplayMode::Windowed1920x1080,
            DisplayMode::Windowed1280x720,
        ] {
            assert_eq!(DisplayMode::from_storage_value(mode.storage_value()), mode);
        }
    }

    #[test]
    fn invalid_storage_value_falls_back_to_windowed() {
        assert_eq!(
            DisplayMode::from_storage_value("invalid"),
            DisplayMode::Windowed1280x720
        );
    }

    #[test]
    fn canonical_app_icon_decodes() {
        assert!(load_window_icon().is_some());
    }
}
