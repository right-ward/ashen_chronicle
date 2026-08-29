use super::console_ui::{ConsoleState, ScrollPosition};
use crate::content::{campaign_content_load_diagnostics, load_campaign_content};
use crate::model::{Condition, EntityId, GameState, Item};
use crate::persistence::save_game;
use std::env;
use std::fs;
use std::io;
use std::path::Path;
pub(super) fn execute_line(
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
            console.scroll = ScrollPosition::Follow;
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
        "logkeys" => logkeys(console, args),
        "exit" | "quit" => console.exit = true,
        _ => console
            .output
            .push(format!("Unknown command '{command}'. Try 'help'.")),
    }
    Ok(())
}
fn help(console: &mut ConsoleState) {
    console.output.push("help clear status where mods content locations goto teleport npc npcs quests quest factions faction inventory give remove heal damage kill revive xp level attr condition time day history reload save logkeys exit".into());
    console
        .output
        .push("Tab: candidates | ↑↓: select | Enter: accept | Esc: cancel autocomplete".into());
}
fn logkeys(console: &mut ConsoleState, args: &[&str]) {
    let enabled = match args.first().copied() {
        None | Some("false") => false,
        Some("true") => true,
        Some(_) => {
            console.output.push("usage: logkeys [true|false]".into());
            return;
        }
    };
    crate::ui::set_key_logging(enabled);
    console.output.push(format!(
        "Key-event logging {}.",
        if crate::ui::key_logging_enabled() {
            "enabled"
        } else {
            "disabled"
        }
    ));
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
    console
        .output
        .push("-- content loader diagnostics --".into());
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
