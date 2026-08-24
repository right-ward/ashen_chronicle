use crate::content::load_campaign_content_report;
use crate::game::{presentation, world};
use crate::model::{Condition, EntityId, GameState, Item};
use crate::persistence::save_game;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
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
    selected_candidate: usize,
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
            let area = frame.area();
            let popup = centered_rect(64, 70, area);
            frame.render_widget(Clear, popup);
            let block = Block::default().title(title).borders(Borders::ALL);
            let inner = block.inner(popup);
            frame.render_widget(block, popup);
            let mut lines = vec![Line::from("↑↓ / jk  Enter: choose  /: console  Esc: back"), Line::from("")];
            for (index, option) in options.iter().enumerate() {
                let marker = if index == selected { '▶' } else { ' ' };
                lines.push(Line::from(format!("{marker} {}. {}", index + 1, option)));
            }
            frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
        })?;

        let Event::Key(key) = event::read()? else { continue };
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                selected = selected.checked_sub(1).unwrap_or(options.len().saturating_sub(1));
            }
            KeyCode::Down | KeyCode::Char('j') => {
                selected = (selected + 1) % options.len().max(1);
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
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let mut console = ConsoleState::default();
    console.output.extend([
        "Ashen Chronicle developer console".to_string(),
        "help: commands | Tab: completion | Esc: close".to_string(),
    ]);

    loop {
        refresh_completion(&mut console, state);
        draw_console(&mut terminal, &console)?;
        let Event::Key(key) = event::read()? else { continue };

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Ok(());
        }

        if console.autocomplete {
            match key.code {
                KeyCode::Up => select_previous(&mut console),
                KeyCode::Down => select_next(&mut console),
                KeyCode::Enter => accept_completion(&mut console),
                KeyCode::Esc => cancel_completion(&mut console),
                KeyCode::Tab => {}
                _ => {
                    cancel_completion(&mut console);
                    edit_input(&mut console, key.code);
                }
            }
            continue;
        }

        match key.code {
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
                    console.selected_candidate = 0;
                }
            }
            KeyCode::Up => history_previous(&mut console),
            KeyCode::Down => history_next(&mut console),
            KeyCode::PageUp => console.scroll = console.scroll.saturating_add(6),
            KeyCode::PageDown => console.scroll = console.scroll.saturating_sub(6),
            _ => edit_input(&mut console, key.code),
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
    if console.history.is_empty() { return; }
    let index = match console.history_index {
        None => console.history.len() - 1,
        Some(index) => index.saturating_sub(1),
    };
    console.history_index = Some(index);
    console.input = console.history[index].clone();
}

fn history_next(console: &mut ConsoleState) {
    let Some(index) = console.history_index else { return };
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
    let prefix = if trailing { "" } else { tokens.last().map(String::as_str).unwrap_or("") };
    let mut candidates = Vec::new();

    if tokens.len() <= 1 && !trailing {
        for (value, hint) in command_candidates() {
            if value.starts_with(prefix) {
                candidates.push(Candidate { value, hint });
            }
        }
    } else {
        match tokens.first().map(String::as_str) {
            Some("goto") | Some("teleport") => candidates = location_candidates(state, prefix),
            Some("quest") => {
                if tokens.len() <= 2 {
                    for value in ["list", "complete", "reset"] {
                        if value.starts_with(prefix) {
                            candidates.push(Candidate { value: value.to_string(), hint: "quest subcommand".into() });
                        }
                    }
                } else if matches!(tokens.get(1).map(String::as_str), Some("complete" | "reset")) {
                    candidates = quest_candidates(state, prefix);
                }
            }
            Some("faction") if tokens.len() <= 2 => {
                if "set".starts_with(prefix) {
                    candidates.push(Candidate { value: "set".into(), hint: "set reputation".into() });
                }
            }
            Some("faction") if tokens.get(1).map(String::as_str) == Some("set") => {
                candidates = faction_candidates(state, prefix);
            }
            Some("npcs") => candidates = npc_candidates(state, prefix),
            Some("give") | Some("remove") => candidates = item_candidates(state, prefix),
            _ => {}
        }
    }

    console.candidates = candidates;
    if console.candidates.is_empty() {
        console.autocomplete = false;
    } else if console.selected_candidate >= console.candidates.len() {
        console.selected_candidate = 0;
    }
}

fn tokenize(input: &str) -> (Vec<String>, bool) {
    let trailing = input.chars().last().is_some_and(char::is_whitespace);
    let tokens = input.split_whitespace().map(ToOwned::to_owned).collect();
    (tokens, trailing)
}

fn command_candidates() -> Vec<(String, String)> {
    [
        ("help", "show help"), ("clear", "clear output"), ("status", "current state"),
        ("where", "content paths"), ("mods", "loaded mods and warnings"), ("content", "content counts"),
        ("locations", "list location IDs"), ("goto", "move by location ID"), ("teleport", "goto alias"),
        ("npcs", "list NPC IDs"), ("quests", "list quests"), ("quest", "quest commands"),
        ("factions", "list faction IDs"), ("faction", "faction commands"), ("inventory", "list items"),
        ("give", "clone an inventory item"), ("remove", "remove an item"), ("heal", "restore HP"),
        ("damage", "deal damage"), ("kill", "kill character"), ("revive", "revive character"),
        ("xp", "set/add XP"), ("level", "set level"), ("attr", "set attribute"),
        ("condition", "add/clear condition"), ("time", "set/add time points"), ("day", "set day"),
        ("history", "show history"), ("reload", "reload content"), ("save", "save game"), ("exit", "close console"),
    ].into_iter().map(|(a,b)| (a.into(), b.into())).collect()
}

fn location_candidates(state: &GameState, prefix: &str) -> Vec<Candidate> {
    state.world.locations.iter().filter_map(|entry| {
        let value = entry.id.to_string();
        value.starts_with(prefix).then(|| Candidate { value, hint: entry.name.clone() })
    }).collect()
}

fn npc_candidates(state: &GameState, prefix: &str) -> Vec<Candidate> {
    state.npcs.iter().filter_map(|entry| {
        let value = entry.id.to_string();
        value.starts_with(prefix).then(|| Candidate { value, hint: entry.display_name() })
    }).collect()
}

fn quest_candidates(state: &GameState, prefix: &str) -> Vec<Candidate> {
    state.quests.iter().filter_map(|entry| {
        let value = entry.id.to_string();
        value.starts_with(prefix).then(|| Candidate { value, hint: entry.title.clone() })
    }).collect()
}

fn faction_candidates(state: &GameState, prefix: &str) -> Vec<Candidate> {
    state.factions.iter().filter_map(|entry| {
        let value = entry.id.to_string();
        value.starts_with(prefix).then(|| Candidate { value, hint: entry.name.clone() })
    }).collect()
}

fn item_candidates(state: &GameState, prefix: &str) -> Vec<Candidate> {
    state.character.inventory.iter().filter_map(|entry| {
        let value = entry.id.to_string();
        value.starts_with(prefix).then(|| Candidate { value, hint: entry.name.clone() })
    }).collect()
}

fn select_previous(console: &mut ConsoleState) {
    if console.candidates.is_empty() { return; }
    console.selected_candidate = console.selected_candidate.checked_sub(1).unwrap_or(console.candidates.len() - 1);
}

fn select_next(console: &mut ConsoleState) {
    if !console.candidates.is_empty() {
        console.selected_candidate = (console.selected_candidate + 1) % console.candidates.len();
    }
}

fn cancel_completion(console: &mut ConsoleState) {
    console.autocomplete = false;
    console.candidates.clear();
}

fn accept_completion(console: &mut ConsoleState) {
    let Some(candidate) = console.candidates.get(console.selected_candidate).cloned() else { return };
    let (mut tokens, trailing) = tokenize(&console.input);
    if trailing { tokens.push(candidate.value); }
    else if let Some(last) = tokens.last_mut() { *last = candidate.value; }
    else { tokens.push(candidate.value); }
    console.input = tokens.join(" ");
    cancel_completion(console);
}

fn execute_line(state: &mut GameState, save_path: &Path, console: &mut ConsoleState) -> io::Result<()> {
    let line = console.input.trim().to_string();
    if line.is_empty() { return Ok(()) }
    console.output.push(format!("> {line}"));
    if console.history.last() != Some(&line) { console.history.push(line.clone()); }
    console.input.clear();
    console.history_index = None;
    let parts = line.split_whitespace().collect::<Vec<_>>();
    let (command, args) = parts.split_first().unwrap();

    match *command {
        "help" => help(console),
        "clear" => console.output.clear(),
        "status" => status(state, console),
        "where" => where_content(console),
        "mods" => mods(console),
        "content" => content(state, console),
        "locations" => locations(state, console),
        "goto" | "teleport" => goto(state, console, args),
        "npcs" => npcs(state, console, args),
        "quests" => quests(state, console),
        "quest" => quest(state, console, args),
        "factions" => factions(state, console),
        "faction" => faction(state, console, args),
        "inventory" => inventory(state, console),
        "give" => give(state, console, args),
        "remove" => remove(state, console, args),
        "heal" => heal(state, console, args),
        "damage" => damage(state, console, args),
        "kill" => kill(state, console),
        "revive" => revive(state, console),
        "xp" => xp(state, console, args),
        "level" => level(state, console, args),
        "attr" => attr(state, console, args),
        "condition" => condition(state, console, args),
        "time" => time(state, console, args),
        "day" => day(state, console, args),
        "history" => history(state, console, args),
        "reload" => reload(state, console),
        "save" => save(state, save_path, console),
        "exit" | "quit" => console.exit = true,
        _ => console.output.push(format!("Unknown command '{command}'. Try 'help'.")),
    }
    Ok(())
}

fn help(console: &mut ConsoleState) {
    console.output.push("help clear status where mods content locations goto teleport npcs quests quest factions faction inventory give remove heal damage kill revive xp level attr condition time day history reload save exit".into());
    console.output.push("Tab completes commands and entity IDs; arrows select a completion; Enter accepts; Esc cancels completion.".into());
}

fn status(state: &GameState, console: &mut ConsoleState) {
    let location = state.world.location_by_id(state.character.location_id).map(|l| l.name.as_str()).unwrap_or("<unknown>");
    console.output.extend([
        format!("Character {} [{}]", state.character.display_name(), state.character.id),
        format!("HP {}/{}  alive={}  level={}  xp={}", state.character.hp, state.character.max_hp, state.character.alive, state.character.level, state.character.experience),
        format!("World {} [{}]  day={}  time_points={}", state.world.name, state.world.id, state.world.day, state.world.time_points),
        format!("Location {} [{}]", location, state.character.location_id),
    ]);
}

fn where_content(console: &mut ConsoleState) {
    let cwd = env::current_dir().ok();
    let exe = env::current_exe().ok();
    console.output.push(format!("cwd: {}", display_opt(cwd.as_deref())));
    console.output.push(format!("exe: {}", display_opt(exe.as_deref())));
    let mut roots = Vec::new();
    if let Some(cwd) = cwd { roots.push(cwd.join("data")); }
    if let Some(exe) = exe {
        if let Some(dir) = exe.parent() { roots.push(dir.join("data")); if let Some(parent) = dir.parent() { roots.push(parent.join("data")); } }
    }
    for root in roots {
        console.output.push(format!("data {} [{}]", root.display(), if root.is_dir() { "dir" } else { "missing" }));
        console.output.push(format!("  base {}", root.join("base_content.json").display()));
        console.output.push(format!("  mods {}", root.join("mods").display()));
    }
}

fn display_opt(path: Option<&Path>) -> String { path.map(|p| p.display().to_string()).unwrap_or_else(|| "<unavailable>".into()) }

fn mods(console: &mut ConsoleState) {
    let report = load_campaign_content_report();
    console.output.push(format!("Loaded mods: {}", report.loaded_mods.len()));
    for manifest in report.loaded_mods { console.output.push(format!("  {} — {} v{}", manifest.id, manifest.name, manifest.version)); }
    console.output.push(format!("Warnings: {}", report.warnings.len()));
    for warning in report.warnings { console.output.push(format!("  ! {warning}")); }
}

fn content(state: &GameState, console: &mut ConsoleState) {
    let Some(content) = &state.campaign_content else { console.output.push("campaign content: not loaded".into()); return };
    console.output.extend([
        format!("locations={} factions={} npcs={} quests={}", content.world.locations.len(), content.factions.len(), content.npcs.len(), content.quests.len()),
        format!("encounters={} events={} atmospheres={} item_visuals={}", content.encounters.len(), content.events.len(), content.atmospheres.len(), content.item_visuals.len()),
    ]);
}

fn locations(state: &GameState, console: &mut ConsoleState) {
    for location in &state.world.locations { console.output.push(format!("{} — {}", location.id, location.name)); }
}

fn goto(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let Some(id) = args.first().and_then(|v| v.parse::<EntityId>().ok()) else { console.output.push("usage: goto <location-id>".into()); return };
    let Some(name) = state.world.location_by_id(id).map(|l| l.name.clone()) else { console.output.push(format!("unknown location id {id}")); return };
    state.character.location_id = id;
    state.last_announced_location_id = None;
    console.output.push(format!("moved to {name} [{id}]"));
}

fn npcs(state: &GameState, console: &mut ConsoleState, args: &[&str]) {
    let location = args.first().and_then(|v| v.parse::<EntityId>().ok());
    for npc in &state.npcs {
        if location.is_some_and(|id| npc.location_id != id) { continue }
        console.output.push(format!("{} — {}", npc.id, npc.display_name()));
    }
}

fn quests(state: &GameState, console: &mut ConsoleState) {
    for quest in &state.quests { console.output.push(format!("{} — {} [{}]", quest.id, quest.title, if quest.completed { "complete" } else if quest.offered { "offered" } else { "new" })); }
}

fn quest(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    match args {
        [] | ["list"] => quests(state, console),
        [action, id] if matches!(*action, "complete" | "reset") => {
            let Some(id) = id.parse::<EntityId>().ok() else { console.output.push("quest id must be numeric".into()); return };
            let Some(entry) = state.quests.iter_mut().find(|q| q.id == id) else { console.output.push(format!("unknown quest id {id}")); return };
            if *action == "complete" {
                entry.completed = true;
                entry.offered = true;
                if !state.world.completed_quest_ids.contains(&entry.content_id) { state.world.completed_quest_ids.push(entry.content_id.clone()); }
                console.output.push(format!("completed {}", entry.title));
            } else {
                entry.completed = false;
                entry.reward_claimed = false;
                state.world.completed_quest_ids.retain(|value| value != &entry.content_id);
                console.output.push(format!("reset {}", entry.title));
            }
        }
        _ => console.output.push("usage: quest list | quest complete <id> | quest reset <id>".into()),
    }
}

fn factions(state: &GameState, console: &mut ConsoleState) { for faction in &state.factions { console.output.push(format!("{} — {} ({:+})", faction.id, faction.name, faction.reputation)); } }

fn faction(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let ["set", id, value] = args else { console.output.push("usage: faction set <id> <reputation>".into()); return };
    let (Some(id), Some(value)) = (id.parse::<EntityId>().ok(), value.parse::<i32>().ok()) else { console.output.push("id/reputation invalid".into()); return };
    let Some(entry) = state.factions.iter_mut().find(|f| f.id == id) else { console.output.push(format!("unknown faction id {id}")); return };
    entry.reputation = value;
    console.output.push(format!("{} reputation={:+}", entry.name, value));
}

fn inventory(state: &GameState, console: &mut ConsoleState) { for item in &state.character.inventory { console.output.push(format!("{} — {}", item.id, item.name)); } if state.character.inventory.is_empty() { console.output.push("inventory: empty".into()); } }

fn give(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let Some(id) = args.first().and_then(|v| v.parse::<EntityId>().ok()) else { console.output.push("usage: give <inventory-item-id> [count]".into()); return };
    let count = args.get(1).and_then(|v| v.parse::<usize>().ok()).unwrap_or(1).clamp(1, 100);
    let Some(item) = state.character.inventory.iter().find(|item| item.id == id).cloned() else { console.output.push(format!("unknown item id {id}")); return };
    for _ in 0..count { let new_id = state.world.allocate_id(); state.character.inventory.push(Item { id: new_id, name: item.name.clone(), description: item.description.clone() }); }
    console.output.push(format!("added {} x{}", item.name, count));
}

fn remove(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let Some(id) = args.first().and_then(|v| v.parse::<EntityId>().ok()) else { console.output.push("usage: remove <inventory-item-id>".into()); return };
    let before = state.character.inventory.len();
    state.character.inventory.retain(|item| item.id != id);
    console.output.push(if before == state.character.inventory.len() { format!("unknown item id {id}") } else { format!("removed item {id}") });
}

fn heal(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) { let amount = args.first().and_then(|v| v.parse::<i32>().ok()).unwrap_or(state.character.max_hp); state.character.heal(amount.max(0)); console.output.push(format!("HP {}/{}", state.character.hp, state.character.max_hp)); }
fn damage(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) { let amount = args.first().and_then(|v| v.parse::<i32>().ok()).unwrap_or(1).max(0); state.character.hp = (state.character.hp - amount).max(0); if state.character.hp == 0 { state.character.alive = false; } console.output.push(format!("HP {}/{} alive={}", state.character.hp, state.character.max_hp, state.character.alive)); }
fn kill(state: &mut GameState, console: &mut ConsoleState) { state.character.hp = 0; state.character.alive = false; console.output.push("character killed".into()); }
fn revive(state: &mut GameState, console: &mut ConsoleState) { state.character.alive = true; state.character.hp = state.character.max_hp.max(1); console.output.push("character revived".into()); }

fn xp(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let (mode, value) = match args { ["set", value] => ("set", value.parse::<u32>().ok()), ["add", value] | [value] => ("add", value.parse::<u32>().ok()), _ => ("add", None) };
    let Some(value) = value else { console.output.push("usage: xp set <n> | xp add <n>".into()); return };
    state.character.experience = if mode == "set" { value } else { state.character.experience.saturating_add(value) };
    console.output.push(format!("xp={}", state.character.experience));
}

fn level(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) { let value = match args { ["set", value] | [value] => value.parse::<u32>().ok(), _ => None }; let Some(value) = value else { console.output.push("usage: level set <n>".into()); return }; state.character.level = value.max(1); console.output.push(format!("level={}", state.character.level)); }

fn attr(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    let args = if args.first().copied() == Some("set") { &args[1..] } else { args };
    let [name, value] = args else { console.output.push("usage: attr set <might|insight|endurance> <n>".into()); return };
    let Some(value) = value.parse::<i32>().ok() else { console.output.push("attribute value must be numeric".into()); return };
    match *name { "might" => state.character.attributes.might = value, "insight" => state.character.attributes.insight = value, "endurance" => state.character.attributes.endurance = value, _ => { console.output.push("unknown attribute".into()); return } }
    console.output.push(format!("{name}={value}"));
}

fn condition(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) {
    match args {
        ["clear"] => { state.character.conditions.clear(); console.output.push("conditions cleared".into()); }
        ["add", name, remaining, penalty, bonus] => {
            let (Ok(remaining), Ok(penalty), Ok(bonus)) = (remaining.parse::<u32>(), penalty.parse::<i32>(), bonus.parse::<i32>()) else { console.output.push("invalid condition values".into()); return };
            state.character.conditions.push(Condition { name: (*name).into(), remaining, penalty, bonus });
            console.output.push(format!("added condition {name}"));
        }
        _ => console.output.push("usage: condition add <name> <remaining> <penalty> <bonus> | condition clear".into()),
    }
}

fn time(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) { let (mode, value) = match args { ["set", value] | [value] => ("set", value.parse::<u32>().ok()), ["add", value] => ("add", value.parse::<u32>().ok()), _ => ("set", None) }; let Some(value) = value else { console.output.push("usage: time set <0..3> | time add <n>".into()); return }; state.world.time_points = if mode == "set" { value.min(3) } else { state.world.time_points.saturating_add(value).min(3) }; console.output.push(format!("time_points={}", state.world.time_points)); }
fn day(state: &mut GameState, console: &mut ConsoleState, args: &[&str]) { let value = match args { ["set", value] | [value] => value.parse::<u32>().ok(), _ => None }; let Some(value) = value else { console.output.push("usage: day set <n>".into()); return }; state.world.day = value.max(1); console.output.push(format!("day={}", state.world.day)); }
fn history(state: &GameState, console: &mut ConsoleState, args: &[&str]) { let count = args.first().and_then(|v| v.parse::<usize>().ok()).unwrap_or(10).min(100); for entry in state.world.history.iter().rev().take(count).rev() { console.output.push(format!("[{}] {}", entry.turn, entry.text)); } }

fn reload(state: &mut GameState, console: &mut ConsoleState) {
    let report = load_campaign_content_report();
    let loaded = report.loaded_mods.len();
    let warnings = report.warnings.len();
    state.campaign_content = Some(report.content);
    world::bootstrap_campaign_content(state);
    state.last_announced_location_id = None;
    console.output.push(format!("reloaded content: mods={} warnings={}", loaded, warnings));
}

fn save(state: &GameState, path: &Path, console: &mut ConsoleState) { console.output.push(match save_game(path, state) { Ok(()) => format!("saved {}", path.display()), Err(err) => format!("save failed: {err}") }); }

fn draw_console(terminal: &mut Terminal<CrosstermBackend<Stdout>>, console: &ConsoleState) -> io::Result<()> {
    terminal.draw(|frame| {
        let popup = centered_rect(92, 88, frame.area());
        frame.render_widget(Clear, popup);
        let block = Block::default().title("DEV CONSOLE").borders(Borders::ALL);
        let inner = block.inner(popup);
        frame.render_widget(block, popup);
        let candidate_height = if console.autocomplete { (console.candidates.len() as u16 + 2).min(inner.height.saturating_sub(7).max(3)) } else { 0 };
        let chunks = Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(4), Constraint::Length(candidate_height), Constraint::Length(3)]).split(inner);
        let visible = chunks[0].height.saturating_sub(2) as usize;
        let start = console.output.len().saturating_sub(visible).saturating_sub(console.scroll);
        let output = console.output[start..].join("\n");
        frame.render_widget(Paragraph::new(output).block(Block::default().borders(Borders::ALL).title("output")).wrap(Wrap { trim: false }), chunks[0]);
        if candidate_height > 0 {
            let lines = console.candidates.iter().enumerate().map(|(index, candidate)| { let marker = if index == console.selected_candidate { '▶' } else { ' ' }; Line::from(format!("{marker} {:<12} {}", candidate.value, candidate.hint)) }).collect::<Vec<_>>();
            frame.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("completion")), chunks[1]);
        }
        frame.render_widget(Paragraph::new(format!("/ {}", console.input)).block(Block::default().borders(Borders::ALL).title("input")), chunks[2]);
        frame.set_cursor_position((chunks[2].x + 2 + console.input.len().min(chunks[2].width.saturating_sub(3) as usize) as u16, chunks[2].y + 1));
    })?;
    Ok(())
}

fn centered_rect(px: u16, py: u16, area: Rect) -> Rect { Rect { x: area.x + area.width.saturating_mul(100 - px) / 200, y: area.y + area.height.saturating_mul(100 - py) / 200, width: area.width.saturating_mul(px) / 100, height: area.height.saturating_mul(py) / 100 } }
