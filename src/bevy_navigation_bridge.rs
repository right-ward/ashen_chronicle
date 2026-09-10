//! Bevy navigation glue between the gameplay dashboard and dedicated screens.
//!
//! This keeps transition detection outside gameplay and record renderers while
//! using their existing navigation state as the hand-off protocol.

use bevy::prelude::*;

use crate::bevy_gameplay::GameplayState;
use crate::bevy_lifecycle::{GameSession, LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{GameplayInputQueue, NavigationState, ScreenId};
use crate::game::menu;
use crate::input::InputEvent;

#[derive(Resource, Default)]
pub(crate) struct NavigationBridgeState {
    last_screen: Option<ScreenId>,
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<NavigationBridgeState>()
        .add_systems(Update, bridge_navigation);
}

fn bridge_navigation(
    lifecycle: Res<LifecycleState>,
    gameplay: Res<GameplayState>,
    mut navigation: ResMut<NavigationState>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
    mut bridge: ResMut<NavigationBridgeState>,
) {
    if lifecycle.phase != LifecyclePhase::Complete || lifecycle.session.is_none() {
        bridge.last_screen = navigation.current_screen;
        return;
    }

    if navigation.current_screen == Some(ScreenId::Lifecycle)
        && lifecycle.pending_state.is_none()
        && navigation.return_screen.is_none()
    {
        navigation.current_screen = Some(ScreenId::Gameplay);
        navigation.selected = gameplay.selected;
    }

    if navigation.current_screen == Some(ScreenId::Gameplay)
        && navigation.return_screen == Some(ScreenId::Gameplay)
    {
        if let Some(screen) = dedicated_screen_for_selection(&lifecycle, gameplay.selected) {
            navigation.current_screen = Some(screen);
            navigation.selected = 0;
        }
    }

    if navigation.current_screen != bridge.last_screen {
        if navigation.current_screen == Some(ScreenId::Gameplay) {
            gameplay_queue.0.push(InputEvent::Home);
        }
        bridge.last_screen = navigation.current_screen;
    }
}

fn dedicated_screen_for_selection(lifecycle: &LifecycleState, selected: usize) -> Option<ScreenId> {
    let session: &GameSession = lifecycle.session.as_ref()?;
    let entries = menu::build_main_menu(&session.state);
    let entry = entries.get(selected)?;
    match entry.action {
        menu::GameAction::CharacterSheet => Some(ScreenId::Character),
        menu::GameAction::Inventory => Some(ScreenId::Inventory),
        menu::GameAction::QuestLog => Some(ScreenId::Quests),
        menu::GameAction::Meditate => Some(ScreenId::Meditation),
        menu::GameAction::History => Some(ScreenId::History),
        menu::GameAction::Journal => Some(ScreenId::Journal),
        _ => None,
    }
}
