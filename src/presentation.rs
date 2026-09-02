//! Presentation-facing data models shared by frontend renderers.
//!
//! These types intentionally contain no ratatui/crossterm types and no gameplay
//! behavior. Gameplay and screen modules can translate authoritative game state
//! into these models, while a frontend is responsible for rendering them.

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
pub(crate) struct AttributesView {
    pub might: i32,
    pub insight: i32,
    pub endurance: i32,
    pub effective_might: i32,
    pub effective_insight: i32,
    pub effective_endurance: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ConditionView {
    pub name: String,
    pub remaining: u32,
    pub penalty: i32,
    pub bonus: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CharacterSheetView {
    pub character: CharacterView,
    pub level: u32,
    pub experience: u32,
    pub next_level_experience: u32,
    pub attributes: AttributesView,
    pub conditions: Vec<ConditionView>,
    pub factions: Vec<FactionView>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ItemView {
    pub id: u64,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LocationView {
    pub id: u64,
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
    pub event_id: Option<String>,
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
    pub status: String,
    pub completed: bool,
    pub reward_claimed: bool,
    pub reward_item_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CombatantView {
    pub name: String,
    pub current_hp: i32,
    pub max_hp: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CombatView {
    pub character: CharacterView,
    pub player_condition: Option<String>,
    pub enemy: CombatantView,
    pub enemy_power: i32,
    pub location_name: String,
    pub turn: u32,
    pub events: Vec<String>,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct CombatResultView {
    pub combat: CombatView,
    pub result_title: String,
    pub result_note: String,
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct HistoryView {
    pub world_name: String,
    pub time: String,
    pub character: CharacterView,
    pub entries: Vec<HistoryEntryView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct NavigationView {
    pub current_location: Option<LocationView>,
    pub destinations: Vec<LocationView>,
    pub art: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TalkView {
    pub npcs: Vec<NpcView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ConversationView {
    pub npc: NpcView,
    pub portrait: Option<String>,
    pub memory: Option<String>,
    pub options: Vec<String>,
    pub available: bool,
    pub unavailable_message: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct InventoryView {
    pub character: CharacterView,
    pub items: Vec<ItemView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct InventoryDetailView {
    pub item: ItemView,
    pub position: usize,
    pub total: usize,
    pub art: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct QuestLogView {
    pub character: CharacterView,
    pub quests: Vec<QuestView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MeditationTargetView {
    pub label: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MeditationView {
    pub character: CharacterView,
    pub current_time: String,
    pub safe_to_meditate: bool,
    pub unavailable_message: Option<String>,
    pub targets: Vec<MeditationTargetView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct MeditationResultView {
    pub ending_time: String,
    pub portions: u32,
    pub hp_recovered: i32,
    pub exhausted_removed: bool,
    pub well_rested_applied: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ScreenView {
    pub title: String,
    pub subtitle: Option<String>,
    pub art: Option<String>,
    pub body: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ChoiceView {
    pub screen: ScreenView,
    pub prompt: String,
    pub options: Vec<String>,
    pub back_label: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct DeathView {
    pub screen: ScreenView,
    pub character: CharacterView,
    pub location_name: String,
    pub turn: u32,
    pub deeds: Vec<String>,
    pub faction_standing: Vec<FactionView>,
    pub dropped_items: Vec<ItemView>,
    pub memory_note: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RemainsEntryView {
    pub id: u64,
    pub label: String,
    pub former_name: String,
    pub former_title: String,
    pub scavenged: bool,
    pub items: Vec<ItemView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RemainsView {
    pub location_name: String,
    pub remains: Vec<RemainsEntryView>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct RemainsResultView {
    pub location_name: String,
    pub former_name: String,
    pub former_title: String,
    pub items: Vec<ItemView>,
    pub hidden_item: Option<ItemView>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ConsoleCandidateView {
    pub value: String,
    pub hint: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ConsoleScrollView {
    #[default]
    Follow,
    Offset(usize),
    Home,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ConsoleView {
    pub output: Vec<String>,
    pub input: String,
    pub scroll: ConsoleScrollView,
    pub completion_scroll: usize,
    pub candidates: Vec<ConsoleCandidateView>,
    pub selected: usize,
    pub autocomplete: bool,
}

#[cfg(test)]
mod tests {
    use super::{
        CharacterSheetView, CharacterView, ChoiceView, CombatResultView, CombatView, CombatantView,
        ConditionView, ConsoleScrollView, ConsoleView, ConversationView, DeathView, FactionView,
        HistoryEntryView, HistoryEntryViewType, HistoryView, InventoryDetailView, InventoryView,
        ItemView, LocationView, MeditationResultView, MeditationTargetView, MeditationView,
        NavigationView, NpcView, QuestLogView, QuestObjectiveView, QuestView, RemainsResultView,
        RemainsView, ScreenView, TalkView, ThreatView, WorldView,
    };

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
    fn empty_screen_views_are_frontend_safe() {
        let world = WorldView::default();
        let history = HistoryView::default();
        let inventory = InventoryView::default();
        let meditation = MeditationView::default();

        assert!(world.world_name.is_empty());
        assert!(world.time.is_empty());
        assert!(world.location.is_none());
        assert!(world.threat.is_none());
        assert!(world.history.is_empty());
        assert!(history.world_name.is_empty());
        assert!(history.entries.is_empty());
        assert!(inventory.items.is_empty());
        assert!(meditation.targets.is_empty());
    }

    #[test]
    fn lifecycle_and_console_views_hold_only_owned_frontend_data() {
        let screen = ScreenView {
            title: "TITLE".to_string(),
            subtitle: Some("Subtitle".to_string()),
            art: Some("art".to_string()),
            body: vec!["line".to_string()],
        };
        let choice = ChoiceView {
            screen: screen.clone(),
            prompt: "Choose".to_string(),
            options: vec!["One".to_string()],
            back_label: Some("Back".to_string()),
        };
        let _death = DeathView {
            screen: screen.clone(),
            character: CharacterView {
                name: "Ash".to_string(),
                ..Default::default()
            },
            location_name: "Gate".to_string(),
            turn: 4,
            deeds: vec!["Completed a quest.".to_string()],
            faction_standing: vec![],
            dropped_items: vec![],
            memory_note: "The next life will not remember.".to_string(),
        };
        let _remains = RemainsView {
            location_name: "Gate".to_string(),
            remains: vec![],
        };
        let _result = RemainsResultView {
            location_name: "Gate".to_string(),
            former_name: "Ash".to_string(),
            former_title: "Walker".to_string(),
            items: vec![],
            hidden_item: None,
            notes: vec![],
        };
        let console = ConsoleView {
            output: vec!["output".to_string()],
            input: "help".to_string(),
            scroll: ConsoleScrollView::Follow,
            completion_scroll: 0,
            candidates: vec![],
            selected: 0,
            autocomplete: false,
        };

        assert_eq!(choice.options, vec!["One".to_string()]);
        assert_eq!(console.input, "help");
    }

    #[test]
    fn view_models_are_renderer_neutral_and_owned() {
        let character = CharacterView {
            name: "Ash".to_string(),
            title: "Wanderer".to_string(),
            hp: 8,
            max_hp: 10,
        };
        let item = ItemView {
            id: 7,
            name: "Relic".to_string(),
            description: "A weathered relic.".to_string(),
        };
        let location = LocationView {
            id: 2,
            name: "Ruined Gate".to_string(),
            description: "A broken road marker.".to_string(),
            region_name: "North".to_string(),
            dangerous: true,
        };
        #[allow(unused_variables)]
        let threat = ThreatView {
            label: "Marauders stir".to_string(),
            description: "Someone is watching the road.".to_string(),
        };
        let history_entry = HistoryEntryView {
            day: 4,
            entry_type: HistoryEntryViewType::Event,
            text: "The road remembers.".to_string(),
            event_id: Some("road_event".to_string()),
            location_name: Some(location.name.clone()),
            outcome: Some("Danger remains.".to_string()),
        };
        let faction = FactionView {
            name: "Wardens".to_string(),
            reputation: 3,
            memories: vec!["A debt remembered.".to_string()],
        };
        let npc = NpcView {
            name: "Mara".to_string(),
            title: "Keeper".to_string(),
            faction_name: Some("Wardens".to_string()),
        };
        let quest_objective = QuestObjectiveView {
            label: "Visit Ruined Gate".to_string(),
            progress: 1,
            required: 1,
            completed: true,
        };
        let quest = QuestView {
            title: "A remembered road".to_string(),
            description: "Find the gate.".to_string(),
            objectives: vec![quest_objective],
            status: "READY".to_string(),
            completed: false,
            reward_claimed: false,
            reward_item_name: Some(item.name.clone()),
        };
        let combat = CombatView {
            character: character.clone(),
            player_condition: Some("Wounded".to_string()),
            enemy: CombatantView {
                name: "Marauder".to_string(),
                current_hp: 4,
                max_hp: 7,
            },
            enemy_power: 3,
            location_name: location.name.clone(),
            turn: 6,
            events: vec!["A clash begins.".to_string()],
            actions: vec![
                "Attack".to_string(),
                "Guard".to_string(),
                "Flee".to_string(),
            ],
        };
        let result = CombatResultView {
            combat: combat.clone(),
            result_title: "Fled".to_string(),
            result_note: "The threat remains.".to_string(),
        };
        let _character_sheet = CharacterSheetView {
            character: character.clone(),
            level: 2,
            experience: 12,
            next_level_experience: 100,
            attributes: super::AttributesView {
                might: 2,
                insight: 1,
                endurance: 3,
                effective_might: 1,
                effective_insight: 1,
                effective_endurance: 4,
            },
            conditions: vec![ConditionView {
                name: "Wounded".to_string(),
                remaining: 2,
                penalty: -1,
                bonus: 0,
            }],
            factions: vec![faction.clone()],
            notes: vec!["The gate was quiet.".to_string()],
        };
        let _conversation = ConversationView {
            npc,
            portrait: Some("portrait".to_string()),
            memory: Some("A remembered meeting.".to_string()),
            options: vec!["Ask about the road.".to_string()],
            available: true,
            unavailable_message: None,
        };
        let _inventory_detail = InventoryDetailView {
            item,
            position: 1,
            total: 1,
            art: Some("relic".to_string()),
        };
        let _quest_log = QuestLogView {
            character: character.clone(),
            quests: vec![quest],
        };
        let _navigation = NavigationView {
            current_location: Some(location.clone()),
            destinations: vec![location],
            art: Some("gate".to_string()),
        };
        let _talk = TalkView { npcs: vec![] };
        let _meditation_target = MeditationTargetView {
            label: "Dawn".to_string(),
        };
        let _meditation = MeditationView {
            character: character.clone(),
            current_time: "Day 1 · Dawn".to_string(),
            safe_to_meditate: true,
            unavailable_message: None,
            targets: vec![],
        };
        let _meditation_result = MeditationResultView {
            ending_time: "Day 1 · Morning".to_string(),
            portions: 1,
            hp_recovered: 3,
            exhausted_removed: true,
            well_rested_applied: true,
        };
        let _history = HistoryView {
            world_name: "Test World".to_string(),
            time: "Day 1 · Dawn".to_string(),
            character,
            entries: vec![history_entry],
        };
        assert_eq!(combat, result.combat);
    }
}
