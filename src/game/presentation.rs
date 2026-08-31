use crate::game::time::time_display;
use crate::model::GameState;
use crate::ui::{set_dashboard, Dashboard};

pub(crate) fn render_state(state: &GameState) {
    let world = &state.world;
    let character = &state.character;
    let location = world.location_by_id(character.location_id);
    let condition_line = if character.conditions.is_empty() {
        None
    } else {
        Some(format!(
            "Condition: {}",
            character
                .conditions
                .iter()
                .map(|c| format!("{} ({})", c.name, c.remaining))
                .collect::<Vec<_>>()
                .join(", ")
        ))
    };
    let threat_line = if state.threat.active {
        Some(format!("Threat: {}", state.threat.label))
    } else {
        None
    };
    let dashboard = Dashboard {
        world_name: world.name.clone(),
        hp: character.hp,
        max_hp: character.max_hp,
        enemy_name: None,
        enemy_hp: None,
        enemy_max_hp: None,
        time_display: time_display(world.time_points, world.day),
        condition_line,
        location_name: location.map(|location| format!("~ {} ~", location.name)),
        location_description: location.map(|location| location.description.clone()),
        danger_line: location.and_then(|location| {
            if location.dangerous {
                Some("You feel the danger.".to_string())
            } else {
                None
            }
        }),
        threat_line,
        action_hint: Some("Arrows / Enter / Esc".to_string()),
    };
    set_dashboard(dashboard);
}
