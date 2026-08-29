use crate::model::{EntityId, GameState};
use crate::ui;
use crossterm::event::KeyCode;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Terminal;
use std::io;
use std::path::Path;

#[derive(Clone)]
pub(super) struct Candidate {
    pub(super) value: String,
    pub(super) hint: String,
}

#[derive(Default)]
pub(super) struct ConsoleState {
    pub(super) output: Vec<String>,
    pub(super) input: String,
    pub(super) history: Vec<String>,
    pub(super) history_index: Option<usize>,
    pub(super) scroll: ScrollPosition,
    pub(super) completion_scroll: usize,
    pub(super) candidates: Vec<Candidate>,
    pub(super) selected: usize,
    pub(super) autocomplete: bool,
    pub(super) exit: bool,
}

#[derive(Default)]
pub(super) enum ScrollPosition {
    #[default]
    Follow,
    Offset(usize),
    Home,
}

pub(super) fn edit_input(console: &mut ConsoleState, key: KeyCode) {
    match key {
        KeyCode::Char(c) if !c.is_control() => {
            console.input.push(c);
            console.history_index = None;
        }
        KeyCode::Backspace => {
            console.input.pop();
            console.history_index = None;
        }
        _ => {}
    }
}

pub(super) fn history_previous(console: &mut ConsoleState) {
    if console.history.is_empty() {
        return;
    }
    let index = console
        .history_index
        .map_or(console.history.len() - 1, |i| i.saturating_sub(1));
    console.history_index = Some(index);
    console.input = console.history[index].clone();
}

pub(super) fn history_next(console: &mut ConsoleState) {
    let Some(index) = console.history_index else {
        return;
    };
    if index + 1 >= console.history.len() {
        console.history_index = None;
        console.input.clear();
        return;
    }
    let index = index + 1;
    console.history_index = Some(index);
    console.input = console.history[index].clone();
}

pub(super) fn scroll_up(console: &mut ConsoleState, amount: usize) {
    console.scroll = match console.scroll {
        ScrollPosition::Follow => ScrollPosition::Offset(amount),
        ScrollPosition::Offset(offset) => ScrollPosition::Offset(offset.saturating_add(amount)),
        ScrollPosition::Home => ScrollPosition::Home,
    };
}

pub(super) fn scroll_down(console: &mut ConsoleState, amount: usize) {
    console.scroll = match console.scroll {
        ScrollPosition::Follow => ScrollPosition::Follow,
        ScrollPosition::Home => ScrollPosition::Offset(0),
        ScrollPosition::Offset(offset) => {
            let next = offset.saturating_sub(amount);
            if next == 0 {
                ScrollPosition::Follow
            } else {
                ScrollPosition::Offset(next)
            }
        }
    };
}

pub(super) fn jump_home(console: &mut ConsoleState) {
    console.scroll = ScrollPosition::Home;
}

pub(super) fn jump_end(console: &mut ConsoleState) {
    console.scroll = ScrollPosition::Follow;
}

pub(super) fn refresh_completion(console: &mut ConsoleState, state: &GameState) {
    let (tokens, trailing) = tokenize(&console.input);
    let prefix = if trailing {
        ""
    } else {
        tokens.last().map(String::as_str).unwrap_or("")
    };
    let mut candidates = Vec::new();

    if tokens.len() <= 1 && !trailing {
        for (value, hint) in command_candidates() {
            if value.starts_with(prefix) {
                candidates.push(Candidate { value, hint });
            }
        }
    } else {
        match tokens.first().map(String::as_str) {
            Some("goto") | Some("teleport") => {
                candidates = entity_candidates(
                    state
                        .world
                        .locations
                        .iter()
                        .map(|location| (location.id, location.name.clone())),
                    prefix,
                )
            }
            Some("quest") => {
                if tokens.len() <= 2 {
                    for value in ["list", "complete", "reset"] {
                        if value.starts_with(prefix) {
                            candidates.push(Candidate {
                                value: value.into(),
                                hint: "quest subcommand".into(),
                            });
                        }
                    }
                } else if matches!(
                    tokens.get(1).map(String::as_str),
                    Some("complete" | "reset")
                ) {
                    candidates = entity_candidates(
                        state
                            .quests
                            .iter()
                            .map(|quest| (quest.id, quest.title.clone())),
                        prefix,
                    );
                }
            }
            Some("faction") => {
                if tokens.len() <= 2 && "set".starts_with(prefix) {
                    candidates.push(Candidate {
                        value: "set".into(),
                        hint: "set reputation".into(),
                    });
                } else if tokens.get(1).map(String::as_str) == Some("set") {
                    candidates = entity_candidates(
                        state
                            .factions
                            .iter()
                            .map(|faction| (faction.id, faction.name.clone())),
                        prefix,
                    );
                }
            }
            Some("npc") | Some("npcs") => {
                candidates = entity_candidates(
                    state.npcs.iter().map(|npc| (npc.id, npc.display_name())),
                    prefix,
                )
            }
            Some("give") | Some("remove") => {
                candidates = entity_candidates(
                    state
                        .character
                        .inventory
                        .iter()
                        .map(|item| (item.id, item.name.clone())),
                    prefix,
                )
            }
            _ => {}
        }
    }

    let candidates_changed = console.candidates.len() != candidates.len()
        || console
            .candidates
            .iter()
            .zip(candidates.iter())
            .any(|(left, right)| left.value != right.value || left.hint != right.hint);

    console.candidates = candidates;
    if console.candidates.is_empty() {
        console.autocomplete = false;
        console.completion_scroll = 0;
        return;
    }
    if console.selected >= console.candidates.len() {
        console.selected = 0;
    }
    if candidates_changed {
        console.completion_scroll = console
            .completion_scroll
            .min(console.candidates.len().saturating_sub(1));
    }
    keep_completion_selection_visible(console, 8);
}

fn entity_candidates<I>(entities: I, prefix: &str) -> Vec<Candidate>
where
    I: IntoIterator<Item = (EntityId, String)>,
{
    entities
        .into_iter()
        .filter_map(|(id, hint)| {
            let value = id.to_string();
            value
                .starts_with(prefix)
                .then_some(Candidate { value, hint })
        })
        .collect()
}

fn tokenize(input: &str) -> (Vec<String>, bool) {
    (
        input.split_whitespace().map(ToOwned::to_owned).collect(),
        input.chars().last().is_some_and(char::is_whitespace),
    )
}

fn command_candidates() -> Vec<(String, String)> {
    [
        ("help", "show commands"),
        ("clear", "clear output"),
        ("status", "character/world state"),
        ("where", "filesystem content paths"),
        ("mods", "external mod directories"),
        ("content", "loaded content diagnostics"),
        ("locations", "list location IDs"),
        ("goto", "move to location ID"),
        ("teleport", "goto alias"),
        ("npcs", "list NPC IDs"),
        ("quests", "list quests"),
        ("quest", "quest commands"),
        ("factions", "list factions"),
        ("faction", "faction commands"),
        ("inventory", "list inventory"),
        ("give", "clone an item"),
        ("remove", "remove an item"),
        ("heal", "restore HP"),
        ("damage", "deal damage"),
        ("kill", "set alive=false"),
        ("revive", "set alive=true"),
        ("xp", "set/add experience"),
        ("level", "set level"),
        ("attr", "set attribute"),
        ("condition", "add/clear condition"),
        ("time", "set/add time points"),
        ("day", "set day"),
        ("history", "recent history"),
        ("reload", "reload campaign content"),
        ("save", "save current game"),
        ("exit", "close console"),
    ]
    .into_iter()
    .map(|(value, hint)| (value.into(), hint.into()))
    .collect()
}

pub(super) fn keep_completion_selection_visible(console: &mut ConsoleState, visible: usize) {
    if console.candidates.is_empty() || visible == 0 {
        console.completion_scroll = 0;
        return;
    }
    let max_start = console.candidates.len().saturating_sub(visible);
    if console.selected < console.completion_scroll {
        console.completion_scroll = console.selected;
    } else if console.selected >= console.completion_scroll + visible {
        console.completion_scroll = console.selected + 1 - visible;
    }
    console.completion_scroll = console.completion_scroll.min(max_start);
}

pub(super) fn select_previous(console: &mut ConsoleState) {
    if !console.candidates.is_empty() {
        console.selected = console
            .selected
            .checked_sub(1)
            .unwrap_or(console.candidates.len() - 1);
        keep_completion_selection_visible(console, 8);
    }
}

pub(super) fn select_next(console: &mut ConsoleState) {
    if !console.candidates.is_empty() {
        console.selected = (console.selected + 1) % console.candidates.len();
        keep_completion_selection_visible(console, 8);
    }
}

pub(super) fn cancel_completion(console: &mut ConsoleState) {
    console.autocomplete = false;
    console.candidates.clear();
    console.completion_scroll = 0;
}

pub(super) fn accept_completion(console: &mut ConsoleState) {
    let Some(candidate) = console.candidates.get(console.selected).cloned() else {
        return;
    };
    let (mut tokens, trailing) = tokenize(&console.input);
    if trailing {
        tokens.push(candidate.value);
    } else if let Some(last) = tokens.last_mut() {
        *last = candidate.value;
    } else {
        tokens.push(candidate.value);
    }
    console.input = tokens.join(" ");
    cancel_completion(console);
}

pub(super) fn choose_main_menu(
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
        terminal.draw(|frame| {
            let popup = centered_rect(64, 70, frame.area());
            frame.render_widget(Clear, popup);
            let block = Block::default().title(title).borders(Borders::ALL);
            let inner = block.inner(popup);
            frame.render_widget(block, popup);

            let visible_rows = inner.height.saturating_sub(2) as usize;
            let visible_rows = visible_rows.max(1);
            let mut start = selected.saturating_sub(visible_rows / 2);
            start = start.min(options.len().saturating_sub(visible_rows));
            let end = (start + visible_rows).min(options.len());

            let mut lines = vec![
                Line::from("↑↓ / jk  Enter: choose  Esc: back"),
                Line::from(""),
            ];
            if start > 0 {
                lines.push(Line::from("⋯ more above ⋯"));
            }
            for (index, option) in options.iter().enumerate().take(end).skip(start) {
                let marker = if index == selected { '▶' } else { ' ' };
                lines.push(Line::from(format!("{marker} {}. {option}", index + 1)));
            }
            if end < options.len() {
                lines.push(Line::from("⋯ more below ⋯"));
            }
            frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
        })?;

        match ui::read_key()? {
            KeyCode::Up | KeyCode::Char('k') => {
                selected = selected
                    .checked_sub(1)
                    .unwrap_or(options.len().saturating_sub(1));
            }
            KeyCode::Down | KeyCode::Char('j') => selected = (selected + 1) % options.len(),
            KeyCode::Home => selected = 0,
            KeyCode::End => selected = options.len().saturating_sub(1),
            KeyCode::Enter => return Ok(Some(selected)),
            KeyCode::Esc => return Ok(None),
            KeyCode::Char('/') => super::run_console_session(state, save_path)?,
            _ => {}
        }
    }
}

pub(super) fn draw_console(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    console: &ConsoleState,
) -> io::Result<()> {
    terminal.draw(|frame| {
        let area = centered_rect(92, 82, frame.area());
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title("Developer Console")
            .borders(Borders::ALL);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(4), Constraint::Length(3)].as_ref())
            .split(inner);

        let lines = console
            .output
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect::<Vec<_>>();
        let max_scroll = wrapped_line_count(&lines, chunks[0].width as usize)
            .saturating_sub(chunks[0].height as usize);
        let scroll = match console.scroll {
            ScrollPosition::Follow => 0,
            ScrollPosition::Offset(offset) => offset.min(max_scroll),
            ScrollPosition::Home => max_scroll,
        };
        frame.render_widget(
            Paragraph::new(lines)
                .wrap(Wrap { trim: false })
                .scroll((scroll.min(u16::MAX as usize) as u16, 0)),
            chunks[0],
        );

        let input = Paragraph::new(Line::from(format!("> {}", console.input)))
            .block(Block::default().borders(Borders::ALL).title("Input"));
        frame.render_widget(input, chunks[1]);

        if console.autocomplete && !console.candidates.is_empty() {
            let visible = console.candidates.len().min(8);
            let popup_height = visible as u16 + 2;
            let popup = Rect {
                x: chunks[1].x,
                y: chunks[1].y.saturating_sub(popup_height),
                width: chunks[1].width,
                height: popup_height,
            };
            frame.render_widget(Clear, popup);
            let start = console
                .completion_scroll
                .min(console.candidates.len().saturating_sub(visible));
            let lines = console
                .candidates
                .iter()
                .enumerate()
                .skip(start)
                .take(visible)
                .map(|(index, candidate)| {
                    let marker = if index == console.selected {
                        '▶'
                    } else {
                        ' '
                    };
                    Line::from(format!(
                        "{marker} {}  — {}",
                        candidate.value, candidate.hint
                    ))
                })
                .collect::<Vec<_>>();
            frame.render_widget(
                Paragraph::new(lines)
                    .block(Block::default().borders(Borders::ALL).title("Completion")),
                popup,
            );
        }
    })?;
    Ok(())
}

fn wrapped_line_count(lines: &[Line<'_>], width: usize) -> usize {
    if width == 0 {
        return 0;
    }
    lines
        .iter()
        .map(|line| line.width().div_ceil(width).max(1))
        .sum()
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
