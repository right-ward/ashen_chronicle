#[path = "console_commands.rs"]
mod commands;
#[path = "console_ui.rs"]
mod console_ui;

use crate::game::world;
use crate::input::{self, InputEvent};
use crate::model::GameState;
use crossterm::cursor;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::Path;

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

fn run_console_session(state: &mut GameState, save_path: &Path) -> io::Result<()> {
    crate::ui::set_console_input_active(true);
    let result = (|| -> io::Result<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
        terminal.clear()?;
        let mut console = console_ui::ConsoleState::default();
        console
            .output
            .push("Ashen Chronicle developer console".into());
        console
            .output
            .push("help for commands | Tab completion | Esc closes".into());
        console.output.extend(crate::ui::take_key_log());

        loop {
            console_ui::refresh_completion(&mut console, state);
            let view = console_ui::build_view(&console);
            console_ui::draw_console(&mut terminal, &view)?;
            let key = input::read()?;

            if console.autocomplete {
                match key {
                    InputEvent::Up => console_ui::select_previous(&mut console),
                    InputEvent::Down => console_ui::select_next(&mut console),
                    InputEvent::Confirm => console_ui::accept_completion(&mut console),
                    InputEvent::Cancel => console_ui::cancel_completion(&mut console),
                    InputEvent::Tab => {}
                    _ => {
                        console_ui::cancel_completion(&mut console);
                        console_ui::edit_input(&mut console, key);
                    }
                }
                continue;
            }

            match key {
                InputEvent::Cancel => return Ok(()),
                InputEvent::Confirm => {
                    commands::execute_line(state, save_path, &mut console)?;
                    if console.exit {
                        return Ok(());
                    }
                }
                InputEvent::Tab => {
                    console_ui::refresh_completion(&mut console, state);
                    if !console.candidates.is_empty() {
                        console.autocomplete = true;
                        console.selected = 0;
                        console.completion_scroll = 0;
                        console_ui::keep_completion_selection_visible(&mut console, 8);
                    }
                }
                InputEvent::Up => console_ui::history_previous(&mut console),
                InputEvent::Down => console_ui::history_next(&mut console),
                InputEvent::Home => console_ui::jump_home(&mut console),
                InputEvent::End => console_ui::jump_end(&mut console),
                InputEvent::PageUp => console_ui::scroll_up(&mut console, 6),
                InputEvent::PageDown => console_ui::scroll_down(&mut console, 6),
                _ => console_ui::edit_input(&mut console, key),
            }
        }
    })();
    crate::ui::set_console_input_active(false);
    result
}

fn enter_console_screen() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    Ok(())
}

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
