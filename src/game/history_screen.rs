use crate::game::time::time_display;
use crate::model::{GameState, HistoryEntryType};
use crate::presentation::{CharacterView, HistoryEntryView, HistoryEntryViewType, HistoryView};

pub(crate) fn build_view(state: &GameState) -> HistoryView {
    HistoryView {
        world_name: state.world.name.clone(),
        time: time_display(state.world.time_points, state.world.day),
        character: CharacterView {
            name: state.character.name.clone(),
            title: state.character.title.clone(),
            hp: state.character.hp,
            max_hp: state.character.max_hp,
        },
        entries: state
            .world
            .history
            .iter()
            .rev()
            .map(|entry| HistoryEntryView {
                day: entry.turn,
                entry_type: match entry.entry_type {
                    HistoryEntryType::Event => HistoryEntryViewType::Event,
                    HistoryEntryType::Narrative => HistoryEntryViewType::Narrative,
                },
                text: entry.text.clone(),
                event_id: entry.event_id.clone(),
                location_name: entry.location_name.clone(),
                outcome: entry.outcome.clone(),
            })
            .collect(),
    }
}
