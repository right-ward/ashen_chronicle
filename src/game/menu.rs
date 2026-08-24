use crate::model::GameState;

#[derive(Clone, Copy)]
pub(crate) enum GameAction {
    Travel,
    InvestigateThreat,
    SearchRemains,
    Talk,
    Meditate,
    QuestLog,
    Inventory,
    Journal,
    TestDeath,
    Quit,
    CharacterSheet,
}

pub(crate) struct MenuEntry {
    pub(crate) label: String,
    pub(crate) action: GameAction,
}

pub(crate) fn build_main_menu(state: &GameState) -> Vec<MenuEntry> {
    let mut menu = vec![
        MenuEntry {
            label: "Travel".to_string(),
            action: GameAction::Travel,
        },
        MenuEntry {
            label: "Meditate".to_string(),
            action: GameAction::Meditate,
        },
        MenuEntry {
            label: "Character sheet".to_string(),
            action: GameAction::CharacterSheet,
        },
        MenuEntry {
            label: "View inventory".to_string(),
            action: GameAction::Inventory,
        },
        MenuEntry {
            label: "Quest log".to_string(),
            action: GameAction::QuestLog,
        },
        MenuEntry {
            label: "Write journal note".to_string(),
            action: GameAction::Journal,
        },
        MenuEntry {
            label: "Talk".to_string(),
            action: GameAction::Talk,
        },
        MenuEntry {
            label: "Quit".to_string(),
            action: GameAction::Quit,
        },
        MenuEntry {
            label: "Test the death flow".to_string(),
            action: GameAction::TestDeath,
        },
    ];
    if state.threat.active {
        menu.insert(
            6,
            MenuEntry {
                label: "Investigate".to_string(),
                action: GameAction::InvestigateThreat,
            },
        );
    }
    if has_unscavenged_remains_at_location(state) {
        let insert_at = if state.threat.active { 7 } else { 6 };
        menu.insert(
            insert_at,
            MenuEntry {
                label: "Search remains".to_string(),
                action: GameAction::SearchRemains,
            },
        );
    }
    menu
}

fn has_unscavenged_remains_at_location(state: &GameState) -> bool {
    let location_id = state.character.location_id;
    state
        .corpses
        .iter()
        .any(|corpse| corpse.location_id == location_id && !corpse.inventory.is_empty())
}
