use crate::game::{character, interactions, state_effects};
use crate::model::{Corpse, GameState, Item};
use crate::presentation::{
    ChoiceView, ItemView, RemainsEntryView, RemainsResultView, RemainsView, ScreenView,
};
use crate::ui::{choose_from_list, narrate, pause};
use std::mem;
use std::time::{SystemTime, UNIX_EPOCH};

macro_rules! println {
    () => {
        crate::ui::line("");
    };
    ($($arg:tt)*) => {
        crate::ui::line(&format!($($arg)*))
    };
}

fn item_view(item: &Item) -> ItemView {
    ItemView {
        id: item.id,
        name: item.name.clone(),
        description: item.description.clone(),
    }
}

pub(crate) fn search_remains(state: &mut GameState) -> std::io::Result<()> {
    let location_id = state.character.location_id;
    let indices: Vec<usize> = state
        .corpses
        .iter()
        .enumerate()
        .filter(|(_, corpse)| corpse.location_id == location_id && !corpse.inventory.is_empty())
        .map(|(index, _)| index)
        .collect();
    if indices.is_empty() {
        crate::ui::show_screen_view(&ScreenView {
            title: "REMAINS".to_string(),
            subtitle: Some("There are no remains worth searching here.".to_string()),
            ..Default::default()
        });
        pause();
        return Ok(());
    }

    let location_name = state
        .world
        .location_by_id(location_id)
        .map(|location| location.name.clone())
        .unwrap_or_else(|| "this place".to_string());
    let remains_view = RemainsView {
        location_name: location_name.clone(),
        remains: indices
            .iter()
            .map(|index| {
                let corpse = &state.corpses[*index];
                RemainsEntryView {
                    id: corpse.id,
                    label: corpse_label(corpse),
                    former_name: corpse.former_name.clone(),
                    former_title: corpse.former_title.clone(),
                    scavenged: corpse.scavenged,
                    items: corpse.inventory.iter().map(item_view).collect(),
                }
            })
            .collect(),
    };
    let options = remains_view
        .remains
        .iter()
        .map(|remains| remains.label.clone())
        .collect::<Vec<_>>();
    let choice_view = ChoiceView {
        screen: ScreenView {
            title: "REMAINS".to_string(),
            subtitle: Some(format!("Search the remains at {}.", remains_view.location_name)),
            ..Default::default()
        },
        prompt: "Search which remains?".to_string(),
        options,
        back_label: Some("Back".to_string()),
    };

    if let Some(choice) = crate::ui::choose_screen_view(&choice_view)? {
        let corpse_index = indices[choice];
        let (former_name, former_title, items, corpse_id) = {
            let corpse = &mut state.corpses[corpse_index];
            let items = mem::take(&mut corpse.inventory);
            corpse.scavenged = true;
            (
                corpse.former_name.clone(),
                corpse.former_title.clone(),
                items,
                corpse.id,
            )
        };
        state_effects::advance_time(state, 1);
        println!("You search the remains at {}.", location_name);
        if items.is_empty() {
            println!("Nothing useful remains.");
            state.world.record_history(
                state.character.turn,
                format!(
                    "{} searched the remains of {} the {} at {}.",
                    state.character.display_name(),
                    former_name,
                    former_title,
                    location_name
                ),
            );
            pause();
            return Ok(());
        }

        let item_names: Vec<String> = items.iter().map(|item| item.name.clone()).collect();
        for item in &items {
            notify_item_gain(state, item);
            interactions::grant_reward_reputation(state, item);
        }
        state.character.inventory.extend(items);

        let hidden_item = if state.character.effective_insight() >= 2 && item_names.len() < 3 {
            let tick = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0);
            if tick.is_multiple_of(2) {
                let hidden = Item {
                    id: state.world.allocate_id(),
                    name: "Ashen Note".to_string(),
                    description: "A scrap of writing that might reveal something about the life that ended here.".to_string(),
                };
                notify_item_gain(state, &hidden);
                state.character.inventory.push(hidden.clone());
                Some(hidden)
            } else {
                None
            }
        } else {
            None
        };
        if hidden_item.is_some() {
            println!("Your insight uncovers something the hurried would have missed.");
        }

        character::gain_experience(
            state,
            (5 + state.character.effective_insight())
                .try_into()
                .unwrap(),
        );

        let result_view = RemainsResultView {
            location_name: location_name.clone(),
            former_name: former_name.clone(),
            former_title: former_title.clone(),
            items: state
                .character
                .inventory
                .iter()
                .filter(|item| item_names.contains(&item.name))
                .map(item_view)
                .collect(),
            hidden_item: hidden_item.as_ref().map(item_view),
            notes: vec![
                "Feel like a deja-vu.".to_string(),
                "You feel as if they were once yours. Though, These items can be inherited, Their memories cannot.".to_string(),
            ],
        };
        println!("{}", result_view.notes[0]);
        println!("{}", result_view.notes[1]);
        println!("Recovered {}", item_names.join(", "));
        state.character.turn += 1;
        state.world.record_history(
            state.character.turn,
            format!(
                "{} searched the remains of {} the {} at {}.",
                state.character.display_name(),
                former_name,
                former_title,
                location_name
            ),
        );
        if let Some(location) = state.world.location_by_id_mut(location_id) {
            if !location.corpse_ids.contains(&corpse_id) {
                location.corpse_ids.push(corpse_id);
            }
        }
        narrate("You gather what can still be carried.");
    }
    Ok(())
}

pub(crate) fn notify_item_gain(state: &GameState, item: &Item) {
    println!("You gain: {}", item.name);
    println!("{}", item.description);
    if let Some(art) = state
        .campaign_content
        .as_ref()
        .and_then(|content| content.item_art_for(&item.name))
    {
        println!("");
        println!("{}", art);
    }
}

fn corpse_label(corpse: &Corpse) -> String {
    if corpse.former_name.is_empty() {
        "Unidentified remains".to_string()
    } else if corpse.scavenged {
        format!(
            "{} the {} (searched)",
            corpse.former_name, corpse.former_title
        )
    } else {
        format!("{} the {}", corpse.former_name, corpse.former_title)
    }
}

pub(crate) fn mark_character_dead(state: &mut GameState, cause: String, location_name: &str) {
    if !state.character.alive {
        return;
    }
    state.character.alive = false;
    state.character.hp = 0;
    let corpse = create_corpse(state, cause.clone());
    let dropped_count = corpse.inventory.len();
    state.corpses.push(corpse.clone());
    if let Some(location) = state.world.location_by_id_mut(corpse.location_id) {
        if !location.corpse_ids.contains(&corpse.id) {
            location.corpse_ids.push(corpse.id);
        }
    }
    let character_name = state.character.display_name();
    state.world.record_history(
        state.character.turn,
        format!("{} died at {} ({cause}).", character_name, location_name),
    );
    interactions::update_faction_memory_for_location(
        state,
        corpse.location_id,
        format!("{} died at {}.", character_name, location_name),
    );
    if dropped_count > 0 {
        println!("{} item(s) were left behind.", dropped_count);
    }
}

fn create_corpse(state: &mut GameState, epitaph: String) -> Corpse {
    let corpse_id = state.world.allocate_id();
    let location_id = state.character.location_id;
    let inventory = mem::take(&mut state.character.inventory);
    Corpse {
        id: corpse_id,
        former_name: state.character.name.clone(),
        former_title: state.character.title.clone(),
        location_id,
        turn_of_death: state.character.turn,
        inventory,
        epitaph,
        scavenged: false,
    }
}
