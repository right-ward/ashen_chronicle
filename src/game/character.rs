use crate::model::GameState;
use crate::ui::{choose_from_list, set_menu_screen};

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

pub(crate) fn character_sheet(state: &GameState) -> std::io::Result<()> {
    let tabs = vec!["Reputation".to_string(), "Journal".to_string()];

    loop {
        set_character_screen(state, 0);
        let Some(selection) = choose_from_list("General", &tabs, Some("Back"))? else {
            return Ok(());
        };
        show_character_tab(state, selection)?;
    }
}

fn show_character_tab(state: &GameState, tab: usize) -> std::io::Result<()> {
    match tab {
        0 => show_reputation_tab(state),
        1 => show_journal_tab(state),
        _ => Ok(()),
    }
}

fn set_character_screen(state: &GameState, _tab: usize) {
    let character = &state.character;
    let condition_lines = if character.conditions.is_empty() {
        vec!["Conditions: none".to_string()]
    } else {
        let mut lines = vec!["Conditions:".to_string()];
        lines.extend(character.conditions.iter().map(|condition| {
            let effect = match (condition.penalty, condition.bonus) {
                (penalty, bonus) if penalty < 0 && bonus > 0 => {
                    format!(" {:+} penalty, {:+} bonus", penalty, bonus)
                }
                (penalty, _) if penalty < 0 => format!(" {:+} penalty", penalty),
                (_, bonus) if bonus > 0 => format!(" {:+} bonus", bonus),
                _ => String::new(),
            };
            format!(
                "  - {} ({} portions){}",
                condition.name, condition.remaining, effect
            )
        }));
        lines
    };

    let mut general = vec![
        character.display_name(),
        String::new(),
        format!("Level {}", character.level),
        format!(
            "Experience: {}/{}",
            character.experience,
            character.level * 50
        ),
        format!("Health: {}/{}", character.hp, character.max_hp),
        String::new(),
        format!("Might: {}", character.attributes.might),
        format!("Insight: {}", character.attributes.insight),
        format!("Endurance: {}", character.attributes.endurance),
        format!("Effective might: {}", character.effective_might()),
        format!("Effective insight: {}", character.effective_insight()),
        format!("Effective endurance: {}", character.effective_endurance()),
        String::new(),
    ];
    general.extend(condition_lines);

    set_menu_screen("Character — General", Some(general.join("\n")), None);
}

fn show_reputation_tab(state: &GameState) -> std::io::Result<()> {
    let mut lines = Vec::new();
    if state.factions.is_empty() {
        lines.push("No faction reputations have been recorded.".to_string());
    } else {
        for faction in &state.factions {
            lines.push(format!("{} {:+}", faction.name, faction.reputation));
            if faction.memory.is_empty() {
                lines.push("  No remembered dealings.".to_string());
            } else {
                lines.extend(
                    faction
                        .memory
                        .iter()
                        .rev()
                        .map(|memory| format!("  - {}", memory)),
                );
            }
            lines.push(String::new());
        }
        lines.pop();
    }
    set_menu_screen("Character — Reputation", Some(lines.join("\n")), None);
    let _ = choose_from_list("Reputation", &["Back to character".to_string()], None)?;
    Ok(())
}

fn show_journal_tab(state: &GameState) -> std::io::Result<()> {
    let mut lines = Vec::new();
    if state.character.notes.is_empty() {
        lines.push("The journal is empty.".to_string());
    } else {
        for (index, note) in state.character.notes.iter().enumerate() {
            lines.push(format!("{}. {}", index + 1, note));
        }
    }
    set_menu_screen("Character — Journal", Some(lines.join("\n")), None);
    let _ = choose_from_list("Journal", &["Back to character".to_string()], None)?;
    Ok(())
}
