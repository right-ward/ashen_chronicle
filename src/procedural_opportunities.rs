use crate::content::{EventConditionContent, EventContent, EventEffectContent};
use crate::model::{GameState, Quest, QuestObjective, QuestObjectiveKind};

const GENERATED_QUEST_PREFIX: &str = "generated.quest.";
const GENERATED_EVENT_PREFIX: &str = "generated.event.";
const EVOLUTION_QUEST_PREFIX: &str = "generated.evolution.quest.";
const EVOLUTION_EVENT_PREFIX: &str = "generated.evolution.event.";
const RELATIONSHIP_MARKER: &str = "[generated relationship]";
const GENERATED_FACTION_MARKER: &str = "A generated faction shaped by";

/// Derive persistent quests and runtime events from the relationships already
/// present in a generated world. The resulting quests live in GameState, while
/// generated events are inserted into the in-memory campaign content and are
/// recreated deterministically when a saved generated world is loaded.
pub fn populate_generated_opportunities(state: &mut GameState) -> usize {
    if state.world.generation.is_none() {
        return 0;
    }

    let Some((faction_id, target_faction_id, kind, target_location_id)) =
        select_relationship_opportunity(state)
    else {
        return populate_danger_opportunity(state);
    };

    let mut added = 0;
    let quest_id = format!("{GENERATED_QUEST_PREFIX}{faction_id}.{target_faction_id}.{kind}");
    if !state
        .quests
        .iter()
        .any(|quest| quest.content_id == quest_id)
    {
        let giver_npc_id = state
            .npcs
            .iter()
            .find(|npc| npc.faction_id == Some(faction_id))
            .map(|npc| npc.id);
        if let Some(giver_npc_id) = giver_npc_id {
            let target_name = state
                .world
                .location_by_id(target_location_id)
                .map(|location| location.name.clone())
                .unwrap_or_else(|| "an unknown place".to_string());
            let faction_name = state
                .factions
                .iter()
                .find(|faction| faction.id == faction_id)
                .map(|faction| faction.name.clone())
                .unwrap_or_else(|| "a local faction".to_string());
            let target_faction_name = state
                .factions
                .iter()
                .find(|faction| faction.id == target_faction_id)
                .map(|faction| faction.name.clone())
                .unwrap_or_else(|| "a rival faction".to_string());
            let title = match kind.as_str() {
                "rivalry" => "Test the Rival's Reach",
                "alliance" => "Carry Word Between Allies",
                "trade" => "Secure the Trade Route",
                "influence" => "Gauge the Neighbour's Strength",
                "dependency" => "Trace the Missing Supply",
                _ => "Investigate a Faction's Reach",
            };
            let description = format!(
                "{faction_name} has a {kind} relationship with {target_faction_name}. "
                    "Travel to {target_name} and learn what the relationship means on the ground."
            );
            let mut quest = Quest::new(
                state.world.allocate_id(),
                quest_id,
                title,
                description,
                target_location_id,
                faction_id,
                giver_npc_id,
                "",
                format!("Token of {faction_name}"),
            );
            quest.objectives.push(QuestObjective::new(
                QuestObjectiveKind::VisitLocation,
                target_name,
                1,
            ));
            state.quests.push(quest);
            added += 1;
        }
    }

    let event_id = format!("{GENERATED_EVENT_PREFIX}{faction_id}.{target_faction_id}.{kind}");
    if ensure_generated_event(
        state,
        EventContent {
            id: event_id,
            trigger: "travel_arrival".to_string(),
            weight: 1,
            chance_percent: Some(100),
            cooldown_turns: Some(12),
            conditions: Some(EventConditionContent {
                locations: state
                    .world
                    .location_by_id(target_location_id)
                    .map(|location| vec![location.name.clone()])
                    .unwrap_or_default(),
                ..Default::default()
            }),
            effects: vec![EventEffectContent::History {
                text: format!(
                    "The signs of the {kind} between the factions are unmistakable here."
                ),
            }],
        },
    ) {
        added += 1;
    }

    added
}

fn populate_danger_opportunity(state: &mut GameState) -> usize {
    let Some(location) = state
        .world
        .locations
        .iter()
        .find(|location| location.dangerous)
        .cloned()
    else {
        return 0;
    };
    let Some(faction) = state
        .factions
        .iter()
        .find(|faction| is_generated_faction(faction))
        .cloned()
    else {
        return 0;
    };
    let Some(giver_npc_id) = state
        .npcs
        .iter()
        .find(|npc| npc.faction_id == Some(faction.id))
        .map(|npc| npc.id)
    else {
        return 0;
    };

    let quest_id = format!("{GENERATED_QUEST_PREFIX}danger.{}", location.id);
    let mut added = 0;
    if !state
        .quests
        .iter()
        .any(|quest| quest.content_id == quest_id)
    {
        let mut quest = Quest::new(
            state.world.allocate_id(),
            quest_id,
            "Scout the Dangerous Ground",
            format!(
                "The area around {} remains dangerous. Find out what is making it unsafe.",
                location.name
            ),
            location.id,
            faction.id,
            giver_npc_id,
            "",
            format!("Scout's Token of {}", faction.name),
        );
        quest.objectives.push(QuestObjective::new(
            QuestObjectiveKind::VisitLocation,
            location.name.clone(),
            1,
        ));
        state.quests.push(quest);
        added += 1;
    }

    let event_id = format!("{GENERATED_EVENT_PREFIX}danger.{}", location.id);
    if ensure_generated_event(
        state,
        EventContent {
            id: event_id,
            trigger: "travel_arrival".to_string(),
            weight: 1,
            chance_percent: Some(100),
            cooldown_turns: Some(12),
            conditions: Some(EventConditionContent {
                locations: vec![location.name.clone()],
                dangerous: Some(true),
                ..Default::default()
            }),
            effects: vec![EventEffectContent::History {
                text: format!("The danger around {} is close and immediate.", location.name),
            }],
        },
    ) {
        added += 1;
    }
    added
}

/// Advance the generated world from persistent gameplay consequences.
/// Completed generated quests pacify their target location and create one
/// deterministic follow-up opportunity at another still-dangerous location.
pub fn evolve_generated_world(state: &mut GameState) -> usize {
    if state.world.generation.is_none() {
        return 0;
    }

    let completed = state
        .quests
        .iter()
        .filter(|quest| {
            quest.completed
                && (quest.content_id.starts_with(GENERATED_QUEST_PREFIX)
                    || quest.content_id.starts_with(EVOLUTION_QUEST_PREFIX))
        })
        .map(|quest| (quest.content_id.clone(), quest.target_location_id, quest.faction_id))
        .collect::<Vec<_>>();

    let mut changed = 0;
    for (quest_id, target_location_id, faction_id) in completed {
        let location_name = {
            let Some(location) = state.world.location_by_id_mut(target_location_id) else {
                continue;
            };
            if !location.dangerous {
                continue;
            }
            let location_name = location.name.clone();
            location.dangerous = false;
            location_name
        };
        changed += 1;
        state.world.record_history(
            state.character.turn,
            format!(
                "The danger at {} receded after {} was resolved.",
                location_name, quest_id
            ),
        );

        if let Some(faction) = state
            .factions
            .iter_mut()
            .find(|faction| faction.id == faction_id)
        {
            faction.memory.push(format!(
                "[world evolution] {} became safer after {} was completed.",
                location_name, quest_id
            ));
        }

        generate_follow_up_opportunity(state, &quest_id, faction_id, target_location_id);
    }

    changed
}

fn generate_follow_up_opportunity(
    state: &mut GameState,
    source_quest_id: &str,
    faction_id: u64,
    completed_location_id: u64,
) {
    let Some(target) = state
        .world
        .locations
        .iter()
        .find(|location| location.dangerous && location.id != completed_location_id)
        .cloned()
    else {
        return;
    };
    let Some(giver_npc_id) = state
        .npcs
        .iter()
        .find(|npc| npc.faction_id == Some(faction_id))
        .map(|npc| npc.id)
    else {
        return;
    };
    let quest_id = format!("{EVOLUTION_QUEST_PREFIX}{source_quest_id}");
    if state
        .quests
        .iter()
        .any(|quest| quest.content_id == quest_id)
    {
        return;
    }
    let faction_name = state
        .factions
        .iter()
        .find(|faction| faction.id == faction_id)
        .map(|faction| faction.name.clone())
        .unwrap_or_else(|| "the faction".to_string());
    let mut quest = Quest::new(
        state.world.allocate_id(),
        quest_id.clone(),
        "Follow the Opening",
        format!(
            "With {} safer, {} can redirect attention toward {}.",
            state
                .world
                .locations
                .iter()
                .find(|location| location.id == completed_location_id)
                .map(|location| location.name.as_str())
                .unwrap_or("the settled ground"),
            faction_name,
            target.name
        ),
        target.id,
        faction_id,
        giver_npc_id,
        "",
        format!("Renewed Token of {faction_name}"),
    );
    quest.objectives.push(QuestObjective::new(
        QuestObjectiveKind::VisitLocation,
        target.name.clone(),
        1,
    ));
    state.quests.push(quest);

    let event_id = format!("{EVOLUTION_EVENT_PREFIX}{source_quest_id}");
    let _ = ensure_generated_event(
        state,
        EventContent {
            id: event_id,
            trigger: "travel_arrival".to_string(),
            weight: 1,
            chance_percent: Some(100),
            cooldown_turns: Some(12),
            conditions: Some(EventConditionContent {
                locations: vec![target.name.clone()],
                dangerous: Some(true),
                ..Default::default()
            }),
            effects: vec![EventEffectContent::History {
                text: format!(
                    "With the earlier danger reduced, attention turns toward {}.",
                    target.name
                ),
            }],
        },
    );
}

fn ensure_generated_event(state: &mut GameState, event: EventContent) -> bool {
    let Some(content) = state.campaign_content.as_mut() else {
        return false;
    };
    if content.events.iter().any(|candidate| candidate.id == event.id) {
        return false;
    }
    content.events.push(event);
    true
}

fn select_relationship_opportunity(
    state: &GameState,
) -> Option<(u64, u64, String, u64)> {
    let mut generated_factions = state
        .factions
        .iter()
        .filter(|faction| is_generated_faction(faction))
        .collect::<Vec<_>>();
    generated_factions.sort_by_key(|faction| faction.id);

    for faction in generated_factions {
        let mut relationships = faction
            .memory
            .iter()
            .filter(|entry| entry.starts_with(RELATIONSHIP_MARKER))
            .collect::<Vec<_>>();
        relationships.sort();
        for relationship in relationships {
            let Some((kind, target_name)) = parse_relationship(relationship) else {
                continue;
            };
            let Some(target_faction) = state
                .factions
                .iter()
                .find(|candidate| candidate.name == target_name)
            else {
                continue;
            };
            let Some(target_location_id) = state
                .npcs
                .iter()
                .find(|npc| npc.faction_id == Some(target_faction.id))
                .map(|npc| npc.location_id)
                .or_else(|| location_for_faction_name(state, &target_faction.name))
            else {
                continue;
            };
            return Some((
                faction.id,
                target_faction.id,
                kind.to_string(),
                target_location_id,
            ));
        }
    }
    None
}

fn parse_relationship(entry: &str) -> Option<(&str, &str)> {
    let body = entry.strip_prefix(RELATIONSHIP_MARKER)?.trim_start();
    let (kind, rest) = body.split_once(" with ")?;
    let target = rest.split(" (strength").next()?.trim();
    if kind.is_empty() || target.is_empty() {
        None
    } else {
        Some((kind, target))
    }
}

fn location_for_faction_name(state: &GameState, faction_name: &str) -> Option<u64> {
    let region_name = faction_name.rsplit_once(" of ")?.1;
    let region_id = state
        .world
        .regions
        .iter()
        .find(|region| region.name == region_name)
        .map(|region| region.id)?;
    state
        .world
        .locations
        .iter()
        .find(|location| location.region_id == region_id)
        .map(|location| location.id)
}

fn is_generated_faction(faction: &crate::model::Faction) -> bool {
    faction
        .memory
        .iter()
        .any(|entry| entry.starts_with(GENERATED_FACTION_MARKER))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::load_campaign_content;
    use crate::model::{GameState, WorldGenerationMetadata, WorldMode};
    use crate::procedural::{generate_world, place_authored_content, WorldGenerationConfig};
    use crate::procedural_entities::populate_generated_entities;
    use crate::procedural_relationships::populate_generated_relationships;

    fn generated_state() -> GameState {
        let content = load_campaign_content();
        let seed = 9090;
        let config = WorldGenerationConfig::default();
        let mut world = generate_world("Ashen", seed, config);
        world.mode = WorldMode::New;
        world.generation = Some(WorldGenerationMetadata {
            seed,
            region_count: config.region_count,
            location_count: config.location_count,
            extra_edges: config.extra_edges,
        });
        place_authored_content(&mut world, &content);
        let character = world.spawn_character("Test".into(), "Warden".into());
        GameState {
            world,
            character,
            threat: Default::default(),
            corpses: Vec::new(),
            factions: Vec::new(),
            npcs: Vec::new(),
            quests: Vec::new(),
            last_announced_location_id: None,
            rng_state: 0,
            campaign_content: Some(content),
        }
    }

    fn prepared_state() -> GameState {
        let mut state = generated_state();
        let content = state.campaign_content.clone().expect("content should exist");
        populate_generated_entities(&mut state, &content);
        crate::procedural_authored::integrate_authored_content(&mut state, &content);
        populate_generated_relationships(&mut state);
        state
    }

    #[test]
    fn generated_world_gets_deterministic_quest_and_event() {
        let mut first = prepared_state();
        let mut second = prepared_state();
        assert!(populate_generated_opportunities(&mut first) > 0);
        assert!(populate_generated_opportunities(&mut second) > 0);
        let first_quests = first
            .quests
            .iter()
            .filter(|quest| quest.content_id.starts_with(GENERATED_QUEST_PREFIX))
            .map(|quest| quest.content_id.clone())
            .collect::<Vec<_>>();
        let second_quests = second
            .quests
            .iter()
            .filter(|quest| quest.content_id.starts_with(GENERATED_QUEST_PREFIX))
            .map(|quest| quest.content_id.clone())
            .collect::<Vec<_>>();
        assert_eq!(first_quests, second_quests);
        assert!(first
            .campaign_content
            .as_ref()
            .unwrap()
            .events
            .iter()
            .any(|event| event.id.starts_with(GENERATED_EVENT_PREFIX)));
    }

    #[test]
    fn generated_quest_references_valid_runtime_entities() {
        let mut state = prepared_state();
        populate_generated_opportunities(&mut state);
        let quest = state
            .quests
            .iter()
            .find(|quest| quest.content_id.starts_with(GENERATED_QUEST_PREFIX))
            .expect("generated quest should exist");
        assert!(state.world.location_by_id(quest.target_location_id).is_some());
        assert!(state.factions.iter().any(|faction| faction.id == quest.faction_id));
        assert!(state.npcs.iter().any(|npc| npc.id == quest.giver_npc_id));
        assert!(!quest.objectives.is_empty());
    }

    #[test]
    fn completed_generated_quest_evolves_world_and_creates_follow_up() {
        let mut state = prepared_state();
        populate_generated_opportunities(&mut state);
        let index = state
            .quests
            .iter()
            .position(|quest| quest.content_id.starts_with(GENERATED_QUEST_PREFIX))
            .expect("generated quest should exist");
        let target_id = state.quests[index].target_location_id;
        state.quests[index].completed = true;
        state.quests[index].reward_claimed = true;
        assert!(state.world.location_by_id(target_id).unwrap().dangerous);
        assert_eq!(evolve_generated_world(&mut state), 1);
        assert!(!state.world.location_by_id(target_id).unwrap().dangerous);
        assert!(state
            .quests
            .iter()
            .any(|quest| quest.content_id.starts_with(EVOLUTION_QUEST_PREFIX)));
        assert_eq!(evolve_generated_world(&mut state), 0);
    }

    #[test]
    fn generated_quest_and_evolution_survive_serialization() {
        let mut state = prepared_state();
        populate_generated_opportunities(&mut state);
        let quest_index = state
            .quests
            .iter()
            .position(|quest| quest.content_id.starts_with(GENERATED_QUEST_PREFIX))
            .expect("generated quest should exist");
        state.quests[quest_index].completed = true;
        evolve_generated_world(&mut state);
        let serialized = serde_json::to_vec(&state).expect("state should serialize");
        let restored: GameState =
            serde_json::from_slice(&serialized).expect("state should deserialize");
        assert!(restored
            .quests
            .iter()
            .any(|quest| quest.content_id.starts_with(GENERATED_QUEST_PREFIX)));
        assert!(restored
            .quests
            .iter()
            .any(|quest| quest.content_id.starts_with(EVOLUTION_QUEST_PREFIX)));
    }
}
