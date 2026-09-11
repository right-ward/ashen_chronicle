#[path = "console_commands.rs"]
mod commands;
#[path = "console_ui.rs"]
mod console_ui;

use crate::game::world;
use crate::input::{self, InputEvent};
use crate::model::GameState;
use crate::presentation::ConsoleView;
#[cfg(not(feature = "bevy"))]
use crossterm::cursor;
#[cfg(not(feature = "bevy"))]
use crossterm::execute;
#[cfg(not(feature = "bevy"))]
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
#[cfg(not(feature = "bevy"))]
use ratatui::backend::CrosstermBackend;
#[cfg(not(feature = "bevy"))]
use ratatui::Terminal;
use std::io;
use std::path::Path;

pub(crate) struct ConsoleSession {
    state: console_ui::ConsoleState,
}

impl Default for ConsoleSession {
    fn default() -> Self {
        let mut state = console_ui::ConsoleState::default();
        state
            .output
            .push("Ashen Chronicle developer console".into());
        state
            .output
            .push("help for commands | Tab completion | Esc closes".into());
        Self { state }
    }
}

impl ConsoleSession {
    pub(crate) fn view(&self) -> ConsoleView {
        console_ui::build_view(&self.state)
    }

    pub(crate) fn push_text(&mut self, text: &str) {
        for character in text.chars().filter(|character| !character.is_control()) {
            self.state.input.push(character);
            self.state.history_index = None;
        }
    }

    pub(crate) fn edit(&mut self, key: InputEvent) {
        console_ui::edit_input(&mut self.state, key);
    }

    pub(crate) fn history_previous(&mut self) {
        console_ui::history_previous(&mut self.state);
    }

    pub(crate) fn history_next(&mut self) {
        console_ui::history_next(&mut self.state);
    }

    pub(crate) fn scroll_up(&mut self, amount: usize) {
        console_ui::scroll_up(&mut self.state, amount);
    }

    pub(crate) fn scroll_down(&mut self, amount: usize) {
        console_ui::scroll_down(&mut self.state, amount);
    }

    pub(crate) fn jump_home(&mut self) {
        console_ui::jump_home(&mut self.state);
    }

    pub(crate) fn jump_end(&mut self) {
        console_ui::jump_end(&mut self.state);
    }

    pub(crate) fn is_autocomplete(&self) -> bool {
        self.state.autocomplete
    }

    pub(crate) fn start_completion(&mut self, game_state: &GameState) -> bool {
        console_ui::refresh_completion(&mut self.state, game_state);
        if self.state.candidates.is_empty() {
            return false;
        }
        self.state.autocomplete = true;
        self.state.selected = 0;
        self.state.completion_scroll = 0;
        console_ui::keep_completion_selection_visible(&mut self.state, 8);
        true
    }

    pub(crate) fn select_previous_completion(&mut self) {
        console_ui::select_previous(&mut self.state);
    }

    pub(crate) fn select_next_completion(&mut self) {
        console_ui::select_next(&mut self.state);
    }

    pub(crate) fn accept_completion(&mut self) {
        console_ui::accept_completion(&mut self.state);
    }

    pub(crate) fn execute_line(&mut self, state: &mut GameState, save_path: &Path) -> io::Result<()> {
        commands::execute_line(state, save_path, &mut self.state)
    }

    pub(crate) fn should_exit(&self) -> bool {
        self.state.exit
    }

    pub(crate) fn bootstrap(&mut self, state: &mut GameState) {
        world::bootstrap_campaign_content(state);
        self.state.exit = false;
    }
}

pub(crate) fn execute_line(
    state: &mut GameState,
    save_path: &Path,
    console: &mut ConsoleSession,
) -> io::Result<()> {
    console.execute_line(state, save_path)
}

pub(crate) fn accept_completion(console: &mut ConsoleSession) {
    console.accept_completion();
}

pub(crate) fn refresh_completion(console: &mut ConsoleSession, state: &GameState) {
    console.start_completion(state);
}

pub(crate) fn keep_completion_selection_visible(_console: &mut ConsoleSession, _visible: usize) {}
pub(crate) fn select_previous(console: &mut ConsoleSession) {
    console.select_previous_completion();
}
pub(crate) fn select_next(console: &mut ConsoleSession) {
    console.select_next_completion();
}
pub(crate) fn edit_input(console: &mut ConsoleSession, key: InputEvent) {
    console.edit(key);
}
pub(crate) fn history_previous(console: &mut ConsoleSession) {
    console.history_previous();
}
pub(crate) fn history_next(console: &mut ConsoleSession) {
    console.history_next();
}
pub(crate) fn scroll_up(console: &mut ConsoleSession, amount: usize) {
    console.scroll_up(amount);
}
pub(crate) fn scroll_down(console: &mut ConsoleSession, amount: usize) {
    console.scroll_down(amount);
}
pub(crate) fn jump_home(console: &mut ConsoleSession) {
    console.jump_home();
}
pub(crate) fn jump_end(console: &mut ConsoleSession) {
    console.jump_end();
}
pub(crate) fn bootstrap_after_console(state: &mut GameState) {
    world::bootstrap_campaign_content(state);
}

#[cfg(not(feature = "bevy"))]
pub(crate) fn open_console(state: &mut GameState, save_path: &Path) -> io::Result<()> {
    enter_console_screen()?;
    let result = run_console_session(state, save_path);
    let restore = restore_game_screen();

    if result.is_ok() {
        world::bootstrap_campaign_content(state);
    }

    match (result, restore) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Ok(()), Ok(())) => Ok(()),
    }
}

#[cfg(feature = "bevy")]
pub(crate) fn open_console(_state: &mut GameState, _save_path: &Path) -> io::Result<()> {
    Ok(())
}

#[cfg(not(feature = "bevy"))]
fn run_console_session(state: &mut GameState, save_path: &Path) -> io::Result<()> {
    crate::ui::set_console_input_active(true);
    let result = (|| -> io::Result<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        terminal.clear()?;
        let mut console = ConsoleSession::default();
        console.output_bootstrap();

        loop {
            let view = console.view();
            console_ui::draw_console(&mut terminal, &view)?;
            let key = input::read()?;

            if console.is_autocomplete() {
                match key {
                    InputEvent::Up => console.select_previous_completion(),
                    InputEvent::Down => console.select_next_completion(),
                    InputEvent::Confirm => console.accept_completion(),
                    InputEvent::Cancel => console.cancel_completion(),
                    InputEvent::Tab => {}
                    _ => {
                        console.cancel_completion();
                        console.edit(key);
                    }
                }
                continue;
            }

            match key {
                InputEvent::Cancel => return Ok(()),
                InputEvent::Confirm => {
                    console.execute_line(state, save_path)?;
                    if console.should_exit() {
                        return Ok(());
                    }
                }
                InputEvent::Tab => {
                    if console.start_completion(state) {
                        continue;
                    }
                }
                InputEvent::Up => console.history_previous(),
                InputEvent::Down => console.history_next(),
                InputEvent::Home => console.jump_home(),
                InputEvent::End => console.jump_end(),
                InputEvent::PageUp => console.scroll_up(6),
                InputEvent::PageDown => console.scroll_down(6),
                _ => console.edit(key),
            }
        }
    })();
    crate::ui::set_console_input_active(false);
    result
}

#[cfg(not(feature = "bevy"))]
fn enter_console_screen() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    Ok(())
}

#[cfg(not(feature = "bevy"))]
fn restore_game_screen() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(
        stdout,
        Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        EnterAlternateScreen,
        cursor::Hide
    )?;
    Ok(())
}

impl ConsoleSession {
    #[cfg(not(feature = "bevy"))]
    fn output_bootstrap(&mut self) {}

    #[cfg(not(feature = "bevy"))]
    fn cancel_completion(&mut self) {
        self.state.autocomplete = false;
        self.state.candidates.clear();
        self.state.completion_scroll = 0;
    }
}
