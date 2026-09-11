use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;

use crate::bevy_lifecycle::LifecycleState;
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, ScreenId,
};
use crate::game::console::ConsoleSession;
use crate::input::InputEvent;
use crate::presentation::{ConsoleScrollView, ConsoleView};

#[derive(Resource)]
pub(crate) struct BevyConsoleState {
    pub(crate) console: ConsoleSession,
    dirty: bool,
}

impl Default for BevyConsoleState {
    fn default() -> Self {
        Self {
            console: ConsoleSession::default(),
            dirty: true,
        }
    }
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<BevyConsoleState>().add_systems(
        Update,
        (open_shortcut, text_input, console_input, render_if_active).chain(),
    );
}

fn open_shortcut(
    keyboard: Res<ButtonInput<KeyCode>>,
    lifecycle: Res<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
) {
    if lifecycle.phase == crate::bevy_lifecycle::LifecyclePhase::Complete
        && lifecycle.session.is_some()
        && navigation.current_screen == Some(ScreenId::Gameplay)
        && keyboard.just_pressed(KeyCode::Slash)
    {
        navigation.return_screen = Some(ScreenId::Gameplay);
        navigation.current_screen = Some(ScreenId::Console);
        navigation.selected = 0;
    }
}

fn text_input(
    mut keyboard: MessageReader<KeyboardInput>,
    navigation: Res<NavigationState>,
    mut state: ResMut<BevyConsoleState>,
) {
    if navigation.current_screen != Some(ScreenId::Console) || state.console.is_autocomplete() {
        return;
    }
    for event in keyboard.read() {
        if let Some(text) = &event.text {
            state.console.push_text(text);
            state.dirty = true;
        }
    }
}

fn console_input(
    mut state: ResMut<BevyConsoleState>,
    mut lifecycle: ResMut<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
    mut input_queue: ResMut<GameplayInputQueue>,
) {
    if navigation.current_screen != Some(ScreenId::Console) || input_queue.0.is_empty() {
        return;
    }
    let events = std::mem::take(&mut input_queue.0);
    let Some(session) = lifecycle.session.as_mut() else {
        leave_console(&mut state, &mut navigation);
        return;
    };

    for event in events {
        match event {
            InputEvent::Cancel => {
                if state.console.is_autocomplete() {
                    state.console.cancel_completion();
                    state.dirty = true;
                } else {
                    crate::game::console::bootstrap_after_console(&mut session.state);
                    leave_console(&mut state, &mut navigation);
                    break;
                }
            }
            InputEvent::Confirm => {
                if state.console.is_autocomplete() {
                    state.console.accept_completion();
                } else if let Err(error) = state
                    .console
                    .execute_line(&mut session.state, &session.save_path)
                {
                    state
                        .console
                        .output_error(&format!("Command failed: {error}"));
                }
                if state.console.should_exit() {
                    crate::game::console::bootstrap_after_console(&mut session.state);
                    leave_console(&mut state, &mut navigation);
                    break;
                }
                state.dirty = true;
            }
            InputEvent::Tab => {
                if state.console.is_autocomplete() {
                    state.console.accept_completion();
                } else {
                    state.console.start_completion(&session.state);
                }
                state.dirty = true;
            }
            InputEvent::Up => {
                if state.console.is_autocomplete() {
                    state.console.select_previous_completion();
                } else {
                    state.console.history_previous();
                }
                state.dirty = true;
            }
            InputEvent::Down => {
                if state.console.is_autocomplete() {
                    state.console.select_next_completion();
                } else {
                    state.console.history_next();
                }
                state.dirty = true;
            }
            InputEvent::Home => {
                state.console.jump_home();
                state.dirty = true;
            }
            InputEvent::End => {
                state.console.jump_end();
                state.dirty = true;
            }
            InputEvent::PageUp => {
                state.console.scroll_up(6);
                state.dirty = true;
            }
            InputEvent::PageDown => {
                state.console.scroll_down(6);
                state.dirty = true;
            }
            InputEvent::Backspace => {
                state.console.edit(InputEvent::Backspace);
                state.dirty = true;
            }
            InputEvent::Delete | InputEvent::Character(_) => {}
            InputEvent::Other => {}
        }
    }
}

fn leave_console(state: &mut BevyConsoleState, navigation: &mut NavigationState) {
    navigation.current_screen = navigation.return_screen.take().or(Some(ScreenId::Gameplay));
    navigation.selected = 0;
    state.console = ConsoleSession::default();
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
    let view = state.console.view();
    render(&mut commands, &view);
    state.dirty = false;
}

fn render(commands: &mut Commands, view: &ConsoleView) {
    let root = bevy_presentation::spawn_screen(commands, "DEVELOPER CONSOLE");
    let panel = bevy_presentation::spawn_panel(commands, root);
    bevy_presentation::spawn_muted_label(
        commands,
        panel,
        "Enter commands directly. Esc closes the console.",
    );
    for line in visible_output(view) {
        bevy_presentation::spawn_label(commands, panel, line);
    }
    bevy_presentation::spawn_label(commands, panel, format!("> {}", view.input));
    if view.autocomplete && !view.candidates.is_empty() {
        bevy_presentation::spawn_muted_label(commands, panel, "Completions:");
        let visible = view.candidates.len().min(8);
        let start = view
            .completion_scroll
            .min(view.candidates.len().saturating_sub(visible));
        for (index, candidate) in view.candidates.iter().enumerate().skip(start).take(visible) {
            let marker = if index == view.selected { ">" } else { " " };
            bevy_presentation::spawn_label(
                commands,
                panel,
                format!("{marker} {} — {}", candidate.value, candidate.hint),
            );
        }
    }
}

fn visible_output(view: &ConsoleView) -> Vec<String> {
    const VISIBLE_LINES: usize = 24;
    if view.output.len() <= VISIBLE_LINES {
        return view.output.clone();
    }
    match view.scroll {
        ConsoleScrollView::Follow | ConsoleScrollView::Offset(0) => {
            view.output[view.output.len() - VISIBLE_LINES..].to_vec()
        }
        ConsoleScrollView::Offset(offset) => {
            let end = view.output.len().saturating_sub(offset);
            let start = end.saturating_sub(VISIBLE_LINES);
            view.output[start..end].to_vec()
        }
        ConsoleScrollView::Home => view.output[..VISIBLE_LINES].to_vec(),
    }
}
