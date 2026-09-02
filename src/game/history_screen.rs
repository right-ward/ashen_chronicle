use crate::game::time::time_display;
use crate::input::{self, InputEvent};
use crate::model::{GameState, HistoryEntryType};
use crate::presentation::{CharacterView, HistoryEntryView, HistoryEntryViewType, HistoryView};
use crate::ui;
use crate::ui_components;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use std::io;

pub(crate) fn run(state: &GameState) -> io::Result<()> {
    let view = build_view(state);
    let mut selected = 0usize;

    if view.entries.is_empty() {
        return draw_empty_history();
    }

    loop {
        selected = selected.min(view.entries.len().saturating_sub(1));
        draw_list(&view, selected)?;

        match input::read()? {
            InputEvent::Up | InputEvent::Character('k') => {
                selected = selected.checked_sub(1).unwrap_or(view.entries.len() - 1);
            }
            InputEvent::Down | InputEvent::Character('j') => {
                selected = (selected + 1) % view.entries.len();
            }
            InputEvent::Home => selected = 0,
            InputEvent::End => selected = view.entries.len() - 1,
            InputEvent::Confirm => show_detail(&view.entries[selected])?,
            InputEvent::Cancel => return Ok(()),
            _ => {}
        }
    }
}

fn build_view(state: &GameState) -> HistoryView {
    HistoryView {
        world_name: state.world.name.clone(),
        time: time_display(state.world.time_points, state.world.day),
        character: CharacterView {
            name: state.character.name.clone(),
            title: state.character.title.clone(),
            hp: state.character.hp,
            max_hp: state.character.max_hp,
        },
        entries: state
            .world
            .history
            .iter()
            .rev()
            .map(|entry| HistoryEntryView {
                day: entry.turn,
                entry_type: match entry.entry_type {
                    HistoryEntryType::Event => HistoryEntryViewType::Event,
                    HistoryEntryType::Narrative => HistoryEntryViewType::Narrative,
                },
                text: entry.text.clone(),
                event_id: entry.event_id.clone(),
                location_name: entry.location_name.clone(),
                outcome: entry.outcome.clone(),
            })
            .collect(),
    }
}

fn draw_list(view: &HistoryView, selected: usize) -> io::Result<()> {
    ui::draw_combat_screen(|frame, area| {
        let compact = ui_components::is_compact(area);
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
            .spacing(ui_components::overlap_spacing())
            .split(outer);

        draw_header(frame, root[0], view, compact);
        draw_entries(frame, root[1], view, selected, compact);
        draw_controls(frame, root[2], compact);
    })
}

fn draw_header(frame: &mut ratatui::Frame<'_>, area: Rect, view: &HistoryView, compact: bool) {
    ui_components::render_panel(
        frame,
        area,
        "World History",
        &[
            format!("World: {}", view.world_name),
            view.character.display_name(),
            view.time.clone(),
        ],
        compact,
    );
}

fn draw_entries(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    view: &HistoryView,
    selected: usize,
    compact: bool,
) {
    let visible_rows = area.height.saturating_sub(2).max(1) as usize;
    let mut start = selected.saturating_sub(visible_rows / 2);
    let max_start = view.entries.len().saturating_sub(visible_rows);
    start = start.min(max_start);
    let end = (start + visible_rows).min(view.entries.len());

    let mut lines = Vec::new();
    if start > 0 {
        lines.push("⋯ more above ⋯".to_string());
    }
    for (row, entry) in view.entries[start..end].iter().enumerate() {
        let absolute_index = start + row;
        let marker = entry_marker(entry);
        let selector = if absolute_index == selected { '▶' } else { ' ' };
        lines.push(format!(
            "{selector} Day {} {marker} {}",
            entry.day, entry.text
        ));
    }
    if end < view.entries.len() {
        lines.push("⋯ more below ⋯".to_string());
    }

    ui_components::render_panel(frame, area, "Chronicle", &lines, compact);
}

fn draw_controls(frame: &mut ratatui::Frame<'_>, area: Rect, compact: bool) {
    ui_components::render_panel(
        frame,
        area,
        "Controls",
        &["↑ ↓ / j k · Enter: details · Esc: back".to_string()],
        compact,
    );
}

fn show_detail(entry: &HistoryEntryView) -> io::Result<()> {
    ui::draw_combat_screen(|frame, area| draw_detail(frame, area, entry))?;
    loop {
        match input::read()? {
            InputEvent::Confirm | InputEvent::Cancel => return Ok(()),
            _ => {}
        }
    }
}

fn draw_detail(frame: &mut ratatui::Frame<'_>, area: Rect, entry: &HistoryEntryView) {
    let compact = ui_components::is_compact(area);
    let margin = if compact { 1 } else { 2 };
    let outer = Rect {
        x: area.x + margin.min(area.width.saturating_sub(1)),
        y: area.y + margin.min(area.height.saturating_sub(1)),
        width: area.width.saturating_sub(margin.saturating_mul(2)).max(1),
        height: area.height.saturating_sub(margin.saturating_mul(2)).max(1),
    };

    let title = match entry.entry_type {
        HistoryEntryViewType::Event => "History Entry · Event",
        HistoryEntryViewType::Narrative => "History Entry · Narrative",
    };
    let mut lines = vec![
        format!("Day {}", entry.day),
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

    ui_components::render_panel(frame, outer, title, &lines, compact);
}

fn draw_empty_history() -> io::Result<()> {
    ui::draw_combat_screen(|frame, area| {
        let compact = ui_components::is_compact(area);
        ui_components::render_panel(
            frame,
            area,
            "World History",
            &[
                "The world has not recorded any history yet.".to_string(),
                String::new(),
                "Press Esc to return.".to_string(),
            ],
            compact,
        );
    })?;

    loop {
        if matches!(input::read()?, InputEvent::Cancel | InputEvent::Confirm) {
            return Ok(());
        }
    }
}

fn entry_marker(entry: &HistoryEntryView) -> &'static str {
    match entry.entry_type {
        HistoryEntryViewType::Event => "[EVENT]",
        HistoryEntryViewType::Narrative => "[NOTE]",
    }
}
