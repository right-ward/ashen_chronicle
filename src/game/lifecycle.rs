use crate::game::validate_loaded_state;
use crate::model::{create_inherited_state, create_new_state, GameState, WorldMode};
use crate::persistence::{character_save_path, find_save_files, legacy_save_path, load_game};
use crate::presentation::{ChoiceView, DeathView, FactionView, ItemView, ScreenView};
use crate::ui::{pause, prompt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn show_screen(view: &ScreenView) {
    crate::ui::show_screen_view(view);
}

fn choose(view: &ChoiceView) -> std::io::Result<Option<usize>> {
    crate::ui::choose_screen_view(view)
}

pub(crate) fn start_screen() -> std::io::Result<Option<(GameState, PathBuf)>> {
    let current_dir = PathBuf::from(".");
    loop {
        let save_files = find_save_files(&current_dir)?;
        let legacy_path = legacy_save_path(&current_dir);
        let has_saves = !save_files.is_empty() || legacy_path.exists();
        let tick = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.subsec_nanos() as usize)
            .unwrap_or(0);
        let subtitle = match tick % 4 {
            0 => "The road is quiet. Something is listening.",
            1 => "The old gods are silent. The stones remember.",
            2 => "Only ash remains where the fire once lived.",
            _ => "You are not the first to walk this road.",
        };
        let view = ScreenView {
            title: "THE ASHEN CHRONICLE".to_string(),
            subtitle: Some(subtitle.to_string()),
            art: None,
            body: Vec::new(),
        };
        let mut options = vec!["New Game".to_string()];
        if has_saves {
            options.push("Load Game".to_string());
        }
        options.push("Quit".to_string());
        let choice_view = ChoiceView {
            screen: view,
            prompt: "Begin".to_string(),
            options,
            back_label: None,
        };
        let Some(choice) = choose(&choice_view)? else {
            continue;
        };

        match choice_view.options[choice].as_str() {
            "New Game" => {
                show_screen(&ScreenView {
                    title: "NEW GAME".to_string(),
                    subtitle: Some(
                        "Begin a new life in a world that has yet to remember you.".to_string(),
                    ),
                    ..Default::default()
                });
                let state = create_from_prompts(WorldMode::New)?;
                let save_path = character_save_path(&current_dir, &state.character.name);
                return Ok(Some((state, save_path)));
            }
            "Load Game" => {
                if let Some(result) = load_screen(&current_dir, save_files, legacy_path)? {
                    return Ok(Some(result));
                }
            }
            "Quit" => return Ok(None),
            _ => {}
        }
    }
}

fn load_screen(
    current_dir: &Path,
    mut save_files: Vec<PathBuf>,
    legacy_path: PathBuf,
) -> std::io::Result<Option<(GameState, PathBuf)>> {
    if legacy_path.exists() && !save_files.iter().any(|path| path == &legacy_path) {
        save_files.push(legacy_path);
    }
    if save_files.is_empty() {
        return Ok(None);
    }

    let options = save_files
        .iter()
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Unknown save")
                .to_string()
        })
        .collect::<Vec<_>>();
    let choice_view = ChoiceView {
        screen: ScreenView {
            title: "LOAD GAME".to_string(),
            subtitle: Some("Choose a life to continue.".to_string()),
            ..Default::default()
        },
        prompt: "Saved lives".to_string(),
        options,
        back_label: Some("Back".to_string()),
    };
    let Some(choice) = choose(&choice_view)? else {
        return Ok(None);
    };

    let path = &save_files[choice];
    match load_game(path) {
        Ok(state) => {
            let warnings = validate_loaded_state(&state);
            let save_path = character_save_path(current_dir, &state.character.name);
            if warnings.is_empty() {
                return Ok(Some((state, save_path)));
            }
            show_screen(&ScreenView {
                title: "LOAD GAME".to_string(),
                subtitle: Some(format!(
                    "Save loaded with {} warning(s). The game will continue, but the save should be reviewed.",
                    warnings.len()
                )),
                ..Default::default()
            });
            pause();
            Ok(Some((state, save_path)))
        }
        Err(err) => {
            show_screen(&ScreenView {
                title: "LOAD GAME".to_string(),
                subtitle: Some(format!("Could not load {}: {}", path.display(), err)),
                ..Default::default()
            });
            pause();
            Ok(None)
        }
    }
}

fn create_from_prompts(mode: WorldMode) -> std::io::Result<GameState> {
    let world_name = if matches!(&mode, WorldMode::New) {
        let input = prompt("Name the world [The Ashen Crown]: ")?;
        if input.is_empty() {
            "The Ashen Crown".to_string()
        } else {
            input
        }
    } else {
        "The Ashen Crown".to_string()
    };
    let character_name = prompt("Character name: ")?;
    let title = prompt("Character title [Ash Walker]: ")?;
    let character_name = if character_name.is_empty() {
        "Wanderer".to_string()
    } else {
        character_name
    };
    let title = if title.is_empty() {
        "Ash Walker".to_string()
    } else {
        title
    };
    Ok(create_new_state(&world_name, mode, character_name, title))
}

fn create_inherited_from_world(state: &GameState) -> std::io::Result<GameState> {
    let character_name = prompt("New character name: ")?;
    let title = prompt("New character title [Ash Walker]: ")?;
    let character_name = if character_name.is_empty() {
        "Heir".to_string()
    } else {
        character_name
    };
    let title = if title.is_empty() {
        "Ash Walker".to_string()
    } else {
        title
    };
    Ok(create_inherited_state(state, character_name, title))
}

pub(crate) fn quit_screen() -> std::io::Result<bool> {
    let tick = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos() as usize)
        .unwrap_or(0);
    let (line, leave_choice, stay_choice) = match tick % 3 {
        0 => (
            "The road ends here.\nFor tonight, anyway.",
            "Let the ashes take it.",
            "Not yet. The night has more to say.",
        ),
        1 => (
            "The fire is dying.\nYour story does not have to.",
            "Close the book.",
            "Turn the page.",
        ),
        _ => (
            "Night has swallowed the road.\nOnly your footprints remain.",
            "Leave them to the dark.",
            "Keep walking.",
        ),
    };
    let view = ChoiceView {
        screen: ScreenView {
            title: "LEAVE?".to_string(),
            subtitle: Some(line.to_string()),
            ..Default::default()
        },
        prompt: String::new(),
        options: vec![leave_choice.to_string(), stay_choice.to_string()],
        back_label: None,
    };
    match choose(&view)? {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        _ => Ok(false),
    }
}

pub(crate) fn death_screen(state: &mut GameState) -> std::io::Result<bool> {
    let death_view = build_death_view(state);
    show_screen(&death_view.screen);
    pause();

    let choice_view = ChoiceView {
        screen: ScreenView {
            title: "WHAT REMAINS?".to_string(),
            subtitle: Some(death_view.memory_note.clone()),
            body: death_view.screen.body.clone(),
            art: death_view.screen.art.clone(),
        },
        prompt: "What remains?".to_string(),
        options: vec![
            "Create a new world".to_string(),
            "Inherit this world with a new character".to_string(),
            "Quit".to_string(),
        ],
        back_label: None,
    };

    match choose(&choice_view)? {
        Some(0) => {
            show_screen(&ScreenView {
                title: "NEW GAME".to_string(),
                subtitle: Some(
                    "A new life begins, but this world remembers what happened here.".to_string(),
                ),
                ..Default::default()
            });
            *state = create_from_prompts(WorldMode::New)?;
            Ok(true)
        }
        Some(1) => {
            show_screen(&ScreenView {
                title: "INHERIT THIS WORLD".to_string(),
                subtitle: Some(
                    "The next life will inherit the world, not the memories.".to_string(),
                ),
                ..Default::default()
            });
            *state = create_inherited_from_world(state)?;
            Ok(true)
        }
        Some(2) => quit_screen(),
        _ => Ok(false),
    }
}

fn build_death_view(state: &GameState) -> DeathView {
    let character = crate::presentation::CharacterView {
        name: state.character.name.clone(),
        title: state.character.title.clone(),
        hp: state.character.hp,
        max_hp: state.character.max_hp,
    };
    let location_name = state
        .world
        .location_by_id(state.character.location_id)
        .map(|location| location.name.clone())
        .unwrap_or_else(|| "an unknown place".to_string());
    let deeds = state
        .world
        .history
        .iter()
        .filter(|entry| {
            entry.text.contains(&character.display_name()) && entry.text.contains("completed ")
        })
        .map(|entry| entry.text.clone())
        .take(5)
        .collect::<Vec<_>>();
    let faction_standing = state
        .factions
        .iter()
        .map(|faction| FactionView {
            name: faction.name.clone(),
            reputation: faction.reputation,
            memories: Vec::new(),
        })
        .collect::<Vec<_>>();
    let dropped_items = state
        .corpses
        .last()
        .map(|corpse| {
            corpse
                .inventory
                .iter()
                .map(|item| ItemView {
                    id: item.id,
                    name: item.name.clone(),
                    description: item.description.clone(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut body = Vec::new();
    body.push(format!(
        "{} died at {} on turn {}.",
        character.display_name(), location_name, state.character.turn
    ));
    body.push(String::new());
    body.push("Deeds remembered:".to_string());
    if deeds.is_empty() {
        body.push("  None recorded.".to_string());
    } else {
        body.extend(deeds.iter().map(|deed| format!("  - {deed}")));
    }
    body.push(String::new());
    body.push("Faction standing at death:".to_string());
    if faction_standing.is_empty() {
        body.push("  None recorded.".to_string());
    } else {
        body.extend(
            faction_standing
                .iter()
                .map(|faction| format!("  - {} {:+}", faction.name, faction.reputation)),
        );
    }
    body.push(String::new());
    body.push("What remains on the body:".to_string());
    if dropped_items.is_empty() {
        body.push("  Nothing worth carrying.".to_string());
    } else {
        body.push(format!(
            "  {}",
            dropped_items
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    DeathView {
        screen: ScreenView {
            title: "DEATH".to_string(),
            subtitle: Some("The body is still. The world is not.".to_string()),
            art: None,
            body,
        },
        character,
        location_name,
        turn: state.character.turn,
        deeds,
        faction_standing,
        dropped_items,
        memory_note: "The next life will know none of this as memory. It can only be discovered.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::build_death_view;
    use crate::model::{create_new_state, WorldMode};

    #[test]
    fn death_view_contains_life_summary() {
        let state = create_new_state(
            "Test World",
            WorldMode::New,
            "Ash".to_string(),
            "Wanderer".to_string(),
        );
        let view = build_death_view(&state);
        assert_eq!(view.character.display_name(), "Ash the Wanderer");
        assert!(view.screen.body.iter().any(|line| line.contains("died at")));
        assert!(view.screen.body.iter().any(|line| line == "Deeds remembered:"));
        assert_eq!(
            view.memory_note,
            "The next life will know none of this as memory. It can only be discovered."
        );
    }
}
