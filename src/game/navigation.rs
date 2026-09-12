use crate::model::GameState;
use crate::presentation::{LocationView, NavigationView};

pub(crate) fn build_view(state: &GameState) -> NavigationView {
    let Some(current_location) = state.world.location_by_id(state.character.location_id) else {
        return NavigationView::default();
    };

    let current = location_view(state, current_location);
    let destinations = current_location
        .exits
        .iter()
        .filter_map(|id| state.world.location_by_id(*id))
        .map(|location| location_view(state, location))
        .collect();
    let art = state
        .campaign_content
        .clone()
        .unwrap_or_else(crate::content::load_campaign_content)
        .location_art_for(&current.name)
        .map(str::to_string);

    NavigationView {
        current_location: Some(current),
        destinations,
        art,
    }
}

fn location_view(state: &GameState, location: &crate::model::Location) -> LocationView {
    let region_name = state
        .world
        .regions
        .iter()
        .find(|region| region.id == location.region_id)
        .map(|region| region.name.clone())
        .unwrap_or_else(|| "Unknown region".to_string());

    LocationView {
        id: location.id,
        name: location.name.clone(),
        description: location.description.clone(),
        region_name,
        dangerous: location.dangerous,
    }
}
