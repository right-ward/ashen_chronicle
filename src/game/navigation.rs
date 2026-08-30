use crate::game::actions;
use crate::model::{EntityId, GameState, Location};
use crate::ui::set_menu_screen;
use std::io;

pub(crate) fn open(state: &mut GameState) -> io::Result<()> {
    let Some(current_location) = state.world.location_by_id(state.character.location_id) else {
        set_menu_screen(
            "WORLD NAVIGATION",
            Some("Your current location can no longer be resolved.".to_string()),
            None,
        );
        let _ = crate::ui::choose_from_list("Navigation", &["Back".to_string()], None)?;
        return Ok(());
    };

    let region_name = state
        .world
        .regions
        .iter()
        .find(|region| region.id == current_location.region_id)
        .map(|region| region.name.as_str())
        .unwrap_or("Unknown region");

    let destinations: Vec<(EntityId, String)> = current_location
        .exits
        .iter()
        .filter_map(|id| state.world.location_by_id(*id))
        .map(|location| (location.id, route_label(location)))
        .collect();

    let mut details = vec![
        format!("Current location: {}", current_location.name),
        format!("Region: {}", region_name),
    ];
    if current_location.dangerous {
        details.push("Danger: this location is dangerous.".to_string());
    }
    if !current_location.description.trim().is_empty() {
        details.push(String::new());
        details.extend(current_location.description.lines().map(str::to_string));
    }
    details.push(String::new());
    details.push(if destinations.is_empty() {
        "No known routes lead onward.".to_string()
    } else {
        "Choose a route to travel.".to_string()
    });

    let art = state
        .campaign_content
        .clone()
        .unwrap_or_else(crate::content::load_campaign_content)
        .location_art_for(&current_location.name)
        .map(str::to_string);

    set_menu_screen(
        format!("World Navigation — {}", state.character.display_name()),
        Some(details.join("\n")),
        art,
    );

    if destinations.is_empty() {
        let _ = crate::ui::choose_from_list("Routes", &["Back".to_string()], None)?;
        return Ok(());
    }

    let options: Vec<String> = destinations
        .iter()
        .map(|(_, label)| label.clone())
        .collect();
    if let Some(selection) = crate::ui::choose_from_list("Routes", &options, Some("Back"))? {
        if let Some((target_id, _)) = destinations.get(selection) {
            actions::travel_to(state, *target_id)?;
        }
    }
    Ok(())
}

fn route_label(location: &Location) -> String {
    let danger = if location.dangerous { " [DANGER]" } else { "" };
    let summary = location
        .description
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("No description available.");
    format!("→ {}{} — {}", location.name, danger, summary)
}

#[cfg(test)]
mod tests {
    use super::route_label;
    use crate::model::Location;

    #[test]
    fn route_label_marks_dangerous_locations() {
        let location = Location {
            id: 2,
            name: "The Hollow".to_string(),
            description: "A broken road disappears into the ash.".to_string(),
            region_id: 1,
            dangerous: true,
            corpse_ids: Vec::new(),
            exits: Vec::new(),
        };

        assert_eq!(
            route_label(&location),
            "→ The Hollow [DANGER] — A broken road disappears into the ash."
        );
    }

    #[test]
    fn route_label_uses_first_non_empty_description_line() {
        let location = Location {
            id: 2,
            name: "The Gate".to_string(),
            description: "\nThe old gate still stands.\nBeyond it, the road is quiet.".to_string(),
            region_id: 1,
            dangerous: false,
            corpse_ids: Vec::new(),
            exits: Vec::new(),
        };

        assert_eq!(
            route_label(&location),
            "→ The Gate — The old gate still stands."
        );
    }
}
