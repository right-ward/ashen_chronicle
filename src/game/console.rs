use crate::game::presentation;
use crate::model::GameState;
use crossterm::event::{self, Event, KeyCode};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::prelude::{Color, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Terminal;
use std::io::{self};
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

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let mut selected = 0usize;

    loop {
        presentation::render_state(state);

        terminal.draw(|frame| {
            let area = frame.area();
            let compact = area.width <= 112
                || area.height <= 36
                || area.width <= area.height.saturating_mul(2);
            let base_height = if compact { 7 } else { 6 };
            let max_height = if compact {
                area.height.saturating_mul(40) / 100
            } else {
                area.height.saturating_mul(34) / 100
            };
            let content_lines = options.len() + 4;
            let panel_height = (content_lines as u16)
                .saturating_add(2)
                .clamp(base_height, max_height.max(base_height))
                .min(area.height);
            let panel = Rect {
                x: area.x,
                y: area.y + area.height.saturating_sub(panel_height),
                width: area.width,
                height: panel_height,
            };

            frame.render_widget(Clear, panel);
            let mut lines = vec![
                Line::from(title),
                Line::from(""),
                Line::from("↑ ↓ / j k  Enter: choose  Esc: back  /: console"),
                Line::from(""),
            ];
            for (index, option) in options.iter().enumerate() {
                let marker = if index == selected { '▶' } else { ' ' };
                lines.push(Line::from(format!("{marker} {}. {}", index + 1, option)));
            }

            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black));
            frame.render_widget(
                Paragraph::new(lines)
                    .block(block)
                    .wrap(Wrap { trim: false }),
                panel,
            );
        })?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        match key.code {
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

fn open_console(state: &mut GameState, save_path: &Path) -> io::Result<()> {
    let mut background = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    background.clear()?;
    drop(background);
    legacy::open_console(state, save_path)
}
