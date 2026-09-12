use crate::model::GameState;
use crate::presentation::{
    AttributesView, CharacterSheetView, CharacterView, ConditionView, FactionView,
};
use crate::ui::choose_from_list;

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

pub(crate) fn build_character_sheet_view(state: &GameState) -> CharacterSheetView {
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

#[cfg(test)]
mod tests {
    use super::build_character_sheet_view;
    use crate::model::{create_new_state, Condition, Faction, WorldMode};

    #[test]
    fn character_sheet_view_preserves_derived_character_data() {
        let mut state = create_new_state(
            "Test World",
            WorldMode::New,
            "Ash".to_string(),
            "Wanderer".to_string(),
        );
        state.character.level = 3;
        state.character.experience = 27;
        state.character.hp = 8;
        state.character.attributes.might = 4;
        state.character.conditions = vec![Condition {
            name: "Wounded".to_string(),
            remaining: 2,
            penalty: -1,
            bonus: 0,
        }];
        let mut faction = Faction::new(20, "Wardens");
        faction.reputation = 5;
        faction.memory.push("A debt remembered.".to_string());
        state.factions.push(faction);

        let view = build_character_sheet_view(&state);

        assert_eq!(view.character.display_name(), "Ash the Wanderer");
        assert_eq!(view.level, 3);
        assert_eq!(view.experience, 27);
        assert_eq!(view.next_level_experience, 150);
        assert_eq!(view.attributes.might, 4);
        assert_eq!(view.attributes.effective_might, 3);
        assert_eq!(view.conditions[0].remaining, 2);
        assert_eq!(view.factions[0].reputation, 5);
        assert_eq!(view.factions[0].memories, vec!["A debt remembered."]);
    }
}
