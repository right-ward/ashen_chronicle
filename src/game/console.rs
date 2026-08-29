#[path = "console_commands.rs"]
mod commands;
#[path = "console_ui.rs"]
mod console_ui;

use crate::game::world;
use crate::model::GameState;
use crossterm::cursor;
use crossterm::event::KeyCode;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::Path;

pub(crate) fn choose_main_menu(
    state: &mut GameState,
    save_path: &Path,
    title: &str,
    options: &[String],
) -> io::Result<Option<usize>> {
    if options.is_empty() {
        return console_ui::choose_main_menu(state, save_path, title, options);
    }

    let mut selected = 0usize;
    loop {
        crate::ui::render_main_menu(title, options, selected)?;

        match crate::ui::read_key()? {
            KeyCode::Up | KeyCode::Char('k') => {
                selected = selected
                    .checked_sub(1)
                    .unwrap_or(options.len().saturating_sub(1));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                selected = (selected + 1) % options.len();
            }
            KeyCode::Home => selected = 0,
            KeyCode::End => selected = options.len().saturating_sub(1),
            KeyCode::Enter => return Ok(Some(selected)),
            KeyCode::Esc => return Ok(None),
            KeyCode::Char('/') => open_console(state, save_path)?,
            _ => {}
        }
    }
}

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
            console_ui::draw_console(&mut terminal, &console)?;
            let key = crate::ui::read_key()?;

            if console.autocomplete {
                match key {
                    KeyCode::Up => console_ui::select_previous(&mut console),
                    KeyCode::Down => console_ui::select_next(&mut console),
                    KeyCode::Enter => console_ui::accept_completion(&mut console),
                    KeyCode::Esc => console_ui::cancel_completion(&mut console),
                    KeyCode::Tab => {}
                    _ => {
                        console_ui::cancel_completion(&mut console);
                        console_ui::edit_input(&mut console, key);
                    }
                }
                continue;
            }

            match key {
                KeyCode::Esc => return Ok(()),
                KeyCode::Enter => {
                    commands::execute_line(state, save_path, &mut console)?;
                    if console.exit {
                        return Ok(());
                    }
                }
                KeyCode::Tab => {
                    console_ui::refresh_completion(&mut console, state);
                    if !console.candidates.is_empty() {
                        console.autocomplete = true;
                        console.selected = 0;
                        console.completion_scroll = 0;
                        console_ui::keep_completion_selection_visible(&mut console, 8);
                    }
                }
                KeyCode::Up => console_ui::history_previous(&mut console),
                KeyCode::Down => console_ui::history_next(&mut console),
                KeyCode::Home => console_ui::jump_home(&mut console),
                KeyCode::End => console_ui::jump_end(&mut console),
                KeyCode::PageUp => console_ui::scroll_up(&mut console, 6),
                KeyCode::PageDown => console_ui::scroll_down(&mut console, 6),
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
