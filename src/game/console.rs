use crate::content::load_campaign_content_report;
use crate::game::{presentation, world};
use crate::model::{Condition, EntityId, GameState};
use crate::persistence::save_game;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Terminal;
use std::env;
use std::io::{self, Stdout};
use std::path::Path;

#[derive(Clone, Debug)]
struct Candidate {
    value: String,
    hint: String,
}

#[derive(Default)]
struct ConsoleState {
    output: Vec<String>,
    input: String,
    history: Vec<String>,
    history_index: Option<usize>,
    scroll: usize,
    candidates: Vec<Candidate>,
    candidate_index: usize,
    autocomplete: bool,
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
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.hide_cursor()?;

    loop {
        draw_main_menu(&mut terminal, title, options, selected)?;
        match read_key()? {
            KeyCode::Up | KeyCode::Char('k') => {
                selected = if selected == 0 {
                    options.len() - 1
                } else {
                    selected - 1
                };
            }
            KeyCode::Down | KeyCode::Char('j') => {
                selected = (selected + 1) % options.len();
            }
            KeyCode::Home => selected = 0,
            KeyCode::End => selected = options.len() - 1,
            KeyCode::Enter => {
                terminal.show_cursor()?;
                return Ok(Some(selected));
            }
            KeyCode::Esc => {
                terminal.show_cursor()?;
                return Ok(None);
            }
            KeyCode::Char('/') => {
                open_console(state, save_path)?;
                presentation::render_state(state);
                presentation::maybe_run_location_scene(state)?;
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                if let Some(digit) = c.to_digit(10) {
                    let index = digit as usize;
                    if index >= 1 && index <= options.len() {
                        terminal.show_cursor()?;
                        return Ok(Some(index - 1));
                    }
                }
            }
            _ => {}
        }
    }
}

fn draw_main_menu(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    title: &str,
    options: &[String],
    selected: usize,
) -> io::Result<()> {
    terminal.draw(|frame| {
        let area = frame.area();
        let popup = centered_rect(62, (options.len() as u16 * 2 + 10).min(70), area);
        frame.render_widget(Clear, popup);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL);
        let inner = block.inner(popup);
        frame.render_widget(block, popup);

        let mut lines = vec![Line::from("↑ ↓ / j k   Enter: choose   /: console   Esc: back"), Line::from("")];
        for (index, option) in options.iter().enumerate() {
            let marker = if index == selected { '▶' } else { ' ' };
            lines.push(Line::from(format!("{marker} {}. {}", index + 1, option)));
            if index + 1 != options.len() {
                lines.push(Line::from(""));
            }
        }
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    })?;
    Ok(())
}

fn open_console(state: &mut GameState, save_path: &Path) -> io::Result<()> {
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.hide_cursor()?;
    let mut console = ConsoleState::default();
    console.output.push("Ashen Chronicle developer console".to_string());
    console.output.push("Type 'help' for commands. Tab enables completion.".to_string());

    loop {
        refresh_candidates(&mut console, state);
        draw_console(&mut terminal, &console)?;

        match event::read()? {
            Event::Key(key) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) && matches!(key.code, KeyCode::Char('c')) {
                    break;
                }
                if console.autocomplete {
                    match key.code {
                        KeyCode::Up => {
                            if !console.candidates.is_empty() {
                                console.candidate_index = console
                                    .candidate_index
                                    .checked_sub(1)
                                    .unwrap_or(console.candidates.len() - 1);
                            }
                        }
                        KeyCode::Down => {
                            if !console.candidates.is_empty() {
                                console.candidate_index =
                                    (console.candidate_index + 1) % console.candidates.len();
                            }
                        }
                        KeyCode::Enter => {
                            apply_candidate(&mut console);
                        }
                        KeyCode::Esc => {
                            console.autocomplete = false;
                            console.candidates.clear();
                        }
                        KeyCode::Tab => {
                            refresh_candidates(&mut console, state);
                            if console.candidates.is_empty() {
                                console.autocomplete = false;
                            }
                        }
                        _ => {
                            console.autocomplete = false;
                            handle_editing_key(&mut console, key.code);
                        }
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Esc => break,
                    KeyCode::Enter => {
                        execute_line(state, save_path, &mut console)?;
                    }
                    KeyCode::Tab => {
                        refresh_candidates(&mut console, state);
                        if !console.candidates.is_empty() {
                            console.autocomplete = true;
                            console.candidate_index = 0;
                        }
                    }
                    KeyCode::Up => history_up(&mut console),
                    KeyCode::Down => history_down(&mut console),
                    KeyCode::PageUp => {
                        console.scroll = console.scroll.saturating_add(6);
                    }
                    KeyCode::PageDown => {
                        console.scroll = console.scroll.saturating_sub(6);
                    }
                    KeyCode::Char(c) if !c.is_control() => {
                        console.input.push(c);
                        console.history_index = None;
                        console.scroll = 0;
                    }
                    _ => handle_editing_key(&mut console, key.code),
                }
            }
            _ => {}
        }
    }

    terminal.show_cursor()?;
    Ok(())
}

fn handle_editing_key(console: &mut ConsoleState, code: KeyCode) {
    match code {
        KeyCode::Backspace => {
            console.input.pop();
            console.history_index = None;
        }
        KeyCode::Delete => {}
        KeyCode::Home => {}
        KeyCode::End => {}
        _ => {}
    }
}

fn history_up(console: &mut ConsoleState) {
    if console.history.is_empty() {
        return;
    }
    let next = match console.history_index {
        None => console.history.len() - 1,
        Some(index) => index.saturating_sub(1),
    };
    console.history_index = Some(next);
    console.input = console.history[next].clone();
}

fn history_down(console: &mut ConsoleState) {
    let Some(index) = console.history_index else {
        return;
    };
    if index + 1 >= console.history.len() {
        console.history_index = None;
        console.input.clear();
        return;
    }
    let next = index + 1;
    console.history_index = Some(next);
    console.input = console.history[next].clone();
}

fn refresh_candidates(console: &mut ConsoleState, state: &GameState) {
    let (tokens, trailing_space) = split_input(&console.input);
    let (replace_index, prefix) = if trailing_space {
        (tokens.len(), "")
    } else if tokens.is_empty() {
        (0, "")
    } else {
        (tokens.len() - 1, tokens.last().map(String::as_str).unwrap_or(""))
    };

    let mut candidates = Vec::new();
    let commands = [
        ("help", "show commands"),
        ("clear", "clear console output"),
        ("status", "show current game state"),
        ("where", "show content search paths"),
        ("mods", "show discovered/loaded mods"),
        ("content", "show merged content counts"),
        ("locations", "list location entity IDs"),
        ("goto", "move to a location by ID"),
        ("teleport", "alias for goto"),
        ("npcs", "list NPC entity IDs"),
        ("quests", "list quest entity IDs"),
        ("quest", "quest debug commands"),
        ("factions", "list faction entity IDs"),
        ("faction", "faction debug commands"),
        ("inventory", "list item entity IDs"),
        ("give", "duplicate an existing item by ID"),
        ("remove", "remove an item by ID"),
        ("heal", "restore HP"),
        ("damage", "deal direct damage"),
        ("kill", "kill the character"),
        ("revive", "revive the character"),
        ("xp", "set/add experience"),
        ("level", "set character level"),
        ("attr", "set a character attribute"),
        ("condition", "add/clear a condition"),
        ("time", "set/add time points"),
        ("day", "set the world day"),
        ("history", "show history entries"),
        ("reload", "reload external campaign content"),
        ("save", "save the current game"),
        ("exit", "close the console"),
    ];

    if tokens.len() <= 1 && !trailing_space {
        for (command, hint) in commands {
            if command.starts_with(prefix) {
                candidates.push(Candidate {
                    value: command.to_string(),
                    hint: hint.to_string(),
                });
            }
        }
    } else if tokens.first().map(String::as_str) == Some("goto")
        || tokens.first().map(String::as_str) == Some("teleport")
    {
        candidates = location_candidates(state, prefix);
    } else if tokens.first().map(String::as_str) == Some("quest") {
        if tokens.len() <= 2 {
            for command in ["list", "complete", "reset"] {
                if command.starts_with(prefix) {
                    candidates.push(Candidate {
                        value: command.to_string(),
                        hint: "quest subcommand".to_string(),
                    });
                }
            }
        } else if matches!(tokens.get(1).map(String::as_str), Some("complete" | "reset")) {
            candidates = quest_candidates(state, prefix);
        }
    } else if tokens.first().map(String::as_str) == Some("faction") {
        if tokens.len() <= 2 {
            if "set".starts_with(prefix) {
                candidates.push(Candidate {
                    value: "set".to_string(),
                    hint: "set faction reputation".to_string(),
                });
            }
        } else if tokens.get(1).map(String::as_str) == Some("set") {
            candidates = faction_candidates(state, prefix);
        }
    } else if matches!(tokens.first().map(String::as_str), Some("npcs")) {
        if !trailing_space || replace_index > 0 {
            candidates = npc_candidates(state, prefix);
        }
    } else if matches!(tokens.first().map(String::as_str), Some("remove" | "give")) {
        candidates = item_candidates(state, prefix);
    }

    console.candidates = candidates;
    if console.candidates.is_empty() {
        console.autocomplete = false;
    } else if console.candidate_index >= console.candidates.len() {
        console.candidate_index = 0;
    }
}

fn split_input(input: &str) -> (Vec<String>, bool) {
    let trailing_space = input.chars().last().is_some_and(char::is_whitespace);
    let tokens = input
        .split_whitespace()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    (tokens, trailing_space)
}

fn location_candidates(state: &GameState, prefix: &str) -> Vec<Candidate> {
    state
        .world
        .locations
        .iter()
        .filter(|location| location.id.to_string().starts_with(prefix))
        .map(|location| Candidate {
            value: location.id.to_string(),
            hint: location.name.clone(),
        })
        .collect()
}

fn npc_candidates(state: &GameState, prefix: &str) -> Vec<Candidate> {
    state
        .npcs
        .iter()
        .filter(|npc| npc.id.to_string().starts_with(prefix))
        .map(|npc| Candidate {
            value: npc.id.to_string(),
            hint: npc.display_name(),
        })
        .collect()
}

fn quest_candidates(state: &GameState, prefix: &str) -> Vec<Candidate> {
    state
        .quests
        .iter()
        .filter(|quest| quest.id.to_string().starts_with(prefix))
        .map(|quest| Candidate {
            value: quest.id.to_string(),
            hint: quest.title.clone(),
        })
        .collect()
}

fn faction_candidates(state: &GameState, prefix: &str) -> Vec<Candidate> {
    state
        .factions
        .iter()
        .filter(|faction| faction.id.to_string().starts_with(prefix))
        .map(|faction| Candidate {
            value: faction.id.to_string(),
            hint: faction.name.clone(),
        })
        .collect()
}

fn item_candidates(state: &GameState, prefix: &str) -> Vec<Candidate> {
    state
        .character
        .inventory
        .iter()
        .filter(|item| item.id.to_string().starts_with(prefix))
        .map(|item| Candidate {
            value: item.id.to_string(),
            hint: item.name.clone(),
        })
        .collect()
}

fn apply_candidate(console: &mut ConsoleState) {
    let Some(candidate) = console.candidates.get(console.candidate_index).cloned() else {
        return;
    };
    let (mut tokens, trailing_space) = split_input(&console.input);
    if trailing_space {
        tokens.push(candidate.value);
    } else if let Some(last) = tokens.last_mut() {
        *last = candidate.value;
    } else {
        tokens.push(candidate.value);
    }
    console.input = tokens.join(" ");
    console.autocomplete = false;
    console.candidates.clear();
    console.history_index = None;
}

fn execute_line(
    state: &mut GameState,
    save_path: &Path,
    console: &mut ConsoleState,
) -> io::Result<()> {
    let line = console.input.trim().to_string();
    if line.is_empty() {
        return Ok(());
    }
    console.output.push(format!("> {line}"));
    if console.history.last() != Some(&line) {
        console.history.push(line.clone());
    }
    console.history_index = None;
    console.input.clear();
    console.scroll = 0;

    let parts = line.split_whitespace().collect::<Vec<_>>();
    let command = parts.first().copied().unwrap_or("");
    let args = &parts[1..];

    match command {
        "help" => command_help(console, args),
        "clear" => console.output.clear(),
        "status" => command_status(state, console),
        "where" => command_where(console),
        "mods" => command_mods(console),
        "content" => command_content(state, console),
        "locations" => command_locations(state, console),
        "goto" | "teleport" => command_goto(state, console, args),
        "npcs" => command_npcs(state, console, args),
        "quests" => command_quests(state, console),
        "quest" => command_quest(state, console, args),
        "factions" => command_factions(state, console),
        "faction" => command_faction(state, console, args),
        "inventory" => command_inventory(state, console),
        "give" => command_give(state, console, args),
        "remove" => command_remove(state, console, args),
        "heal" => command_heal(state, console, args),
        "damage" => command_damage(state, console, args),
        "kill" => command_kill(state, console),
        "revive" => command_revive(state, console),
        "xp" => command_xp(state, console, args),
        "level" => command_level(state, console, args),
        "attr" => command_attr(state, console, args),
        "condition" => command_condition(state, console, args),
        "time" => command_time(state, console, args),
        "day" => command_day(state, console, args),
        "history" => command_history(state, console, args),
        "reload" => command_reload(state, console),
        "save" => command_save(state, save_path, console)?,
        "exit" | "quit" => {
            console.output.push("Console closed.".to_string());
            console.autocomplete = false;
            return Ok(());
        }
        _ => console.output.push(format!("Unknown command '{command}'. Try 'help'.")),
    }
    Ok(())
}

fn command_help(console: &mut ConsoleState, args: &[&str]) {
    let help = match args.first().copied() {
        Some("goto") | Some("teleport") => "goto <location-id>     move anywhere by runtime location ID",
        Some("quest") => "quest list | quest complete <id> | quest reset <id>",
        Some("faction") => "faction set <id> <reputation>",
        Some("xp") => "xp add <amount> | xp set <amount>",
        Some("condition") => "condition add <name> <remaining> <penalty> <bonus> | condition clear",
        _ => "Commands: help clear status where mods content locations goto teleport npcs quests quest factions faction inventory give remove heal damage kill revive xp level attr condition time day history reload save exit",
    };
    console.output.push(help.to_string());
}

fn command_status(state: &GameState, console: &mut ConsoleState) {
    let location = state.world.location_by_id(state.character.location_id);
    console.output.extend([
        format!("Character: {} (id {})", state.character.display_name(), state.character.id),
        format!("Alive: {}  HP: {}/{}", state.character.alive, state.character.hp, state.character.max_hp),
        format!("Level: {}  XP: {}", state.character.level, state.character.experience),
        format!("World: {} (id {})", state.world.name, state.world.id),
        format!("Day: {}  Time points: {}", state.world.day, state.world.time_points),
        format!(
            "Location: {} (id {})",
            location.map(|entry| entry.name.as_str()).unwrap_or("<unknown>"),
            state.character.location_id
        ),
    ]);
}

fn command_where(console: &mut ConsoleState) {
    let cwd = env::current_dir().ok();
    let exe = env::current_exe().ok();
    console.output.push(format!("cwd: {}", cwd.as_deref().map(Path::display).map(|v| v.to_string()).unwrap_or_else(|| "<unavailable>".to_string())));
    console.output.push(format!("exe: {}", exe.as_deref().map(Path::display).map(|v| v.to_string()).unwrap_or_else(|| "<unavailable>".to_string())));
    let mut candidates = Vec::new();
    if let Some(cwd) = cwd {
        candidates.push(cwd.join("data"));
    }
    if let Some(exe) = exe {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("data"));
            if let Some(parent) = dir.parent() {
                candidates.push(parent.join("data"));
            }
        }
    }
    for path in candidates {
        console.output.push(format!("data candidate: {} [{}]", path.display(), if path.is_dir() { "dir" } else if path.exists() { "exists" } else { "missing" }));
        console.output.push(format!("  base: {}", path.join("base_content.json").display()));
        console.output.push(format!("  mods: {}", path.join("mods").display()));
    }
}

fn command_mods(console: &mut ConsoleState) {
    let report = load_campaign_content_report();
    if report.loaded_mods.is_empty() {
        console.output.push("Loaded mods: none".to_string());
    } else {
        console.output.push(format!("Loaded mods: {}", report.loaded_mods.len()));
        for manifest in report.loaded_mods {
            console.output.push(format!("  {} — {} v{}", manifest.id, manifest.name, manifest.version));
        }
    }
    if report.warnings.is_empty() {
        console.output.push("Warnings: none".to_string());
    } else {
        console.output.push(format!("Warnings: {}", report.warnings.len()));
        for warning in report.warnings {
            console.output.push(format!("  ! {warning}"));
        }
    }
}

fn command_content(state: &GameState, console: &mut ConsoleState) {
    let Some(content) = state.campaign_content.as_ref() else {
        console.output.push("Campaign content is not loaded.".to_string());
        return;
    };
    console.output.extend([
        format!("Locations: {}", content.world.locations.len()),
        format!("Factions: {}", content.factions.len()),
        format!("NPCs: {}", content.npcs.len()),
        format!("Quests: {}", content.quests.len()),
        format!("Encounters: {}", content.encounters.len()),
        format!("Events: {}", content.events.len()),
        format!("Atmospheres: {}", content.atmospheres.len()),
        format!("Item visuals: {}", content.item_visuals.len()),
    ]);
}

fn command_locations(state: &GameState, console: &mut ConsoleState) {
    for location in &state.world.locations {
        let marker = if location.id == state.character.location_id { '*' } else { ' ' };
        console.output.push(format!("{marker} {} — {}", location.id, location.name));
    }
}

fn command_goto(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let Some(id) = args.first().and_then(|value| value.parse::<EntityId>().ok()) else {
        console.output.push("Usage: goto <location-id>".to_string());
        return;
    };
    let Some(location) = state.world.location_by_id(id) else {
        console.output.push(format!("Unknown location id {id}."));
        return;
    };
    state.character.location_id = id;
    state.last_announced_location_id = None;
    console.output.push(format!("Moved to {} ({id}).", location.name));
}

fn command_npcs(state: &GameState, console: &mut ConsoleState, args: &[&str]) {
    let location_filter = args.first().and_then(|value| value.parse::<EntityId>().ok());
    for npc in &state.npcs {
        if location_filter.is_some_and(|id| npc.location_id != id) {
            continue;
        }
        let location_name = state
            .world
            .location_by_id(npc.location_id)
            .map(|location| location.name.as_str())
            .unwrap_or("<unknown>");
        console.output.push(format!("{} — {} — {}", npc.id, npc.display_name(), location_name));
    }
}

fn command_quests(state: &GameState, console: &mut ConsoleState) {
    for quest in &state.quests {
        console.output.push(format!(
            "{} — {} [{}]",
            quest.id,
            quest.title,
            if quest.completed { "complete" } else if quest.offered { "offered" } else { "new" }
        ));
    }
}

fn command_quest(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    match args {
        ["list"] | [] => command_quests(state, console),
        [action, id] if matches!(*action, "complete" | "reset") => {
            let Some(id) = id.parse::<EntityId>().ok() else {
                console.output.push("Quest IDs are numeric runtime entity IDs.".to_string());
                return;
            };
            let Some(quest) = state.quests.iter_mut().find(|quest| quest.id == id) else {
                console.output.push(format!("Unknown quest id {id}."));
                return;
            };
            if *action == "complete" {
                quest.completed = true;
                quest.offered = true;
                if !state.world.completed_quest_ids.contains(&quest.content_id) {
                    state.world.completed_quest_ids.push(quest.content_id.clone());
                }
                console.output.push(format!("Completed quest {}.", quest.title));
            } else {
                quest.completed = false;
                quest.reward_claimed = false;
                state.world.completed_quest_ids.retain(|completed| completed != &quest.content_id);
                console.output.push(format!("Reset quest {}.", quest.title));
            }
        }
        _ => console.output.push("Usage: quest list | quest complete <id> | quest reset <id>".to_string()),
    }
}

fn command_factions(state: &GameState, console: &mut ConsoleState) {
    for faction in &state.factions {
        console.output.push(format!("{} — {} ({:+})", faction.id, faction.name, faction.reputation));
    }
}

fn command_faction(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    if let ["set", id, reputation] = args {
        let Some(id) = id.parse::<EntityId>().ok() else {
            console.output.push("Faction IDs are numeric runtime entity IDs.".to_string());
            return;
        };
        let Some(reputation) = reputation.parse::<i32>().ok() else {
            console.output.push("Reputation must be an integer.".to_string());
            return;
        };
        let Some(faction) = state.factions.iter_mut().find(|faction| faction.id == id) else {
            console.output.push(format!("Unknown faction id {id}."));
            return;
        };
        faction.reputation = reputation;
        console.output.push(format!("{} reputation set to {:+}.", faction.name, reputation));
        return;
    }
    console.output.push("Usage: faction set <id> <reputation>".to_string());
}

fn command_inventory(state: &GameState, console: &mut ConsoleState) {
    if state.character.inventory.is_empty() {
        console.output.push("Inventory: empty".to_string());
        return;
    }
    for item in &state.character.inventory {
        console.output.push(format!("{} — {}", item.id, item.name));
    }
}

fn command_give(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let Some(source_id) = args.first().and_then(|value| value.parse::<EntityId>().ok()) else {
        console.output.push("Usage: give <inventory-item-id> [count]".to_string());
        return;
    };
    let count = args
        .get(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1)
        .clamp(1, 100);
    let Some(source) = state.character.inventory.iter().find(|item| item.id == source_id).cloned() else {
        console.output.push(format!("Unknown inventory item id {source_id}."));
        return;
    };
    for _ in 0..count {
        let id = state.world.allocate_id();
        state.character.inventory.push(crate::model::Item {
            id,
            name: source.name.clone(),
            description: source.description.clone(),
        });
    }
    console.output.push(format!("Added {} copy/copies of {}.", count, source.name));
}

fn command_remove(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let Some(id) = args.first().and_then(|value| value.parse::<EntityId>().ok()) else {
        console.output.push("Usage: remove <inventory-item-id>".to_string());
        return;
    };
    let before = state.character.inventory.len();
    state.character.inventory.retain(|item| item.id != id);
    if state.character.inventory.len() == before {
        console.output.push(format!("Unknown inventory item id {id}."));
    } else {
        console.output.push(format!("Removed item {id}."));
    }
}

fn command_heal(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let amount = args
        .first()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(state.character.max_hp);
    state.character.heal(amount.max(0));
    console.output.push(format!("HP: {}/{}", state.character.hp, state.character.max_hp));
}

fn command_damage(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let amount = args.first().and_then(|value| value.parse::<i32>().ok()).unwrap_or(1).max(0);
    state.character.hp = (state.character.hp - amount).max(0);
    if state.character.hp == 0 {
        state.character.alive = false;
    }
    console.output.push(format!("HP: {}/{}  alive={}", state.character.hp, state.character.max_hp, state.character.alive));
}

fn command_kill(state: &mut GameState, console: &mut ConsoleState) {
    state.character.hp = 0;
    state.character.alive = false;
    console.output.push("Character killed.".to_string());
}

fn command_revive(state: &mut GameState, console: &mut ConsoleState) {
    state.character.alive = true;
    state.character.hp = state.character.max_hp.max(1);
    console.output.push("Character revived at full HP.".to_string());
}

fn command_xp(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let (mode, amount) = match args {
        ["add", amount] => ("add", amount.parse::<u32>().ok()),
        ["set", amount] => ("set", amount.parse::<u32>().ok()),
        [amount] => ("add", amount.parse::<u32>().ok()),
        _ => ("add", None),
    };
    let Some(amount) = amount else {
        console.output.push("Usage: xp add <amount> | xp set <amount>".to_string());
        return;
    };
    if mode == "set" {
        state.character.experience = amount;
    } else {
        state.character.experience = state.character.experience.saturating_add(amount);
    }
    console.output.push(format!("XP: {} (level {}).", state.character.experience, state.character.level));
}

fn command_level(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let value = match args {
        ["set", value] | [value] => value.parse::<u32>().ok(),
        _ => None,
    };
    let Some(value) = value else {
        console.output.push("Usage: level set <level>".to_string());
        return;
    };
    state.character.level = value.max(1);
    console.output.push(format!("Level set to {}.", state.character.level));
}

fn command_attr(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let [attribute, value] = args else {
        console.output.push("Usage: attr set <might|insight|endurance> <value>".to_string());
        return;
    };
    let Some(value) = value.parse::<i32>().ok() else {
        console.output.push("Attribute value must be an integer.".to_string());
        return;
    };
    match *attribute {
        "might" => state.character.attributes.might = value,
        "insight" => state.character.attributes.insight = value,
        "endurance" => state.character.attributes.endurance = value,
        _ => {
            console.output.push("Attribute must be might, insight, or endurance.".to_string());
            return;
        }
    }
    console.output.push(format!("{attribute} set to {value}."));
}

fn command_condition(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    if args.first().copied() == Some("clear") {
        state.character.conditions.clear();
        console.output.push("Conditions cleared.".to_string());
        return;
    }
    if args.len() == 5 && args[0] == "add" {
        let Some(remaining) = args[2].parse::<u32>().ok() else {
            console.output.push("remaining must be an integer.".to_string());
            return;
        };
        let Some(penalty) = args[3].parse::<i32>().ok() else {
            console.output.push("penalty must be an integer.".to_string());
            return;
        };
        let Some(bonus) = args[4].parse::<i32>().ok() else {
            console.output.push("bonus must be an integer.".to_string());
            return;
        };
        state.character.conditions.push(Condition {
            name: args[1].to_string(),
            remaining,
            penalty,
            bonus,
        });
        console.output.push(format!("Added condition {}.", args[1]));
        return;
    }
    console.output.push("Usage: condition add <name> <remaining> <penalty> <bonus> | condition clear".to_string());
}

fn command_time(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let (mode, value) = match args {
        ["set", value] | [value] => ("set", value.parse::<u32>().ok()),
        ["add", value] => ("add", value.parse::<u32>().ok()),
        _ => ("set", None),
    };
    let Some(value) = value else {
        console.output.push("Usage: time set <0..3> | time add <points>".to_string());
        return;
    };
    state.world.time_points = if mode == "set" {
        value.min(3)
    } else {
        state.world.time_points.saturating_add(value).min(3)
    };
    console.output.push(format!("Time points: {}.", state.world.time_points));
}

fn command_day(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let value = match args {
        ["set", value] | [value] => value.parse::<u32>().ok(),
        _ => None,
    };
    let Some(value) = value else {
        console.output.push("Usage: day set <day>".to_string());
        return;
    };
    state.world.day = value.max(1);
    console.output.push(format!("Day: {}.", state.world.day));
}

fn command_history(state: &GameState, console: &mut ConsoleState, args: &[&str]) {
    let count = args.first().and_then(|value| value.parse::<usize>().ok()).unwrap_or(10).min(100);
    for entry in state.world.history.iter().rev().take(count).rev() {
        console.output.push(format!("[{}] {}", entry.turn, entry.text));
    }
}

fn command_reload(state: &mut GameState, console: &mut ConsoleState) {
    let report = load_campaign_content_report();
    let loaded = report.loaded_mods.len();
    let warnings = report.warnings.len();
    state.campaign_content = Some(report.content);
    world::bootstrap_campaign_content(state);
    state.last_announced_location_id = None;
    console.output.push(format!("Reloaded campaign content: {loaded} mod(s), {warnings} warning(s)."));
    if warnings > 0 {
        console.output.push(format!("Run 'mods' to inspect the warnings."));
    }
}

fn command_save(state: &GameState, save_path: &Path, console: &mut ConsoleState) -> io::Result<()> {
    match save_game(save_path, state) {
        Ok(()) => console.output.push(format!("Saved to {}.", save_path.display())),
        Err(err) => console.output.push(format!("Save failed: {err}")),
    }
    Ok(())
}

fn draw_console(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    console: &ConsoleState,
) -> io::Result<()> {
    terminal.draw(|frame| {
        let area = frame.area();
        let popup = centered_rect(90, 86, area);
        frame.render_widget(Clear, popup);
        let block = Block::default()
            .title("DEV CONSOLE")
            .borders(Borders::ALL);
        let inner = block.inner(popup);
        frame.render_widget(block, popup);

        let candidate_height = if console.autocomplete && !console.candidates.is_empty() {
            (console.candidates.len() as u16 + 2).min(inner.height.saturating_sub(6).max(3))
        } else {
            0
        };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(candidate_height),
                Constraint::Length(3),
            ])
            .split(inner);

        let output_height = chunks[0].height as usize;
        let start = console
            .output
            .len()
            .saturating_sub(output_height)
            .saturating_sub(console.scroll);
        let end = (start + output_height).min(console.output.len());
        let output = console.output[start..end]
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let mut output_text = output.join("\n");
        if console.scroll > 0 {
            output_text = format!("↑ older output (scroll {}) ↑\n{output_text}", console.scroll);
        }
        frame.render_widget(
            Paragraph::new(output_text)
                .block(Block::default().borders(Borders::ALL).title("output"))
                .wrap(Wrap { trim: false }),
            chunks[0],
        );

        if candidate_height > 0 {
            let lines = console
                .candidates
                .iter()
                .enumerate()
                .map(|(index, candidate)| {
                    let marker = if index == console.candidate_index { '▶' } else { ' ' };
                    Line::from(format!("{marker} {:<18} {}", candidate.value, candidate.hint))
                })
                .collect::<Vec<_>>();
            frame.render_widget(
                Paragraph::new(lines)
                    .block(Block::default().borders(Borders::ALL).title("completion"))
                    .wrap(Wrap { trim: false }),
                chunks[1],
            );
        }

        frame.render_widget(
            Paragraph::new(format!("/ {}", console.input))
                .block(Block::default().borders(Borders::ALL).title("input")),
            chunks[2],
        );
        frame.set_cursor_position((
            chunks[2].x + 2 + console.input.chars().count().min(chunks[2].width.saturating_sub(3) as usize) as u16,
            chunks[2].y + 1,
        ));
    })?;
    Ok(())
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let horizontal = (100 - percent_x) / 2;
    let vertical = (100 - percent_y) / 2;
    Rect {
        x: area.x + area.width.saturating_mul(horizontal) / 100,
        y: area.y + area.height.saturating_mul(vertical) / 100,
        width: area.width.saturating_mul(percent_x) / 100,
        height: area.height.saturating_mul(percent_y) / 100,
    }
}

fn read_key() -> io::Result<KeyCode> {
    loop {
        if let Event::Key(key) = event::read()? {
            return Ok(key.code);
        }
    }
}
