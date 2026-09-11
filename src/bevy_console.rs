use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use crate::bevy_lifecycle::LifecycleState;
use crate::bevy_presentation::{self, BevyScreenRoot, ConsoleInputQueue, NavigationState, ScreenId};
use crate::game::console::{self, ConsoleState, ScrollPosition};
use crate::input::InputEvent;

#[derive(Resource)]
pub(crate) struct BevyConsoleState {
    pub(crate) console: ConsoleState,
    dirty: bool,
}

impl Default for BevyConsoleState {
    fn default() -> Self {
        Self { console: new_console_state(), dirty: true }
    }
}

fn new_console_state() -> ConsoleState {
    let mut console = ConsoleState::default();
    console.output.push("Ashen Chronicle developer console".into());
    console.output.push("help for commands | Tab completion | Esc closes".into());
    console
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<BevyConsoleState>()
        .add_systems(Update, (text_input, console_input, render_if_active).chain());
}

fn text_input(
    mut keyboard: MessageReader<KeyboardInput>,
    navigation: Res<NavigationState>,
    mut state: ResMut<BevyConsoleState>,
) {
    if navigation.current_screen != Some(ScreenId::Console) || state.console.autocomplete {
        return;
    }
    for event in keyboard.read() {
        if let Some(text) = &event.text {
            for character in text.chars().filter(|character| !character.is_control()) {
                state.console.input.push(character);
                state.console.history_index = None;
                state.dirty = true;
            }
        }
    }
}

fn console_input(
    mut state: ResMut<BevyConsoleState>,
    mut lifecycle: ResMut<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
    mut input_queue: ResMut<ConsoleInputQueue>,
) {
    if navigation.current_screen != Some(ScreenId::Console) || input_queue.0.is_empty() {
        return;
    }
    let events = std::mem::take(&mut input_queue.0);
    let Some(session) = lifecycle.session.as_mut() else {
        navigation.current_screen = navigation.return_screen.take();
        state.dirty = true;
        return;
    };

    for event in events {
        match event {
            InputEvent::Cancel => {
                console::bootstrap_after_console(&mut session.state);
                leave_console(&mut state, &mut navigation);
                break;
            }
            InputEvent::Confirm => {
                if state.console.autocomplete {
                    console::accept_completion(&mut state.console);
                } else if let Err(error) = console::execute_line(
                    &mut session.state,
                    &session.save_path,
                    &mut state.console,
                ) {
                    state.console.output.push(format!("Command failed: {error}"));
                }
                if state.console.exit {
                    console::bootstrap_after_console(&mut session.state);
                    leave_console(&mut state, &mut navigation);
                    break;
                }
                state.dirty = true;
            }
            InputEvent::Tab => {
                if state.console.autocomplete {
                    console::accept_completion(&mut state.console);
                } else {
                    console::refresh_completion(&mut state.console, &session.state);
                    if !state.console.candidates.is_empty() {
                        state.console.autocomplete = true;
                        state.console.selected = 0;
                        state.console.completion_scroll = 0;
                        console::keep_completion_selection_visible(&mut state.console, 8);
                    }
                }
                state.dirty = true;
            }
            InputEvent::Up => {
                if state.console.autocomplete {
                    console::select_previous(&mut state.console);
                } else {
                    console::history_previous(&mut state.console);
                }
                state.dirty = true;
            }
            InputEvent::Down => {
                if state.console.autocomplete {
                    console::select_next(&mut state.console);
                } else {
                    console::history_next(&mut state.console);
                }
                state.dirty = true;
            }
            InputEvent::Home => {
                console::jump_home(&mut state.console);
                state.dirty = true;
            }
            InputEvent::End => {
                console::jump_end(&mut state.console);
                state.dirty = true;
            }
            InputEvent::PageUp => {
                console::scroll_up(&mut state.console, 6);
                state.dirty = true;
            }
            InputEvent::PageDown => {
                console::scroll_down(&mut state.console, 6);
                state.dirty = true;
            }
            InputEvent::Backspace => {
                console::edit_input(&mut state.console, InputEvent::Backspace);
                state.dirty = true;
            }
            InputEvent::Delete | InputEvent::Character(_) => {}
        }
    }
}

fn leave_console(state: &mut BevyConsoleState, navigation: &mut NavigationState) {
    navigation.current_screen = navigation.return_screen.take().or(Some(ScreenId::Gameplay));
    navigation.selected = 0;
    state.console = new_console_state();
    state.dirty = true;
}

fn render_if_active(
    mut commands: Commands,
    mut state: ResMut<BevyConsoleState>,
    navigation: Res<NavigationState>,
    roots: Query<Entity, With<BevyScreenRoot>>,
) {
    if navigation.current_screen != Some(ScreenId::Console) || !state.dirty {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    render(&mut commands, &state.console);
    state.dirty = false;
}

fn render(commands: &mut Commands, console_state: &ConsoleState) {
    let root = bevy_presentation::spawn_screen(commands, "DEVELOPER CONSOLE");
    let panel = bevy_presentation::spawn_panel(commands, root);
    bevy_presentation::spawn_muted_label(commands, panel, "Enter commands directly. Esc closes the console.");
    for line in visible_output(console_state) {
        bevy_presentation::spawn_label(commands, panel, line);
    }
    bevy_presentation::spawn_label(commands, panel, format!("> {}", console_state.input));
    if console_state.autocomplete && !console_state.candidates.is_empty() {
        bevy_presentation::spawn_muted_label(commands, panel, "Completions:");
        let visible = console_state.candidates.len().min(8);
        let start = console_state.completion_scroll.min(console_state.candidates.len().saturating_sub(visible));
        for (index, candidate) in console_state.candidates.iter().enumerate().skip(start).take(visible) {
            let marker = if index == console_state.selected { ">" } else { " " };
            bevy_presentation::spawn_label(commands, panel, format!("{marker} {} — {}", candidate.value, candidate.hint));
        }
    }
}

fn visible_output(console_state: &ConsoleState) -> Vec<String> {
    const VISIBLE_LINES: usize = 24;
    if console_state.output.len() <= VISIBLE_LINES {
        return console_state.output.clone();
    }
    match console_state.scroll {
        ScrollPosition::Follow | ScrollPosition::Offset(0) => console_state.output[console_state.output.len() - VISIBLE_LINES..].to_vec(),
        ScrollPosition::Offset(offset) => {
            let end = console_state.output.len().saturating_sub(offset);
            let start = end.saturating_sub(VISIBLE_LINES);
            console_state.output[start..end].to_vec()
        }
        ScrollPosition::Home => console_state.output[..VISIBLE_LINES].to_vec(),
    }
}
