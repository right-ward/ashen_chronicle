//! Presentation-facing data models shared by frontend renderers.
//!
//! These types intentionally contain no ratatui/crossterm types and no gameplay
//! behavior. Gameplay and screen modules can translate authoritative game state
//! into these models, while a frontend is responsible for rendering them.
#![allow(dead_code)] // temporary to avoid CI warnings

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CharacterView {
    pub name: String,
    pub title: String,
    pub hp: i32,
    pub max_hp: i32,
}

impl CharacterView {
    pub(crate) fn display_name(&self) -> String {
        if self.title.is_empty() {
            self.name.clone()
        } else {
            format!("{} the {}", self.name, self.title)
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ItemView {
    pub id: u64,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LocationView {
    pub name: String,
    pub description: String,
    pub region_name: String,
    pub dangerous: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ThreatView {
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum HistoryEntryViewType {
    #[default]
    Narrative,
    Event,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct HistoryEntryView {
    pub day: u32,
    pub entry_type: HistoryEntryViewType,
    pub text: String,
    pub location_name: Option<String>,
    pub outcome: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct FactionView {
    pub name: String,
    pub reputation: i32,
    pub memories: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct NpcView {
    pub name: String,
    pub title: String,
    pub faction_name: Option<String>,
}

impl NpcView {
    pub(crate) fn display_name(&self) -> String {
        if self.title.is_empty() {
            self.name.clone()
        } else {
            format!("{} the {}", self.name, self.title)
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct QuestObjectiveView {
    pub label: String,
    pub progress: u32,
    pub required: u32,
    pub completed: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct QuestView {
    pub title: String,
    pub description: String,
    pub objectives: Vec<QuestObjectiveView>,
    pub completed: bool,
    pub reward_claimed: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CombatantView {
    pub name: String,
    pub current_hp: i32,
    pub max_hp: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct WorldView {
    pub world_name: String,
    pub time: String,
    pub character: CharacterView,
    pub location: Option<LocationView>,
    pub threat: Option<ThreatView>,
    pub history: Vec<HistoryEntryView>,
}

#[cfg(test)]
mod tests {
    use super::{CharacterView, NpcView, WorldView};

    #[test]
    fn display_name_uses_title_when_present() {
        let character = CharacterView {
            name: "Ash".to_string(),
            title: "Wanderer".to_string(),
            ..Default::default()
        };
        let npc = NpcView {
            name: "Mara".to_string(),
            title: "Keeper".to_string(),
            ..Default::default()
        };

        assert_eq!(character.display_name(), "Ash the Wanderer");
        assert_eq!(npc.display_name(), "Mara the Keeper");
    }

    #[test]
    fn display_name_omits_empty_title() {
        let character = CharacterView {
            name: "Ash".to_string(),
            ..Default::default()
        };
        let npc = NpcView {
            name: "Mara".to_string(),
            ..Default::default()
        };

        assert_eq!(character.display_name(), "Ash");
        assert_eq!(npc.display_name(), "Mara");
    }

    #[test]
    fn world_view_defaults_to_empty_frontend_data() {
        let view = WorldView::default();

        assert!(view.world_name.is_empty());
        assert!(view.time.is_empty());
        assert!(view.location.is_none());
        assert!(view.threat.is_none());
        assert!(view.history.is_empty());
    }
}
