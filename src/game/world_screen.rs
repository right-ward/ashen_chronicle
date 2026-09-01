use crate::game::time::time_display;
use crate::game::{console, dispatcher, menu};
use crate::model::{GameState, HistoryEntryType};
use crate::ui;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout, Rect, Spacing};
use ratatui::prelude::{Alignment, Color, Style};
use ratatui::widgets::{Block, Borders, LineGauge, Paragraph, Wrap};
use std::io;
use std::path::Path;

pub(crate) fn run(state: &mut GameState, save_path: &Path) -> io::Result<bool> {
    let mut selected = 0usize;
    loop {
        if !state.character.alive { return Ok(false); }
        let actions = menu::build_main_menu(state);
        if actions.is_empty() { return Ok(false); }
        selected = selected.min(actions.len().saturating_sub(1));
        draw(state, &actions, selected)?;
        match ui::read_key()? {
            KeyCode::Up | KeyCode::Char('k') => selected = selected.checked_sub(1).unwrap_or(actions.len() - 1),
            KeyCode::Down | KeyCode::Char('j') => selected = (selected + 1) % actions.len(),
            KeyCode::Home => selected = 0,
            KeyCode::End => selected = actions.len() - 1,
            KeyCode::Enter => { if dispatcher::dispatch(state, actions[selected].action, save_path)? { return Ok(true); } }
            KeyCode::Char('/') => console::open_console(state, save_path)?,
            KeyCode::Esc => {}
            KeyCode::Char(ch) if ch.is_ascii_digit() => { let index = ch.to_digit(10).unwrap_or(0) as usize; if index >= 1 && index <= actions.len() { if dispatcher::dispatch(state, actions[index - 1].action, save_path)? { return Ok(true); } selected = index - 1; } }
            _ => {}
        }
    }
}
fn draw(state: &GameState, actions: &[menu::MenuEntry], selected: usize) -> io::Result<()> { ui::draw_combat_screen(|frame, area| draw_inner(frame, area, state, actions, selected)) }
fn draw_inner(frame: &mut ratatui::Frame<'_>, area: Rect, state: &GameState, actions: &[menu::MenuEntry], selected: usize) {
    let compact = area.width <= 112 || area.height <= 36 || area.width <= area.height.saturating_mul(2);
    let horizontal_margin = if compact { 1 } else { 2 }; let vertical_margin = if compact { 1 } else { 2 };
    let outer = Rect { x: area.x + horizontal_margin.min(area.width.saturating_sub(1)), y: area.y + vertical_margin.min(area.height.saturating_sub(1)), width: area.width.saturating_sub(horizontal_margin.saturating_mul(2)).max(1), height: area.height.saturating_sub(vertical_margin.saturating_mul(2)).max(1) };
    let header_height = if compact { 7 } else { 8 }; let body_min_height = 8u16;
    let available_action_height = outer.height.saturating_sub(header_height).saturating_sub(body_min_height);
    let action_height = (actions.len() as u16 + 3).min(available_action_height.max(6)).max(1);
    let root = Layout::default().direction(Direction::Vertical).constraints([Constraint::Length(header_height), Constraint::Min(body_min_height), Constraint::Length(action_height)]).spacing(Spacing::Overlap(1)).split(outer);
    draw_header(frame, root[0], state, compact);
    let body = Layout::default().direction(if compact { Direction::Vertical } else { Direction::Horizontal }).constraints(if compact { [Constraint::Percentage(55), Constraint::Percentage(45)] } else { [Constraint::Percentage(58), Constraint::Percentage(42)] }).spacing(Spacing::Overlap(1)).split(root[1]);
    draw_context(frame, body[0], state, compact); draw_history(frame, body[1], state, compact); draw_actions(frame, root[2], actions, selected, compact);
}
fn draw_header(frame: &mut ratatui::Frame<'_>, area: Rect, state: &GameState, compact: bool) {
    let block = Block::default().borders(Borders::ALL).title("The Ashen Chronicle").style(border_style(compact)); let inner = block.inner(area); frame.render_widget(block, area);
    let columns = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage(34), Constraint::Percentage(33), Constraint::Percentage(33)]).split(inner);
    let identity = format!("World: {}\n{}\nLocation: {}", state.world.name, state.character.display_name(), current_location_name(state)); frame.render_widget(Paragraph::new(identity).wrap(Wrap { trim: false }), columns[0]);
    frame.render_widget(Paragraph::new(time_display(state.world.time_points, state.world.day)).alignment(Alignment::Center).wrap(Wrap { trim: false }), columns[1]);
    let maximum = state.character.max_hp.max(1); let current = state.character.hp.clamp(0, maximum);
    let gauge = LineGauge::default().ratio(current as f64 / maximum as f64).label(format!("HP: {}/{}", state.character.hp, maximum)).filled_symbol("█").unfilled_symbol("░").filled_style(Style::default().fg(Color::Indexed(124))).unfilled_style(Style::default().fg(Color::Gray)); frame.render_widget(gauge, columns[2]);
}
fn draw_context(frame: &mut ratatui::Frame<'_>, area: Rect, state: &GameState, compact: bool) {
    let block = Block::default().borders(Borders::ALL).title("World Context").style(border_style(compact)); let inner = block.inner(area); frame.render_widget(block, area);
    let Some(location) = state.world.location_by_id(state.character.location_id) else { frame.render_widget(Paragraph::new("You are lost in an unknown place."), inner); return; };
    let region_name = state.world.regions.iter().find(|region| region.id == location.region_id).map(|region| region.name.as_str()).unwrap_or("Unknown region"); let mut lines = vec![location.name.clone(), format!("Region: {region_name}"), String::new()];
    if !location.description.trim().is_empty() { lines.extend(location.description.lines().map(str::to_string)); } if location.dangerous { lines.push(String::new()); lines.push("Danger: this location is unsafe.".to_string()); } lines.push(String::new());
    if state.threat.active { lines.push(format!("Threat: {}", state.threat.label)); if !state.threat.description.trim().is_empty() { lines.extend(state.threat.description.lines().map(str::to_string)); } } else { lines.push("Threat: none active.".to_string()); }
    frame.render_widget(Paragraph::new(lines.join("\n")).wrap(Wrap { trim: false }), inner);
}
fn draw_history(frame: &mut ratatui::Frame<'_>, area: Rect, state: &GameState, compact: bool) {
    let block = Block::default().borders(Borders::ALL).title("Recent Events").style(border_style(compact)); let inner = block.inner(area); frame.render_widget(block, area); let history = state.world.history.iter().rev().take(5).collect::<Vec<_>>();
    if history.is_empty() { frame.render_widget(Paragraph::new("Nothing has been recorded yet."), inner); return; }
    let lines = history.iter().rev().map(|entry| { let marker = match entry.entry_type { HistoryEntryType::Event => "[EVENT]", HistoryEntryType::Narrative => "[NOTE]" }; format!("Day {} {} {}", entry.turn, marker, entry.text) }).collect::<Vec<_>>().join("\n"); frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}
fn draw_actions(frame: &mut ratatui::Frame<'_>, area: Rect, actions: &[menu::MenuEntry], selected: usize, compact: bool) {
    let block = Block::default().borders(Borders::ALL).title("What will you do?").style(border_style(compact)); let inner = block.inner(area); frame.render_widget(block, area);
    let visible_rows = inner.height.saturating_sub(2).max(1) as usize; let mut start = selected.saturating_sub(visible_rows.saturating_sub(1)); start = start.min(actions.len().saturating_sub(visible_rows)); let end = (start + visible_rows).min(actions.len());
    let mut lines = Vec::with_capacity(visible_rows + 2); lines.push("↑ ↓ / j k · Enter".to_string()); if start > 0 { lines.push("⋯ more above ⋯".to_string()); }
    for (index, entry) in actions.iter().enumerate().take(end).skip(start) { let marker = if index == selected { '▶' } else { ' ' }; lines.push(format!("{marker} {}. {}", index + 1, entry.label)); } if end < actions.len() { lines.push("⋯ more below ⋯".to_string()); }
    frame.render_widget(Paragraph::new(lines.join("\n")).wrap(Wrap { trim: false }), inner);
}
fn current_location_name(state: &GameState) -> String { state.world.location_by_id(state.character.location_id).map(|location| location.name.clone()).unwrap_or_else(|| "Unknown".to_string()) }
fn border_style(_compact: bool) -> Style { Style::default() }
