#[path = "ui_impl.rs"]
mod ui_impl;
pub(crate) mod components {
    #![allow(unused_imports)]
    pub(crate) use crate::ui_components::*;
}

use crate::presentation::{ChoiceView, ScreenView};
use std::sync::{Mutex, OnceLock};

pub(crate) use ui_impl::{
    draw_combat_screen, key_logging_enabled, read_key, set_console_input_active, set_key_logging,
    take_key_log,
};
pub use ui_impl::{Dashboard, UiGuard};

#[derive(Default)]
struct ScreenState {
    title: String,
    subtitle: Option<String>,
    art: Option<String>,
    output: Vec<String>,
}
static SCREEN: OnceLock<Mutex<ScreenState>> = OnceLock::new();
fn screen_state() -> &'static Mutex<ScreenState> {
    SCREEN.get_or_init(|| Mutex::new(ScreenState::default()))
}

pub fn init() -> std::io::Result<UiGuard> {
    ui_impl::init()
}

pub fn set_menu_screen(title: impl Into<String>, subtitle: Option<String>, art: Option<String>) {
    let title = title.into();
    let art = art.or_else(|| {
        (title == "LEAVE?").then(|| {
            r#"        .-''''-.
       /  .--.  \\
      /  /    \\  \\
      | |      | |
      | |      | |
      |  \\____/  |
       \\        /
        '------'
"#
            .to_string()
        })
    });
    let mut state = screen_state().lock().unwrap();
    state.title = title.clone();
    state.subtitle = subtitle.clone();
    state.art = art.clone();
    state.output.clear();
    drop(state);
    ui_impl::set_menu_screen(title, subtitle, art);
}

pub(crate) fn show_screen_view(view: &ScreenView) {
    let mut lines = Vec::new();
    if !view.body.is_empty() {
        lines.extend(view.body.iter().cloned());
    }
    let subtitle = match &view.subtitle {
        Some(subtitle) if !subtitle.is_empty() && !lines.is_empty() => {
            let mut combined = String::with_capacity(subtitle.len() + 1 + lines.len() * 16);
            combined.push_str(subtitle);
            combined.push('\n');
            combined.push_str(&lines.join("\n"));
            Some(combined)
        }
        Some(subtitle) => Some(subtitle.clone()),
        None if !lines.is_empty() => Some(lines.join("\n")),
        None => None,
    };
    set_menu_screen(view.title.clone(), subtitle, view.art.clone());
}

pub(crate) fn choose_screen_view(view: &ChoiceView) -> std::io::Result<Option<usize>> {
    show_screen_view(&view.screen);
    choose_from_list(&view.prompt, &view.options, view.back_label.as_deref())
}

pub fn set_dashboard(dashboard: Dashboard) {
    let mut state = screen_state().lock().unwrap();
    state.title.clear();
    state.output.clear();
    drop(state);
    ui_impl::set_dashboard(dashboard);
}

pub fn set_player_health(current: i32, maximum: i32) {
    ui_impl::set_player_health(current, maximum);
}
pub fn set_combat_health(enemy_name: impl Into<String>, enemy_hp: i32, enemy_max_hp: i32) {
    ui_impl::set_combat_health(enemy_name, enemy_hp, enemy_max_hp);
}
pub fn clear_combat_health() {
    ui_impl::clear_combat_health();
}
pub fn set_location_scene(lines: Vec<String>) {
    ui_impl::set_location_scene(lines);
}

pub fn line(text: &str) {
    let mut state = screen_state().lock().unwrap();
    if state.title.is_empty() {
        drop(state);
        ui_impl::line(text);
        return;
    }
    state.output.extend(text.split('\n').map(str::to_string));
    if state.output.len() > 48 {
        let excess = state.output.len() - 48;
        state.output.drain(0..excess);
    }
    let screen = (
        state.title.clone(),
        state.subtitle.clone(),
        state.art.clone(),
        state.output.clone(),
    );
    drop(state);
    render_screen(screen, None);
}

pub fn clear_log() {
    screen_state().lock().unwrap().output.clear();
    ui_impl::clear_log();
}

pub fn diagnostic(text: &str) {
    line(&format!("[diagnostic] {text}"));
}
pub fn prompt(message: &str) -> std::io::Result<String> {
    ui_impl::prompt(message)
}

pub fn pause() {
    let state = screen_state().lock().unwrap();
    if state.title.is_empty() {
        drop(state);
        ui_impl::pause();
        return;
    }
    let screen = (
        state.title.clone(),
        state.subtitle.clone(),
        state.art.clone(),
        state.output.clone(),
    );
    drop(state);
    render_screen(screen, Some("Press any key to continue..."));
    let _ = ui_impl::read_key();
}

pub fn narrate(message: &str) {
    line(message);
    pause();
}
pub fn choose_from_list(
    title: &str,
    options: &[String],
    zero_label: Option<&str>,
) -> std::io::Result<Option<usize>> {
    ui_impl::choose_from_list(title, options, zero_label)
}

fn render_screen(
    screen: (String, Option<String>, Option<String>, Vec<String>),
    notice: Option<&str>,
) {
    let (title, subtitle, art, output) = screen;
    let mut lines = Vec::new();
    if let Some(subtitle) = subtitle {
        if !subtitle.is_empty() {
            lines.push(subtitle);
        }
    }
    if !output.is_empty() {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.extend(output);
    }
    if let Some(notice) = notice {
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push(notice.to_string());
    }
    ui_impl::set_menu_screen(title, (!lines.is_empty()).then(|| lines.join("\n")), art);
}
