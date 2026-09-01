use crate::game::time::time_display;
use crate::game::{console, dispatcher, menu};
use crate::model::{GameState, HistoryEntryType};
use crate::presentation::{
    CharacterView, HistoryEntryView, HistoryEntryViewType, LocationView, ThreatView, WorldView,
};
use crate::ui;
use crate::ui_components;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Alignment, Color};
use ratatui::widgets::{Paragraph, Wrap};
use std::io;
use std::path::Path;

pub(crate) fn run(state: &mut GameState, save_path: &Path) -> io::Result<bool> {
    let mut selected = 0usize;
    loop {
        if !state.character.alive {
            return Ok(false);
        }
        let actions = menu::build_main_menu(state);
        if actions.is_empty() {
            return Ok(false);
        }
        selected = selected.min(actions.len().saturating_sub(1));
        let view = build_view(state);
        let action_labels = actions
            .iter()
            .map(|entry| entry.label.clone())
            .collect::<Vec<_>>();
        draw(&view, &action_labels, selected)?;
        match ui::read_key()? {
            KeyCode::Up | KeyCode::Char('k') => {
                selected = selected.checked_sub(1).unwrap_or(actions.len() - 1)
            }
            KeyCode::Down | KeyCode::Char('j') => selected = (selected + 1) % actions.len(),
            KeyCode::Home => selected = 0,
            KeyCode::End => selected = actions.len() - 1,
            KeyCode::Enter => {
                if dispatcher::dispatch(state, actions[selected].action, save_path)? {
                    return Ok(true);
                }
            }
            KeyCode::Char('/') => console::open_console(state, save_path)?,
            KeyCode::Esc => {}
            KeyCode::Char(ch) if ch.is_ascii_digit() => {
                let index = ch.to_digit(10).unwrap_or(0) as usize;
                if index >= 1 && index <= actions.len() {
                    if dispatcher::dispatch(state, actions[index - 1].action, save_path)? {
                        return Ok(true);
                    }
                    selected = index - 1;
                }
            }
            _ => {}
        }
    }
}

fn build_view(state: &GameState) -> WorldView {
    let location = state
        .world
        .location_by_id(state.character.location_id)
        .map(|location| {
            let region_name = state
                .world
                .regions
                .iter()
                .find(|region| region.id == location.region_id)
                .map(|region| region.name.clone())
                .unwrap_or_else(|| "Unknown region".to_string());
            LocationView {
                name: location.name.clone(),
                description: location.description.clone(),
                region_name,
                dangerous: location.dangerous,
            }
        });
    let threat = if state.threat.active {
        Some(ThreatView {
            label: state.threat.label.clone(),
            description: state.threat.description.clone(),
        })
    } else {
        None
    };
    let history = state
        .world
        .history
        .iter()
        .rev()
        .take(5)
        .rev()
        .map(|entry| HistoryEntryView {
            day: entry.turn,
            entry_type: match entry.entry_type {
                HistoryEntryType::Event => HistoryEntryViewType::Event,
                HistoryEntryType::Narrative => HistoryEntryViewType::Narrative,
            },
            text: entry.text.clone(),
            location_name: entry.location_name.clone(),
            outcome: entry.outcome.clone(),
        })
        .collect();

    WorldView {
        world_name: state.world.name.clone(),
        time: time_display(state.world.time_points, state.world.day),
        character: CharacterView {
            name: state.character.name.clone(),
            title: state.character.title.clone(),
            hp: state.character.hp,
            max_hp: state.character.max_hp,
        },
        location,
        threat,
        history,
    }
}

fn draw(view: &WorldView, action_labels: &[String], selected: usize) -> io::Result<()> {
    ui::draw_combat_screen(|frame, area| draw_inner(frame, area, view, action_labels, selected))
}

fn draw_inner(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    view: &WorldView,
    action_labels: &[String],
    selected: usize,
) {
    let compact = ui_components::is_compact(area);
    let horizontal_margin = if compact { 1 } else { 2 };
    let vertical_margin = if compact { 1 } else { 2 };
    let outer = Rect {
        x: area.x + horizontal_margin.min(area.width.saturating_sub(1)),
        y: area.y + vertical_margin.min(area.height.saturating_sub(1)),
        width: area
            .width
            .saturating_sub(horizontal_margin.saturating_mul(2))
            .max(1),
        height: area
            .height
            .saturating_sub(vertical_margin.saturating_mul(2))
            .max(1),
    };
    let header_height = if compact { 7 } else { 8 };
    let body_min_height = 8u16;
    let available_action_height = outer
        .height
        .saturating_sub(header_height)
        .saturating_sub(body_min_height);
    let action_height = (action_labels.len() as u16 + 3)
        .min(available_action_height.max(6))
        .max(1);
    let root = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Min(body_min_height),
            Constraint::Length(action_height),
        ])
        .spacing(ui_components::overlap_spacing())
        .split(outer);
    draw_header(frame, root[0], view, compact);
    let body = Layout::default()
        .direction(ui_components::vertical_or_horizontal(compact))
        .constraints(if compact {
            [Constraint::Percentage(55), Constraint::Percentage(45)]
        } else {
            [Constraint::Percentage(58), Constraint::Percentage(42)]
        })
        .spacing(ui_components::overlap_spacing())
        .split(root[1]);
    draw_context(frame, body[0], view, compact);
    draw_history(frame, body[1], view, compact);
    draw_actions(frame, root[2], action_labels, selected, compact);
}

fn draw_header(frame: &mut ratatui::Frame<'_>, area: Rect, view: &WorldView, compact: bool) {
    let block = ui_components::panel_block("The Ashen Chronicle", compact);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let columns = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ])
        .split(inner);
    let location_name = view
        .location
        .as_ref()
        .map(|location| location.name.as_str())
        .unwrap_or("Unknown");
    let identity = format!(
        "World: {}\n{}\nLocation: {}",
        view.world_name,
        view.character.display_name(),
        location_name
    );
    frame.render_widget(
        Paragraph::new(identity).wrap(Wrap { trim: false }),
        columns[0],
    );
    frame.render_widget(
        Paragraph::new(view.time.clone())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false }),
        columns[1],
    );
    ui_components::render_health_gauge(
        frame,
        columns[2],
        &format!("HP: {}/{}", view.character.hp, view.character.max_hp.max(1)),
        view.character.hp,
        view.character.max_hp,
    );
}

fn draw_context(frame: &mut ratatui::Frame<'_>, area: Rect, view: &WorldView, compact: bool) {
    let Some(location) = &view.location else {
        ui_components::render_panel(
            frame,
            area,
            "World Context",
            &["You are lost in an unknown place.".to_string()],
            compact,
        );
        return;
    };
    let mut lines = vec![
        location.name.clone(),
        format!("Region: {}", location.region_name),
        String::new(),
    ];
    if !location.description.trim().is_empty() {
        lines.extend(location.description.lines().map(str::to_string));
    }
    if location.dangerous {
        lines.push(String::new());
        lines.push("Danger: this location is unsafe.".to_string());
    }
    lines.push(String::new());
    if let Some(threat) = &view.threat {
        lines.push(format!("Threat: {}", threat.label));
        if !threat.description.trim().is_empty() {
            lines.extend(threat.description.lines().map(str::to_string));
        }
    } else {
        lines.push("Threat: none active.".to_string());
    }
    ui_components::render_panel(frame, area, "World Context", &lines, compact);
}

fn draw_history(frame: &mut ratatui::Frame<'_>, area: Rect, view: &WorldView, compact: bool) {
    if view.history.is_empty() {
        ui_components::render_panel(
            frame,
            area,
            "Recent Events",
            &["Nothing has been recorded yet.".to_string()],
            compact,
        );
        return;
    }
    let lines = view
        .history
        .iter()
        .map(|entry| {
            let marker = match entry.entry_type {
                HistoryEntryViewType::Event => "[EVENT]",
                HistoryEntryViewType::Narrative => "[NOTE]",
            };
            format!("Day {} {} {}", entry.day, marker, entry.text)
        })
        .collect::<Vec<_>>();
    ui_components::render_panel(frame, area, "Recent Events", &lines, compact);
}

fn draw_actions(
    frame: &mut ratatui::Frame<'_>,
    area: Rect,
    action_labels: &[String],
    selected: usize,
    compact: bool,
) {
    let block = ui_components::panel_block("What will you do?", compact);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let visible_rows = inner.height.saturating_sub(2).max(1) as usize;
    let mut start = selected.saturating_sub(visible_rows.saturating_sub(1));
    start = start.min(action_labels.len().saturating_sub(visible_rows));
    let end = (start + visible_rows).min(action_labels.len());
    let mut lines = Vec::with_capacity(visible_rows + 2);
    lines.push("↑ ↓ / j k · Enter".to_string());
    if start > 0 {
        lines.push("⋯ more above ⋯".to_string());
    }
    for (index, label) in action_labels
        .iter()
        .enumerate()
        .take(end)
        .skip(start)
    {
        let marker = if index == selected { '▶' } else { ' ' };
        lines.push(format!("{marker} {}. {}", index + 1, label));
    }
    if end < action_labels.len() {
        lines.push("⋯ more below ⋯".to_string());
    }
    frame.render_widget(
        Paragraph::new(lines.join("\n")).wrap(Wrap { trim: false }),
        inner,
    );
}
