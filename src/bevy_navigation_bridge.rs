//! Bevy navigation glue for lifecycle completion and gameplay screen hand-offs.
//!
//! Dedicated graphical screen transitions are owned by their originating
//! gameplay systems; this bridge only handles cross-system lifecycle completion
//! and the shared gameplay re-entry selection reset.

use bevy::prelude::*;

use crate::bevy_gameplay::{GameplayScreen, GameplayState};
use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{GameplayInputQueue, NavigationState, ScreenId};
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

    if navigation.current_screen != bridge.last_screen {
        if should_reset_gameplay_selection(
            bridge.last_screen,
            navigation.current_screen,
            gameplay.screen,
        ) {
            gameplay_queue.0.push(InputEvent::Home);
        }
        bridge.last_screen = navigation.current_screen;
    }
}

fn should_reset_gameplay_selection(
    previous_screen: Option<ScreenId>,
    current_screen: Option<ScreenId>,
    gameplay_screen: GameplayScreen,
) -> bool {
    current_screen == Some(ScreenId::Gameplay)
        && previous_screen != current_screen
        && gameplay_screen != GameplayScreen::Pause
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
        menu::GameAction::Talk => Some(ScreenId::Talk),
        menu::GameAction::SearchRemains => Some(ScreenId::Remains),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gameplay_reentry_does_not_reset_pause_selection() {
        assert!(!should_reset_gameplay_selection(
            Some(ScreenId::Options),
            Some(ScreenId::Gameplay),
            GameplayScreen::Pause
        ));
    }

    #[test]
    fn gameplay_reentry_resets_selection_for_normal_gameplay() {
        assert!(should_reset_gameplay_selection(
            Some(ScreenId::Character),
            Some(ScreenId::Gameplay),
            GameplayScreen::Dashboard
        ));
    }
}
