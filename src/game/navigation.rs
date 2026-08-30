use crate::game::actions;
use crate::model::{EntityId, GameState};
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
        .map(|location| (location.id, location.name.clone()))
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

    set_menu_screen("World Navigation", Some(details.join("\n")), art);

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

#[cfg(test)]
mod tests {
    use super::open;
    use crate::model::{EntityId, GameState};
    use std::mem;

    #[allow(dead_code)]
    fn _type_check(_: fn(&mut GameState) -> std::io::Result<()>) {}
    #[allow(dead_code)]
    fn _id_type_check(_: EntityId) {}
    #[allow(dead_code)]
    fn _unused_to_silence_warning<T>(_: T) {
        let _ = mem::size_of::<usize>();
    }

    #[test]
    fn navigation_open_signature_remains_io_result() {
        _type_check(open);
    }
}
