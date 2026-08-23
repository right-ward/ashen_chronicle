use crate::model::GameState;
use crate::ui::{choose_from_list, pause};

macro_rules! println {
    () => {
        crate::ui::line("");
    };
    ($($arg:tt)*) => {
        crate::ui::line(&format!($($arg)*))
    };
}

pub(crate) fn gain_experience(state: &mut GameState, amount: u32) {
    state.character.experience += amount;
    loop {
        let threshold = state.character.level * 50;
        if state.character.experience < threshold {
            break;
        }
        state.character.experience -= threshold;
        state.character.level += 1;
        println!(
            "\nYou have grown stronger. You reached level {}.",
            state.character.level
        );
        let options = vec![
            "Might (+1 attack)".to_string(),
            "Insight (+1 search/recovery)".to_string(),
            "Endurance (+1 meditation healing)".to_string(),
        ];
        if let Ok(Some(choice)) = choose_from_list("Choose a new strength", &options, None) {
            match choice {
                0 => state.character.attributes.might += 1,
                1 => state.character.attributes.insight += 1,
                _ => state.character.attributes.endurance += 1,
            }
        }
    }
}

pub(crate) fn character_sheet(state: &GameState) {
    println!("\n=== Character ===");
    println!("{}", state.character.display_name());
    println!(
        "Level {}  XP {}/{}",
        state.character.level,
        state.character.experience,
        state.character.level * 50
    );
    println!(
        "Might: {}  Insight: {}  Endurance: {}",
        state.character.attributes.might,
        state.character.attributes.insight,
        state.character.attributes.endurance
    );
    println!(
        "Effective might: {}  Effective insight: {}",
        state.character.effective_might(),
        state.character.effective_insight()
    );
    if !state.character.conditions.is_empty() {
        println!(
            "Conditions: {}",
            state
                .character
                .conditions
                .iter()
                .map(|c| format!("{} ({} portions)", c.name, c.remaining))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    if state.factions.is_empty() {
        println!("Faction reputation: none");
    } else {
        println!("Faction reputation:");
        for faction in &state.factions {
            println!("  - {} {:+}", faction.name, faction.reputation);
        }
    }
    pause();
}
