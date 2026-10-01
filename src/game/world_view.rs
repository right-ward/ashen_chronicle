use crate::game::time;
use crate::model::GameState;
use crate::presentation::{
    CharacterView, ConditionView, HistoryEntryView, HistoryEntryViewType, ThreatView, WorldView,
};

pub(crate) fn build_view(state: &GameState) -> WorldView {
    let location = state
        .world
        .location_by_id(state.character.location_id)
        .map(|location| {
            let region_name = state
                .world
                .regions
                .iter()
                .find(|region| region.id == location.region_id)
                .map(|region| region.name.clone())
                .unwrap_or_else(|| "Unknown region".to_string());
            crate::presentation::LocationView {
                id: location.id,
                name: location.name.clone(),
                description: location.description.clone(),
                region_name,
                dangerous: location.dangerous,
            }
        });

    let campaign = state
        .campaign_content
        .clone()
        .unwrap_or_else(crate::content::load_campaign_content);

    let art = location
        .as_ref()
        .and_then(|location| campaign.location_art_for(&location.name))
        .map(str::to_string);
    let atmosphere = location.as_ref().and_then(|location| {
        campaign
            .atmospheres
            .iter()
            .find(|entry| entry.location_name == location.name)
            .map(|entry| entry.text.clone())
    });

    let conditions = state
        .character
        .conditions
        .iter()
        .map(|condition| ConditionView {
            name: condition.name.clone(),
            remaining: condition.remaining,
            penalty: condition.penalty,
            bonus: condition.bonus,
        })
        .collect();

    let threat = state.threat.active.then(|| ThreatView {
        label: state.threat.label.clone(),
        description: state.threat.description.clone(),
    });
    let history = state
        .world
        .history
        .iter()
        .rev()
        .take(12)
        .rev()
        .map(|entry| HistoryEntryView {
            day: entry.turn,
            entry_type: if entry.event_id.is_some() {
                HistoryEntryViewType::Event
            } else {
                HistoryEntryViewType::Narrative
            },
            text: entry.text.clone(),
            event_id: entry.event_id.clone(),
            location_name: entry.location_name.clone(),
            outcome: entry.outcome.clone(),
        })
        .collect();

    WorldView {
        world_name: state.world.name.clone(),
        time: time::time_display(state.world.time_points, state.world.day),
        time_points: state.world.time_points,
        day: state.world.day,
        character: CharacterView {
            name: state.character.name.clone(),
            title: state.character.title.clone(),
            hp: state.character.hp,
            max_hp: state.character.max_hp,
        },
        location,
        art,
        atmosphere,
        conditions,
        threat,
        history,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_view_preserves_world_and_character_data() {
        let state = crate::model::create_new_state(
            "Test World",
            crate::model::WorldMode::New,
            "Ash".to_string(),
            "Wanderer".to_string(),
        );

        let view = build_view(&state);

        assert_eq!(view.world_name, state.world.name);
        assert_eq!(view.time_points, state.world.time_points);
        assert_eq!(view.day, state.world.day);
        assert_eq!(view.character.name, state.character.name);
        assert_eq!(view.character.title, state.character.title);
        assert_eq!(view.character.hp, state.character.hp);
        assert_eq!(view.character.max_hp, state.character.max_hp);
        assert!(view.location.is_some());
    }
}
