use crate::game::state_effects;
use crate::model::GameState;
use crate::ui::{narrate, pause, prompt};

macro_rules! println {
    () => {
        crate::ui::line("");
    };
    ($($arg:tt)*) => {
        crate::ui::line(&format!($($arg)*))
    };
}

pub(crate) fn show_inventory(state: &GameState) {
    println!("\nInventory for {}", state.character.display_name());
    if state.character.inventory.is_empty() {
        println!("  Nothing.");
    } else {
        for item in &state.character.inventory {
            println!("  - {}: {}", item.name, item.description);
        }
    }
    pause();
}

pub(crate) fn review_quests(state: &GameState) {
    println!();
    println!("Quest log for {}", state.character.display_name());
    let visible_quests: Vec<_> = state
        .quests
        .iter()
        .enumerate()
        .filter(|(_, quest)| quest.offered || quest.completed)
        .collect();
    if visible_quests.is_empty() {
        println!("  Nothing yet.");
        pause();
        return;
    }
    for (index, quest) in visible_quests {
        let status = if quest.completed {
            if quest.reward_claimed {
                "completed"
            } else {
                "completed, reward pending"
            }
        } else {
            "active"
        };
        println!("  - {} [{}]", quest.title, status);
        println!("    {}", quest.description);
        for objective in crate::game::quests::objective_summary(state, index) {
            println!("    {}", objective);
        }
    }
    pause();
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
