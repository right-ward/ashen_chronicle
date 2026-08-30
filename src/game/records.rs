use crate::game::state_effects;
use crate::model::{GameState, Quest};
use crate::ui::{choose_from_list, narrate, prompt, set_menu_screen};

macro_rules! println {
    () => {
        crate::ui::line("");
    };
    ($($arg:tt)*) => {
        crate::ui::line(&format!($($arg)*))
    };
}

pub(crate) fn show_inventory(state: &GameState) -> std::io::Result<()> {
    if state.character.inventory.is_empty() {
        set_menu_screen(
            format!("Inventory — {}", state.character.display_name()),
            Some("Your pack is empty.".to_string()),
            None,
        );
        let _ = choose_from_list("Inventory", &["Back".to_string()], None)?;
        crate::game::presentation::render_state(state);
        return Ok(());
    }

    let options: Vec<String> = state
        .character
        .inventory
        .iter()
        .map(|item| item.name.clone())
        .collect();

    loop {
        set_inventory_screen(state);
        let Some(selection) = choose_from_list("Select an item", &options, Some("Back"))? else {
            crate::game::presentation::render_state(state);
            return Ok(());
        };
        if selection >= state.character.inventory.len() {
            continue;
        }
        show_inventory_detail(state, selection)?;
    }
}

fn set_inventory_screen(state: &GameState) {
    set_menu_screen(
        format!("Inventory — {}", state.character.display_name()),
        Some("Select an item to inspect its details.".to_string()),
        None,
    );
}

fn inventory_details(state: &GameState, selected: usize) -> (String, Option<String>) {
    let item = &state.character.inventory[selected];
    let mut details = vec![format!(
        "Item {} of {}",
        selected + 1,
        state.character.inventory.len()
    )];
    details.push(String::new());
    if !item.description.trim().is_empty() {
        details.extend(item.description.lines().map(str::to_string));
    } else {
        details.push("No description is available.".to_string());
    }

    let art = state
        .campaign_content
        .as_ref()
        .and_then(|content| content.item_art_for(&item.name))
        .map(str::to_string);

    (details.join("\n"), art)
}

fn show_inventory_detail(state: &GameState, selected: usize) -> std::io::Result<()> {
    if selected >= state.character.inventory.len() {
        return Ok(());
    }
    let (details, art) = inventory_details(state, selected);
    set_menu_screen(
        state.character.inventory[selected].name.clone(),
        Some(details),
        art,
    );
    let _ = choose_from_list("Item details", &["Back to inventory".to_string()], None)?;
    Ok(())
}

pub(crate) fn review_quests(state: &GameState) -> std::io::Result<()> {
    let visible_quests: Vec<_> = state
        .quests
        .iter()
        .enumerate()
        .filter(|(_, quest)| quest.offered || quest.completed)
        .collect();

    if visible_quests.is_empty() {
        set_menu_screen(
            "Quest Log",
            Some("No quests have been recorded yet.".to_string()),
            None,
        );
        let _ = crate::ui::choose_from_list("Quest Log", &["Back".to_string()], None)?;
        crate::game::presentation::render_state(state);
        return Ok(());
    }

    let options: Vec<String> = visible_quests
        .iter()
        .map(|(_, quest)| format!("{} {}", quest_status_marker(state, quest), quest.title))
        .collect();

    loop {
        set_menu_screen(
            format!("Quest Log — {}", state.character.display_name()),
            Some(
                "ACTIVE = in progress   READY = all objectives complete   COMPLETED = finished"
                    .to_string(),
            ),
            None,
        );

        let Some(selection) =
            crate::ui::choose_from_list("Select a quest", &options, Some("Back"))?
        else {
            crate::game::presentation::render_state(state);
            return Ok(());
        };

        let Some((quest_index, _)) = visible_quests.get(selection).copied() else {
            continue;
        };

        show_quest_detail(state, quest_index)?;
    }
}

fn show_quest_detail(state: &GameState, quest_index: usize) -> std::io::Result<()> {
    let Some(quest) = state.quests.get(quest_index) else {
        return Ok(());
    };

    let status = quest_status_label(state, quest);
    let mut detail_lines = vec![format!("Status: {}", status), String::new()];
    if !quest.description.trim().is_empty() {
        detail_lines.extend(quest.description.lines().map(str::to_string));
        detail_lines.push(String::new());
    }
    detail_lines.push("Objectives".to_string());
    let objectives = crate::game::quests::objective_summary(state, quest_index);
    if objectives.is_empty() {
        detail_lines.push("  No objectives recorded.".to_string());
    } else {
        detail_lines.extend(objectives.into_iter().map(|line| format!("  {}", line)));
    }
    detail_lines.push(String::new());
    if !quest.reward_item_name.trim().is_empty() {
        detail_lines.push(format!("Reward: {}", quest.reward_item_name));
    } else {
        detail_lines.push("Reward: —".to_string());
    }

    set_menu_screen(quest.title.clone(), Some(detail_lines.join("\n")), None);
    let _ = crate::ui::choose_from_list("Quest details", &["Back".to_string()], None)?;
    Ok(())
}

fn quest_status_marker(state: &GameState, quest: &Quest) -> &'static str {
    if quest.completed {
        "[COMPLETED]"
    } else if quest_is_ready(state, quest) {
        "[READY]"
    } else {
        "[ACTIVE]"
    }
}

fn quest_status_label(state: &GameState, quest: &Quest) -> &'static str {
    if quest.completed {
        "COMPLETED"
    } else if quest_is_ready(state, quest) {
        "READY"
    } else {
        "ACTIVE"
    }
}

fn quest_is_ready(state: &GameState, quest: &Quest) -> bool {
    let _ = state;
    !quest.completed
        && !quest.objectives.is_empty()
        && quest.objectives.iter().all(|objective| objective.completed)
}

pub(crate) fn write_note(state: &mut GameState) -> std::io::Result<()> {
    let note = prompt("Write a journal note: ")?;
    if !note.is_empty() {
        state.character.notes.push(note.clone());
        state_effects::advance_time(state, 1);
        state.character.turn += 1;
        let character_name = state.character.display_name();
        state.world.record_history(
            state.character.turn,
            format!("{} noted: {}", character_name, note),
        );
        narrate("The journal entry is recorded.");
    }
    Ok(())
}
