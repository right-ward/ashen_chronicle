use crate::game::{actions, combat, interactions, legacy, menu, screens};
use crate::model::GameState;
use std::io;
use std::path::Path;

pub(crate) fn dispatch(
    state: &mut GameState,
    action: menu::GameAction,
    save_path: &Path,
) -> io::Result<bool> {
    match action {
        menu::GameAction::Travel => actions::travel(state)?,
        menu::GameAction::InvestigateThreat => combat::investigate_threat(state)?,
        menu::GameAction::SearchRemains => legacy::search_remains(state)?,
        menu::GameAction::Talk => interactions::talk(state)?,
        menu::GameAction::Meditate => actions::meditate_and_save(state, save_path)?,
        menu::GameAction::QuestLog => actions::review_quests(state),
        menu::GameAction::Inventory => actions::show_inventory(state),
        menu::GameAction::Journal => actions::write_note(state)?,
        menu::GameAction::CharacterSheet => actions::character_sheet(state),
        menu::GameAction::TestDeath => legacy::force_death(state),
        menu::GameAction::Quit => return screens::quit_screen(),
    }
    Ok(false)
}
