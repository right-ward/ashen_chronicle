use crate::model::GameState;
use crate::presentation::{
    AttributesView, CharacterSheetView, CharacterView, ConditionView, FactionView,
};
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
    let view = build_character_sheet_view(state);
    let tabs = vec!["Reputation".to_string(), "Journal".to_string()];

    loop {
        set_character_screen(&view);
        let Some(selection) = choose_from_list("General", &tabs, Some("Back"))? else {
            return Ok(());
        };
        show_character_tab(&view, selection)?;
    }
}

fn build_character_sheet_view(state: &GameState) -> CharacterSheetView {
    let character = &state.character;
    CharacterSheetView {
        character: CharacterView {
            name: character.name.clone(),
            title: character.title.clone(),
            hp: character.hp,
            max_hp: character.max_hp,
        },
        level: character.level,
        experience: character.experience,
        next_level_experience: character.level * 50,
        attributes: AttributesView {
            might: character.attributes.might,
            insight: character.attributes.insight,
            endurance: character.attributes.endurance,
            effective_might: character.effective_might(),
            effective_insight: character.effective_insight(),
            effective_endurance: character.effective_endurance(),
        },
        conditions: character
            .conditions
            .iter()
            .map(|condition| ConditionView {
                name: condition.name.clone(),
                remaining: condition.remaining,
                penalty: condition.penalty,
                bonus: condition.bonus,
            })
            .collect(),
        factions: state
            .factions
            .iter()
            .map(|faction| FactionView {
                name: faction.name.clone(),
                reputation: faction.reputation,
                memories: faction.memory.clone(),
            })
            .collect(),
        notes: character.notes.clone(),
    }
}

fn show_character_tab(view: &CharacterSheetView, tab: usize) -> std::io::Result<()> {
    match tab {
        0 => show_reputation_tab(view),
        1 => show_journal_tab(view),
        _ => Ok(()),
    }
}

fn set_character_screen(view: &CharacterSheetView) {
    let condition_lines = if view.conditions.is_empty() {
        vec!["Conditions: none".to_string()]
    } else {
        let mut lines = vec!["Conditions:".to_string()];
        lines.extend(view.conditions.iter().map(|condition| {
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
        view.character.display_name(),
        String::new(),
        format!("Level {}", view.level),
        format!(
            "Experience: {}/{}",
            view.experience, view.next_level_experience
        ),
        format!("Health: {}/{}", view.character.hp, view.character.max_hp),
        String::new(),
        format!("Might: {}", view.attributes.might),
        format!("Insight: {}", view.attributes.insight),
        format!("Endurance: {}", view.attributes.endurance),
        format!("Effective might: {}", view.attributes.effective_might),
        format!("Effective insight: {}", view.attributes.effective_insight),
        format!(
            "Effective endurance: {}",
            view.attributes.effective_endurance
        ),
        String::new(),
    ];
    general.extend(condition_lines);

    set_menu_screen("Character — General", Some(general.join("\n")), None);
}

fn show_reputation_tab(view: &CharacterSheetView) -> std::io::Result<()> {
    let mut lines = Vec::new();
    if view.factions.is_empty() {
        lines.push("No faction reputations have been recorded.".to_string());
    } else {
        for faction in &view.factions {
            lines.push(format!("{} {:+}", faction.name, faction.reputation));
            if faction.memories.is_empty() {
                lines.push("  No remembered dealings.".to_string());
            } else {
                lines.extend(
                    faction
                        .memories
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

fn show_journal_tab(view: &CharacterSheetView) -> std::io::Result<()> {
    let mut lines = Vec::new();
    if view.notes.is_empty() {
        lines.push("The journal is empty.".to_string());
    } else {
        for (index, note) in view.notes.iter().enumerate() {
            lines.push(format!("{}. {}", index + 1, note));
        }
    }
    set_menu_screen("Character — Journal", Some(lines.join("\n")), None);
    let _ = choose_from_list("Journal", &["Back to character".to_string()], None)?;
    Ok(())
}
