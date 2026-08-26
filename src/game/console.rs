use crate::game::{presentation, world};
use crate::model::GameState;
use crate::ui;
use crossterm::event::KeyCode;
use std::io;
use std::path::Path;

#[allow(dead_code)]
mod legacy {
    include!("console_fixed.rs");
}

pub(crate) fn choose_main_menu(
    state: &mut GameState,
    save_path: &Path,
    title: &str,
    options: &[String],
) -> io::Result<Option<usize>> {
    if options.is_empty() {
        return Ok(None);
    }

    let mut selected = 0usize;

    loop {
        ui::render_main_menu(title, options, selected)?;

        match ui::read_key()? {
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
            KeyCode::Char('/') => {
                open_console(state, save_path)?;
                presentation::render_state(state);
                presentation::maybe_run_location_scene(state)?;
            }
            _ => {}
        }
    }
}

fn open_console(state: &mut GameState, save_path: &Path) -> io::Result<()> {
    let mut background =
        ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(io::stdout()))?;
    background.clear()?;
    let result = legacy::open_console(state, save_path);
    let cleanup = background.clear();

    if let Ok(()) = result {
        world::bootstrap_campaign_content(state);
    }

    match (result, cleanup) {
        (Err(error), _) => Err(error),
        (Ok(()), Err(error)) => Err(error),
        (Ok(()), Ok(())) => Ok(()),
    }
}
