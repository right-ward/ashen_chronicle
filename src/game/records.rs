use crate::game::state_effects;
use crate::model::{GameState, Quest};
use crate::presentation::{
    CharacterView, InventoryDetailView, InventoryView, ItemView, QuestLogView, QuestObjectiveView,
    QuestView,
};
use crate::ui::{choose_from_list, narrate, prompt, set_menu_screen};

fn character_view(state: &GameState) -> CharacterView {
    CharacterView {
        name: state.character.name.clone(),
        title: state.character.title.clone(),
        hp: state.character.hp,
        max_hp: state.character.max_hp,
    }
}

fn item_view(item: &crate::model::Item) -> ItemView {
    ItemView {
        id: item.id,
        name: item.name.clone(),
        description: item.description.clone(),
    }
}

fn build_inventory_view(state: &GameState) -> InventoryView {
    InventoryView {
        character: character_view(state),
        items: state.character.inventory.iter().map(item_view).collect(),
    }
}

fn build_inventory_detail_view(state: &GameState, selected: usize) -> Option<InventoryDetailView> {
    let item = state.character.inventory.get(selected)?;
    Some(InventoryDetailView {
        item: item_view(item),
        position: selected + 1,
        total: state.character.inventory.len(),
        art: state
            .campaign_content
            .as_ref()
            .and_then(|content| content.item_art_for(&item.name))
            .map(str::to_string),
    })
}

pub(crate) fn show_inventory(state: &GameState) -> std::io::Result<()> {
    let view = build_inventory_view(state);
    if view.items.is_empty() {
        set_menu_screen(
            format!("Inventory — {}", view.character.display_name()),
            Some("Your pack is empty.".to_string()),
            None,
        );
        let _ = choose_from_list("Inventory", &["Back".to_string()], None)?;
        return Ok(());
    }

    let options: Vec<String> = view.items.iter().map(|item| item.name.clone()).collect();
    loop {
        set_inventory_screen(&view);
        let Some(selection) = choose_from_list("Select an item", &options, Some("Back"))? else {
            return Ok(());
        };
        if selection >= view.items.len() {
            continue;
        }
        show_inventory_detail(state, selection)?;
    }
}

fn set_inventory_screen(view: &InventoryView) {
    set_menu_screen(
        format!("Inventory — {}", view.character.display_name()),
        Some("Select an item to inspect its details.".to_string()),
        None,
    );
}

fn show_inventory_detail(state: &GameState, selected: usize) -> std::io::Result<()> {
    let Some(view) = build_inventory_detail_view(state, selected) else {
        return Ok(());
    };
    let description = if view.item.description.trim().is_empty() {
        "No description is available.".to_string()
    } else {
        view.item.description.clone()
    };
    let details = format!(
        "Item {} of {}\n\n{}",
        view.position, view.total, description
    );
    set_menu_screen(view.item.name, Some(details), view.art);
    let _ = choose_from_list("Item details", &["Back to inventory".to_string()], None)?;
    Ok(())
}

fn build_quest_view(quest: &Quest) -> QuestView {
    let status = if quest.completed {
        "COMPLETED"
    } else if quest_is_ready(quest) {
        "READY"
    } else {
        "ACTIVE"
    };
    let objectives = quest
        .objectives
        .iter()
        .map(|objective| QuestObjectiveView {
            label: objective.display_label(),
            progress: objective.progress,
            required: objective.required,
            completed: objective.completed,
        })
        .collect();
    QuestView {
        title: quest.title.clone(),
        description: quest.description.clone(),
        objectives,
        status: status.to_string(),
        completed: quest.completed,
        reward_claimed: quest.reward_claimed,
        reward_item_name: (!quest.reward_item_name.trim().is_empty())
            .then(|| quest.reward_item_name.clone()),
    }
}

fn build_quest_log_view(state: &GameState) -> QuestLogView {
    QuestLogView {
        character: character_view(state),
        quests: state
            .quests
            .iter()
            .filter(|quest| quest.offered || quest.completed)
            .map(build_quest_view)
            .collect(),
    }
}

pub(crate) fn review_quests(state: &GameState) -> std::io::Result<()> {
    let view = build_quest_log_view(state);
    if view.quests.is_empty() {
        set_menu_screen(
            "Quest Log",
            Some("No quests have been recorded yet.".to_string()),
            None,
        );
        let _ = crate::ui::choose_from_list("Quest Log", &["Back".to_string()], None)?;
        return Ok(());
    }

    let options: Vec<String> = view
        .quests
        .iter()
        .map(|quest| format!("[{}] {}", quest.status, quest.title))
        .collect();
    let visible_quest_indices: Vec<usize> = state
        .quests
        .iter()
        .enumerate()
        .filter(|(_, quest)| quest.offered || quest.completed)
        .map(|(index, _)| index)
        .collect();

    loop {
        set_menu_screen(
            format!("Quest Log — {}", view.character.display_name()),
            Some(
                "ACTIVE = in progress   READY = all objectives complete   COMPLETED = finished"
                    .to_string(),
            ),
            None,
        );
        let Some(selection) =
            crate::ui::choose_from_list("Select a quest", &options, Some("Back"))?
        else {
            return Ok(());
        };
        let Some(&quest_index) = visible_quest_indices.get(selection) else {
            continue;
        };
        show_quest_detail(state, quest_index)?;
    }
}

fn show_quest_detail(state: &GameState, quest_index: usize) -> std::io::Result<()> {
    let Some(quest) = state.quests.get(quest_index) else {
        return Ok(());
    };
    let view = build_quest_view(quest);

    let mut detail_lines = vec![format!("Status: {}", view.status), String::new()];
    if !view.description.trim().is_empty() {
        detail_lines.extend(view.description.lines().map(str::to_string));
        detail_lines.push(String::new());
    }
    detail_lines.push("Objectives".to_string());
    if view.objectives.is_empty() {
        detail_lines.push("  No objectives recorded.".to_string());
    } else {
        detail_lines.extend(view.objectives.iter().map(|objective| {
            let marker = if objective.completed { "x" } else { " " };
            format!(
                "  [{}] {} ({}/{})",
                marker, objective.label, objective.progress, objective.required
            )
        }));
    }
    detail_lines.push(String::new());
    detail_lines.push(format!(
        "Reward: {}",
        view.reward_item_name.as_deref().unwrap_or("—")
    ));

    set_menu_screen(view.title, Some(detail_lines.join("\n")), None);
    let _ = crate::ui::choose_from_list("Quest details", &["Back".to_string()], None)?;
    Ok(())
}

fn quest_is_ready(quest: &Quest) -> bool {
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
