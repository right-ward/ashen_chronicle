use crate::game::time::time_display;
use crate::model::{GameState, HistoryEntry, HistoryEntryType};
use crate::ui;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect, Spacing};
use ratatui::prelude::{Alignment, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use std::io;

pub(crate) fn run(state: &GameState) -> io::Result<()> {
    let mut selected = 0usize;
    let entries: Vec<usize> = (0..state.world.history.len()).rev().collect();

    if entries.is_empty() {
        return draw_empty_history();
    }

    loop {
        selected = selected.min(entries.len().saturating_sub(1));
        draw_list(state, &entries, selected)?;

        match ui::read_key()? {
            KeyCode::Up | KeyCode::Char('k') => {
                selected = selected.checked_sub(1).unwrap_or(entries.len() - 1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                selected = (selected + 1) % entries.len();
            }
            KeyCode::Home => selected = 0,
            KeyCode::End => selected = entries.len() - 1,
            KeyCode::Enter => show_detail(state, entries[selected])?,
            KeyCode::Esc => return Ok(()),
            _ => {}
        }
    }
}

fn draw_list(state: &GameState, entries: &[usize], selected: usize) -> io::Result<()> {
    ui::draw_combat_screen(|frame, area| {
        let compact =
            area.width <= 112 || area.height <= 36 || area.width <= area.height.saturating_mul(2);
        let margin = if compact { 1 } else { 2 };
        let outer = Rect {
            x: area.x + margin.min(area.width.saturating_sub(1)),
            y: area.y + margin.min(area.height.saturating_sub(1)),
            width: area.width.saturating_sub(margin.saturating_mul(2)).max(1),
            height: area.height.saturating_sub(margin.saturating_mul(2)).max(1),
        };

        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(if compact { 6 } else { 7 }),
                Constraint::Min(6),
                Constraint::Length(3),
            ])
            .spacing(Spacing::Overlap(1))
            .split(outer);

        draw_header(frame, root[0], state, compact);
        draw_entries(frame, root[1], state, entries, selected, compact);
        draw_controls(frame, root[2], compact);
    })
}

fn draw_header(frame: &mut ratatui::Frame<'_>, area: Rect, state: &GameState, compact: bool) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("World History")
        .style(border_style(compact));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = format!(
        "World: {}\n{}\n{}",
        state.world.name,
        state.character.display_name(),
        time_display(state.world.time_points, state.world.day)
    );
    frame.render_widget(
        Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false }),
        inner,
    );
}

fn draw_entries(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    state: &GameState,
    entries: &[usize],
    selected: usize,
    compact: bool,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Chronicle")
        .style(border_style(compact));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let visible_rows = inner.height.max(1) as usize;
    let mut start = selected.saturating_sub(visible_rows / 2);
    let max_start = entries.len().saturating_sub(visible_rows);
    start = start.min(max_start);
    let end = (start + visible_rows).min(entries.len());

    let mut lines = Vec::new();
    if start > 0 {
        lines.push("⋯ more above ⋯".to_string());
    }
    for (row, history_index) in entries[start..end].iter().enumerate() {
        let absolute_index = start + row;
        let entry = &state.world.history[*history_index];
        let marker = entry_marker(entry);
        let selector = if absolute_index == selected {
            '▶'
        } else {
            ' '
        };
        lines.push(format!(
            "{selector} Day {} {marker} {}",
            entry.turn, entry.text
        ));
    }
    if end < entries.len() {
        lines.push("⋯ more below ⋯".to_string());
    }

    frame.render_widget(
        Paragraph::new(lines.join("\n")).wrap(Wrap { trim: false }),
        inner,
    );
}

fn draw_controls(frame: &mut ratatui::Frame<'_>, area: Rect, compact: bool) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Controls")
        .style(border_style(compact));
    let inner = block.inner(area);
    frame.render_widget(
        Paragraph::new("↑ ↓ / j k · Enter: details · Esc: back").alignment(Alignment::Center),
        inner,
    );
}

fn show_detail(state: &GameState, history_index: usize) -> io::Result<()> {
    let Some(entry) = state.world.history.get(history_index) else {
        return Ok(());
    };

    ui::draw_combat_screen(|frame, area| draw_detail(frame, area, entry))?;
    loop {
        match ui::read_key()? {
            KeyCode::Enter | KeyCode::Esc => return Ok(()),
            _ => {}
        }
    }
}

fn draw_detail(frame: &mut ratatui::Frame<'_>, area: Rect, entry: &HistoryEntry) {
    let compact =
        area.width <= 112 || area.height <= 36 || area.width <= area.height.saturating_mul(2);
    let margin = if compact { 1 } else { 2 };
    let outer = Rect {
        x: area.x + margin.min(area.width.saturating_sub(1)),
        y: area.y + margin.min(area.height.saturating_sub(1)),
        width: area.width.saturating_sub(margin.saturating_mul(2)).max(1),
        height: area.height.saturating_sub(margin.saturating_mul(2)).max(1),
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(match entry.entry_type {
            HistoryEntryType::Event => "History Entry · Event",
            HistoryEntryType::Narrative => "History Entry · Narrative",
        })
        .style(border_style(compact));
    let inner = block.inner(outer);
    frame.render_widget(block, outer);

    let mut lines = vec![
        format!("Day {}", entry.turn),
        String::new(),
        entry.text.clone(),
    ];
    if let Some(event_id) = &entry.event_id {
        lines.push(String::new());
        lines.push(format!("Event: {event_id}"));
    }
    if let Some(location) = &entry.location_name {
        lines.push(format!("Location: {location}"));
    }
    if let Some(outcome) = &entry.outcome {
        lines.push(String::new());
        lines.push(format!("Outcome: {outcome}"));
    }
    lines.push(String::new());
    lines.push("Enter / Esc: back to history".to_string());

    frame.render_widget(
        Paragraph::new(lines.join("\n")).wrap(Wrap { trim: false }),
        inner.inner(Margin {
            vertical: 1,
            horizontal: 2,
        }),
    );
}

fn draw_empty_history() -> io::Result<()> {
    ui::draw_combat_screen(|frame, area| {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("World History");
        let inner = block.inner(area);
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new("The world has not recorded any history yet.\n\nPress Esc to return.")
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false }),
            inner,
        );
    })?;

    loop {
        if matches!(ui::read_key()?, KeyCode::Esc | KeyCode::Enter) {
            return Ok(());
        }
    }
}

fn entry_marker(entry: &HistoryEntry) -> &'static str {
    match entry.entry_type {
        HistoryEntryType::Event => "[EVENT]",
        HistoryEntryType::Narrative => "[NOTE]",
    }
}

fn border_style(_compact: bool) -> Style {
    Style::default()
}
