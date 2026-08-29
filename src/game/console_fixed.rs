use crate::content::{campaign_content_load_diagnostics, load_campaign_content};
use crate::model::{Condition, EntityId, GameState, Item};
use crate::persistence::save_game;
use crate::ui;
use crossterm::event::KeyCode;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Terminal;
use std::env;
use std::fs;
use std::io;
use std::path::Path;

#[derive(Clone)]
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
    completion_scroll: usize,
    candidates: Vec<Candidate>,
    selected: usize,
    autocomplete: bool,
    exit: bool,
}

pub(crate) fn choose_main_menu(
    state: &mut GameState,
    save_path: &Path,
    title: &str,
    options: &[String],
) -> io::Result<Option<usize>> {
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
            KeyCode::Down | KeyCode::Char('j') => selected = (selected + 1) % options.len().max(1),
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
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let mut console = ConsoleState::default();
    console
        .output
        .push("Ashen Chronicle developer console".into());
    console
        .output
        .push("help for commands | Tab completion | Esc closes".into());

    loop {
        refresh_completion(&mut console, state);
        draw_console(&mut terminal, &console)?;
        let key = ui::read_key()?;

        if console.autocomplete {
            match key {
                KeyCode::Up => select_previous(&mut console),
                KeyCode::Down => select_next(&mut console),
                KeyCode::Enter => accept_completion(&mut console),
                KeyCode::Esc => cancel_completion(&mut console),
                KeyCode::Tab => {}
                _ => {
                    cancel_completion(&mut console);
                    edit_input(&mut console, key);
                }
            }
            continue;
        }

        match key {
            KeyCode::Esc => return Ok(()),
            KeyCode::Enter => {
                execute_line(state, save_path, &mut console)?;
                if console.exit {
                    return Ok(());
                }
            }
            KeyCode::Tab => {
                refresh_completion(&mut console, state);
                if !console.candidates.is_empty() {
                    console.autocomplete = true;
                    console.selected = 0;
                    console.completion_scroll = 0;
                    keep_completion_selection_visible(&mut console, 8);
                }
            }
            KeyCode::Up => history_previous(&mut console),
            KeyCode::Down => history_next(&mut console),
            KeyCode::Home => console.scroll = usize::MAX,
            KeyCode::End => console.scroll = 0,
            KeyCode::PageUp => console.scroll = console.scroll.saturating_add(6),
            KeyCode::PageDown => console.scroll = console.scroll.saturating_sub(6),
            _ => edit_input(&mut console, key),
        }
    }
}

fn edit_input(console: &mut ConsoleState, key: KeyCode) {
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

fn history_previous(console: &mut ConsoleState) {
    if console.history.is_empty() {
        return;
    }
    let index = console
        .history_index
        .map_or(console.history.len() - 1, |i| i.saturating_sub(1));
    console.history_index = Some(index);
    console.input = console.history[index].clone();
}

fn history_next(console: &mut ConsoleState) {
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

fn refresh_completion(console: &mut ConsoleState, state: &GameState) {
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
    console.input.clear();
    console.history_index = None;

    let parts = line.split_whitespace().collect::<Vec<_>>();
    let Some((command, args)) = parts.split_first() else {
        return Ok(());
    };

    match *command {
        "help" => help(console),
        "clear" => {
            console.output.clear();
            console.scroll = 0;
        }
        "status" => status(state, console),
        "where" => where_content(console),
        "mods" => mods(console),
        "content" => content(state, console),
        "locations" => locations(state, console),
        "goto" | "teleport" => goto(state, console, args),
        "npc" | "npcs" => npcs(state, console),
        "quests" => quests(state, console),
        "quest" => quest(state, console, args),
        "factions" => factions(state, console),
        "faction" => faction(state, console, args),
        "inventory" => inventory(state, console),
        "give" => give(state, console, args),
        "remove" => remove(state, console, args),
        "heal" => heal(state, console, args),
        "damage" => damage(state, console, args),
        "kill" => state.character.alive = false,
        "revive" => {
            state.character.alive = true;
            if state.character.hp <= 0 {
                state.character.hp = 1;
            }
        }
        "xp" => xp(state, console, args),
        "level" => level(state, console, args),
        "attr" => attr(state, console, args),
        "condition" => condition(state, console, args),
        "time" => time_cmd(state, console, args),
        "day" => day_cmd(state, console, args),
        "history" => history(state, console),
        "reload" => {
            state.campaign_content = Some(load_campaign_content());
            console.output.push("Campaign content reloaded.".into());
        }
        "save" => save(state, save_path, console)?,
        "exit" | "quit" => console.exit = true,
        _ => console
            .output
            .push(format!("Unknown command '{command}'. Try 'help'.")),
    }

    Ok(())
}

fn help(console: &mut ConsoleState) {
    console.output.push("help clear status where mods content locations goto teleport npc npcs quests quest factions faction inventory give remove heal damage kill revive xp level attr condition time day history reload save exit".into());
    console
        .output
        .push("Tab: candidates | ↑↓: select | Enter: accept | Esc: cancel autocomplete".into());
}

fn status(state: &GameState, console: &mut ConsoleState) {
    let location = state
        .world
        .location_by_id(state.character.location_id)
        .map(|location| location.name.as_str())
        .unwrap_or("<unknown>");
    console.output.push(format!(
        "{} [{}] HP {}/{} level {} XP {} | location {} ({}) | day {} time {}",
        state.character.display_name(),
        state.character.id,
        state.character.hp,
        state.character.max_hp,
        state.character.level,
        state.character.experience,
        state.character.location_id,
        location,
        state.world.day,
        state.world.time_points,
    ));
    console.output.push(format!(
        "world={} locations={} npcs={} quests={} factions={} inventory={}",
        state.world.name,
        state.world.locations.len(),
        state.npcs.len(),
        state.quests.len(),
        state.factions.len(),
        state.character.inventory.len(),
    ));
}

fn where_content(console: &mut ConsoleState) {
    console.output.push(format!(
        "cwd: {}",
        env::current_dir()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| "<unavailable>".into())
    ));
    if let Ok(exe) = env::current_exe() {
        console.output.push(format!("exe: {}", exe.display()));
        if let Some(dir) = exe.parent() {
            console.output.push(format!("exe_dir: {}", dir.display()));
            console
                .output
                .push(format!("data beside exe: {}", dir.join("data").display()));
            console.output.push(format!(
                "mods beside exe: {}",
                dir.join("data/mods").display()
            ));
        }
    }
}

fn mods(console: &mut ConsoleState) {
    let mut roots = Vec::new();
    if let Ok(cwd) = env::current_dir() {
        roots.push(cwd.join("data/mods"));
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            roots.push(dir.join("data/mods"));
        }
    }

    let mut found = false;
    for root in roots {
        if !root.exists() {
            console
                .output
                .push(format!("mods root missing: {}", root.display()));
            continue;
        }
        console
            .output
            .push(format!("mods root: {}", root.display()));
        match fs::read_dir(&root) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        found = true;
                        console
                            .output
                            .push(format!("  {}", entry.file_name().to_string_lossy()));
                    }
                }
            }
            Err(error) => console.output.push(format!("  read error: {error}")),
        }
    }
    if !found {
        console
            .output
            .push("No external mod directories discovered.".into());
    }
}

fn content(state: &GameState, console: &mut ConsoleState) {
    console.output.push("-- content loader diagnostics --".into());
    for line in campaign_content_load_diagnostics().lines() {
        console.output.push(line.to_string());
    }
    console.output.push("-- active game content --".into());

    let Some(content) = state.campaign_content.as_ref() else {
        console
            .output
            .push("Campaign content is not loaded.".into());
        return;
    };
    console.output.push(format!(
        "content: regions={} locations={} encounters={} events={} quests={} npcs={} factions={} items={}",
        1,
        content.world.locations.len(),
        content.encounters.len(),
        content.events.len(),
        content.quests.len(),
        content.npcs.len(),
        content.factions.len(),
        content.item_visuals.len(),
    ));
}

fn locations(state: &GameState, console: &mut ConsoleState) {
    for location in &state.world.locations {
        console
            .output
            .push(format!("{}: {}", location.id, location.name));
    }
}

fn goto(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let Some(raw) = args.first() else {
        console.output.push("usage: goto <location-id>".into());
        return;
    };
    let Ok(id) = raw.parse::<EntityId>() else {
        console.output.push("location id must be numeric".into());
        return;
    };
    if let Some(location) = state.world.location_by_id(id) {
        let name = location.name.clone();
        state.character.location_id = id;
        console.output.push(format!("Moved to {name} ({id})."));
    } else {
        console.output.push(format!("No location with id {id}."));
    }
}

fn npcs(state: &GameState, console: &mut ConsoleState) {
    for npc in &state.npcs {
        console
            .output
            .push(format!("{}: {}", npc.id, npc.display_name()));
    }
}

fn quests(state: &GameState, console: &mut ConsoleState) {
    for quest in &state.quests {
        console.output.push(format!(
            "{}: {} [{}] offered={} completed={}",
            quest.id, quest.title, quest.content_id, quest.offered, quest.completed
        ));
    }
}

fn quest(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let Some(action) = args.first().copied() else {
        console
            .output
            .push("usage: quest <list|complete|reset> [id]".into());
        return;
    };
    if action == "list" {
        quests(state, console);
        return;
    }
    let Some(raw) = args.get(1) else {
        console.output.push(format!("usage: quest {action} <id>"));
        return;
    };
    let Ok(id) = raw.parse::<EntityId>() else {
        console.output.push("quest id must be numeric".into());
        return;
    };
    let Some(quest) = state.quests.iter_mut().find(|quest| quest.id == id) else {
        console.output.push(format!("No quest with id {id}."));
        return;
    };
    match action {
        "complete" => {
            let content_id = quest.content_id.clone();
            quest.completed = true;
            state.world.completed_quest_ids.push(content_id);
        }
        "reset" => {
            let content_id = quest.content_id.clone();
            quest.completed = false;
            state
                .world
                .completed_quest_ids
                .retain(|value| value != &content_id);
        }
        _ => {
            console.output.push("unknown quest action".into());
            return;
        }
    }
    console.output.push(format!("Quest {id} updated."));
}

fn factions(state: &GameState, console: &mut ConsoleState) {
    for faction in &state.factions {
        console.output.push(format!(
            "{}: {} reputation={}",
            faction.id, faction.name, faction.reputation
        ));
    }
}

fn faction(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    if args.first().copied() == Some("list") {
        factions(state, console);
        return;
    }
    if args.first().copied() != Some("set") {
        console
            .output
            .push("usage: faction set <id> <reputation>".into());
        return;
    }
    let (Some(raw_id), Some(raw_rep)) = (args.get(1), args.get(2)) else {
        console
            .output
            .push("usage: faction set <id> <reputation>".into());
        return;
    };
    let (Ok(id), Ok(reputation)) = (raw_id.parse::<EntityId>(), raw_rep.parse::<i32>()) else {
        console
            .output
            .push("faction id and reputation must be numeric".into());
        return;
    };
    let Some(faction) = state.factions.iter_mut().find(|faction| faction.id == id) else {
        console.output.push(format!("No faction with id {id}."));
        return;
    };
    faction.reputation = reputation;
    console
        .output
        .push(format!("Faction {id} reputation={reputation}."));
}

fn inventory(state: &GameState, console: &mut ConsoleState) {
    for item in &state.character.inventory {
        console.output.push(format!("{}: {}", item.id, item.name));
    }
}

fn give(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let Some(raw) = args.first() else {
        console.output.push("usage: give <item-id>".into());
        return;
    };
    let Ok(id) = raw.parse::<EntityId>() else {
        console.output.push("item id must be numeric".into());
        return;
    };
    let Some(source) = state
        .character
        .inventory
        .iter()
        .find(|item| item.id == id)
        .cloned()
    else {
        console
            .output
            .push(format!("No inventory item with id {id}."));
        return;
    };
    let new_id = state.world.allocate_id();
    state.character.inventory.push(Item {
        id: new_id,
        ..source
    });
    console
        .output
        .push(format!("Cloned item {id} as {new_id}."));
}

fn remove(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let Some(raw) = args.first() else {
        console.output.push("usage: remove <item-id>".into());
        return;
    };
    let Ok(id) = raw.parse::<EntityId>() else {
        console.output.push("item id must be numeric".into());
        return;
    };
    let before = state.character.inventory.len();
    state.character.inventory.retain(|item| item.id != id);
    console
        .output
        .push(if state.character.inventory.len() < before {
            format!("Removed item {id}.")
        } else {
            format!("No item with id {id}.")
        });
}

fn heal(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let amount = parse_i32(args.first().copied()).unwrap_or(state.character.max_hp);
    state.character.heal(amount);
    console.output.push(format!(
        "HP {}/{}",
        state.character.hp, state.character.max_hp
    ));
}

fn damage(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let amount = parse_i32(args.first().copied()).unwrap_or(1).max(0);
    state.character.hp = (state.character.hp - amount).max(0);
    console.output.push(format!(
        "HP {}/{}",
        state.character.hp, state.character.max_hp
    ));
}

fn xp(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let Some(value) = parse_u32(args.first().copied()) else {
        console.output.push("usage: xp <amount>".into());
        return;
    };
    state.character.experience = value;
    console.output.push(format!("XP={value}"));
}

fn level(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let Some(value) = parse_u32(args.first().copied()) else {
        console.output.push("usage: level <number>".into());
        return;
    };
    state.character.level = value.max(1);
    console
        .output
        .push(format!("level={}", state.character.level));
}

fn attr(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let (Some(name), Some(raw)) = (args.first(), args.get(1)) else {
        console
            .output
            .push("usage: attr <might|insight|endurance> <value>".into());
        return;
    };
    let Ok(value) = raw.parse::<i32>() else {
        console
            .output
            .push("attribute value must be numeric".into());
        return;
    };
    match *name {
        "might" => state.character.attributes.might = value,
        "insight" => state.character.attributes.insight = value,
        "endurance" => state.character.attributes.endurance = value,
        _ => {
            console.output.push("unknown attribute".into());
            return;
        }
    }
    console.output.push(format!("{name}={value}"));
}

fn condition(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    match args.first().copied() {
        Some("clear") => {
            state.character.conditions.clear();
            console.output.push("Conditions cleared.".into());
        }
        Some("add") => {
            let Some(name) = args.get(1) else {
                console
                    .output
                    .push("usage: condition add <name> [remaining] [penalty]".into());
                return;
            };
            let remaining = args
                .get(2)
                .and_then(|raw| raw.parse::<u32>().ok())
                .unwrap_or(1);
            let penalty = args
                .get(3)
                .and_then(|raw| raw.parse::<i32>().ok())
                .unwrap_or(0);
            state
                .character
                .conditions
                .push(Condition::new(*name, remaining, penalty));
            console.output.push(format!("Added condition {name}."));
        }
        _ => console.output.push("usage: condition <add|clear>".into()),
    }
}

fn time_cmd(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let Some(value) = parse_u32(args.first().copied()) else {
        console.output.push("usage: time <value>".into());
        return;
    };
    state.world.time_points = value;
    console.output.push(format!("time_points={value}"));
}

fn day_cmd(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let Some(value) = parse_u32(args.first().copied()) else {
        console.output.push("usage: day <number>".into());
        return;
    };
    state.world.day = value.max(1);
    console.output.push(format!("day={}", state.world.day));
}

fn history(state: &GameState, console: &mut ConsoleState) {
    for entry in state.world.history.iter().rev().take(20).rev() {
        console
            .output
            .push(format!("t{}: {}", entry.turn, entry.text));
    }
}

fn save(state: &GameState, save_path: &Path, console: &mut ConsoleState) -> io::Result<()> {
    save_game(save_path, state)?;
    console
        .output
        .push(format!("Saved {}", save_path.display()));
    Ok(())
}

fn parse_i32(value: Option<&str>) -> Option<i32> {
    value.and_then(|raw| raw.parse::<i32>().ok())
}

fn parse_u32(value: Option<&str>) -> Option<u32> {
    value.and_then(|raw| raw.parse::<u32>().ok())
}

fn draw_console(
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

        let visible_rows = chunks[0].height as usize;
        let max_scroll = console.output.len().saturating_sub(visible_rows);
        let scroll = if console.scroll == usize::MAX {
            max_scroll
        } else {
            console.scroll.min(max_scroll)
        };
        let start = console.output.len().saturating_sub(visible_rows + scroll);
        let end = console.output.len().saturating_sub(scroll);
        let lines = console.output[start..end]
            .iter()
            .map(|line| Line::from(line.as_str()))
            .collect::<Vec<_>>();
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), chunks[0]);

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

fn keep_completion_selection_visible(console: &mut ConsoleState, visible: usize) {
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

fn select_previous(console: &mut ConsoleState) {
    if !console.candidates.is_empty() {
        console.selected = console
            .selected
            .checked_sub(1)
            .unwrap_or(console.candidates.len() - 1);
        keep_completion_selection_visible(console, 8);
    }
}

fn select_next(console: &mut ConsoleState) {
    if !console.candidates.is_empty() {
        console.selected = (console.selected + 1) % console.candidates.len();
        keep_completion_selection_visible(console, 8);
    }
}

fn cancel_completion(console: &mut ConsoleState) {
    console.autocomplete = false;
    console.candidates.clear();
    console.completion_scroll = 0;
}

fn accept_completion(console: &mut ConsoleState) {
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
