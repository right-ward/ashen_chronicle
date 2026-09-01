use crate::game::actions;
use crate::model::GameState;
use crate::presentation::{LocationView, NavigationView};
use crate::ui::{choose_from_list, set_menu_screen};
use std::io;

pub(crate) fn open(state: &mut GameState) -> io::Result<()> {
    let view = build_view(state);

    let Some(current_location) = &view.current_location else {
        set_menu_screen(
            "WORLD NAVIGATION",
            Some("Your current location can no longer be resolved.".to_string()),
            None,
        );
        let _ = choose_from_list("Navigation", &["Back".to_string()], None)?;
        return Ok(());
    };

    let mut details = vec![
        format!("Current location: {}", current_location.name),
        format!("Region: {}", current_location.region_name),
    ];
    if current_location.dangerous {
        details.push("You sense the danger hiding in this location.".to_string());
    }
    if !current_location.description.trim().is_empty() {
        details.push(String::new());
        details.extend(current_location.description.lines().map(str::to_string));
    }
    details.push(String::new());
    details.push(if view.destinations.is_empty() {
        "No known routes lead onward.".to_string()
    } else {
        "Choose a route to travel.".to_string()
    });

    set_menu_screen("World Navigation", Some(details.join("\n")), view.art.clone());

    if view.destinations.is_empty() {
        let _ = choose_from_list("Routes", &["Back".to_string()], None)?;
        return Ok(());
    }

    let options: Vec<String> = view
        .destinations
        .iter()
        .map(|location| location.name.clone())
        .collect();
    if let Some(selection) = choose_from_list("Routes", &options, Some("Back"))? {
        if let Some(destination) = view.destinations.get(selection) {
            actions::travel_to(state, destination.id)?;
        }
    }
    Ok(())
}

fn build_view(state: &GameState) -> NavigationView {
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
