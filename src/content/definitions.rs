use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignContent {
    pub version: u32,
    pub world: WorldContent,
    #[serde(default)]
    pub factions: Vec<FactionContent>,
    #[serde(default)]
    pub npcs: Vec<NpcContent>,
    #[serde(default)]
    pub quests: Vec<QuestContent>,
    #[serde(default)]
    pub encounters: Vec<EncounterContent>,
    #[serde(default)]
    pub atmospheres: Vec<LocationAtmosphere>,
    #[serde(default)]
    pub item_visuals: Vec<ItemVisualContent>,
    #[serde(default)]
    pub events: Vec<EventContent>,
}

impl CampaignContent {
    pub fn is_some(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldContent {
    pub region: RegionContent,
    pub locations: Vec<LocationContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionContent {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationContent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub dangerous: bool,
    #[serde(default)]
    pub exits: Vec<String>,
    #[serde(default)]
    pub scene_art: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionContent {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcContent {
    pub id: String,
    pub name: String,
    pub title: String,
    pub location_name: String,
    #[serde(default)]
    pub faction_name: Option<String>,
    #[serde(default)]
    pub memory: Vec<String>,
    #[serde(default)]
    pub portrait: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestContent {
    pub id: String,
    pub title: String,
    pub description: String,
    pub location_name: String,
    pub faction_name: String,
    pub giver_npc_name: String,
    pub required_item_name: String,
    pub reward_item_name: String,
}
