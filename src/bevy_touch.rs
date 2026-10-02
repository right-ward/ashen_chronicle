//! Touch gesture handling for graphical UI controls.
//!
//! A touch that begins on a choice is treated as a tap only when it remains
//! within a small movement threshold. Once it becomes a drag, the choice is
//! suppressed so the surrounding scrollable panel can consume the gesture.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::bevy_presentation::{
    physical_touch_position, queue_choice_activation, ChoiceButton, GameplayInputQueue,
    NavigationState, ScreenId, SemanticInputQueue, UiMenuButton, UiTabButton,
};
const TOUCH_TAP_THRESHOLD: f32 = 12.0;
const TOUCH_MENU_LONG_PRESS_SECS: f32 = 0.65;

#[derive(Resource, Default)]
struct TouchChoiceState {
    active: Option<ActiveTouch>,
}

#[derive(Clone, Copy)]
struct ActiveTouch {
    id: u64,
    start: Vec2,
    button: Option<Entity>,
    dragging: bool,
}

#[derive(Resource, Default)]
struct TouchMenuState {
    active: Option<ActiveMenuTouch>,
}

#[derive(Clone, Copy)]
struct ActiveMenuTouch {
    id: u64,
    start: Vec2,
    button: Entity,
    started_at: f32,
    dragging: bool,
}

pub fn install(app: &mut App) {
    app.init_resource::<TouchChoiceState>()
        .init_resource::<TouchMenuState>()
        .add_systems(
            PreUpdate,
            (suppress_touch_choice_press, suppress_touch_menu_press)
                .after(bevy::ui::UiSystems::Focus),
        )
        .add_systems(Update, (complete_touch_choice, complete_touch_menu));
}

fn suppress_touch_choice_press(
    touches: Res<Touches>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut state: ResMut<TouchChoiceState>,
    mut buttons: Query<
        (Entity, &mut Interaction, &ComputedNode, &UiGlobalTransform),
        With<ChoiceButton>,
    >,
) {
    for touch in touches.iter_just_pressed() {
        let physical_position = physical_touch_position(touch.position(), window.scale_factor());
        let button = buttons
            .iter()
            .find(|(_, _, computed, transform)| {
                computed.contains_point(**transform, physical_position)
            })
            .map(|(entity, _, _, _)| entity);
        state.active = Some(ActiveTouch {
            id: touch.id(),
            start: touch.position(),
            button,
            dragging: false,
        });
    }

    let Some(active) = state.active else {
        return;
    };

    if let Some(touch) = touches.iter().find(|touch| touch.id() == active.id) {
        let distance = touch.position().distance(active.start);
        if distance > TOUCH_TAP_THRESHOLD && !active.dragging {
            state.active = Some(ActiveTouch {
                dragging: true,
                ..active
            });
        }
    }

    for (_, mut interaction, _, _) in &mut buttons {
        if *interaction == Interaction::Pressed {
            // Touch presses are confirmed on release by complete_touch_choice.
            // Clearing Interaction::Pressed prevents the generic button handler
            // from treating a touch as an immediate click or a scroll as a click.
            *interaction = Interaction::None;
        }
    }
}

fn suppress_touch_menu_press(
    touches: Res<Touches>,
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut state: ResMut<TouchMenuState>,
    mut buttons: Query<
        (Entity, &mut Interaction, &ComputedNode, &UiGlobalTransform),
        With<UiMenuButton>,
    >,
) {
    for touch in touches.iter_just_pressed() {
        let physical_position = physical_touch_position(touch.position(), window.scale_factor());
        let Some((entity, _, _, _)) = buttons.iter().find(|(_, _, computed, transform)| {
            computed.contains_point(**transform, physical_position)
        }) else {
            continue;
        };

        state.active = Some(ActiveMenuTouch {
            id: touch.id(),
            start: touch.position(),
            button: entity,
            started_at: time.elapsed_secs(),
            dragging: false,
        });
    }

    let Some(active) = state.active else {
        return;
    };

    let Some(touch) = touches.iter().find(|touch| touch.id() == active.id) else {
        if touches.just_canceled(active.id) {
            state.active = None;
        }
        return;
    };

    if touch.position().distance(active.start) > TOUCH_TAP_THRESHOLD && !active.dragging {
        state.active = Some(ActiveMenuTouch {
            dragging: true,
            ..active
        });
    }

    if let Ok((_, mut interaction, _, _)) = buttons.get_mut(active.button) {
        if *interaction == Interaction::Pressed {
            // Resolve menu gestures ourselves so a touch cannot open the menu
            // immediately through the generic pointer Interaction path.
            *interaction = Interaction::None;
        }
    }
}

fn should_open_developer_console(elapsed_secs: f32, started_at: f32, dragging: bool) -> bool {
    !dragging && elapsed_secs - started_at >= TOUCH_MENU_LONG_PRESS_SECS
}

fn complete_touch_menu(
    touches: Res<Touches>,
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut state: ResMut<TouchMenuState>,
    navigation: Res<NavigationState>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
    buttons: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<UiMenuButton>>,
) {
    let Some(active) = state.active else {
        return;
    };

    if navigation.current_screen != Some(ScreenId::Gameplay) {
        state.active = None;
        return;
    }

    if should_open_developer_console(time.elapsed_secs(), active.started_at, active.dragging) {
        state.active = None;
        gameplay_queue
            .0
            .push(crate::input::InputEvent::OpenDeveloperConsole);
        return;
    }

    let Some(released_touch) = touches
        .iter_just_released()
        .find(|touch| touch.id() == active.id)
    else {
        if touches.just_canceled(active.id) {
            state.active = None;
        }
        return;
    };

    state.active = None;
    if active.dragging {
        return;
    }

    let physical_release_position =
        physical_touch_position(released_touch.position(), window.scale_factor());
    let Some((_, computed, transform)) = buttons
        .iter()
        .find(|(entity, _, _)| *entity == active.button)
    else {
        return;
    };

    if computed.contains_point((**transform).into(), physical_release_position) {
        gameplay_queue
            .0
            .push(crate::input::InputEvent::OpenSecondaryNavigation);
    }
}

fn complete_touch_choice(
    touches: Res<Touches>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut state: ResMut<TouchChoiceState>,
    buttons: Query<
        (
            Entity,
            &ChoiceButton,
            Option<&UiTabButton>,
            &ComputedNode,
            &UiGlobalTransform,
        ),
        With<ChoiceButton>,
    >,
    navigation: Res<NavigationState>,
    mut lifecycle_queue: ResMut<SemanticInputQueue>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
) {
    let Some(active) = state.active else {
        return;
    };

    let Some(released_touch) = touches
        .iter_just_released()
        .find(|touch| touch.id() == active.id)
    else {
        if touches.just_canceled(active.id) {
            state.active = None;
        }
        return;
    };

    state.active = None;
    if active.dragging {
        return;
    }

    let physical_release_position =
        physical_touch_position(released_touch.position(), window.scale_factor());
    let Some((_, choice, tab, _, _)) = buttons.iter().find(|(entity, _, _, computed, transform)| {
        Some(*entity) == active.button
            && computed.contains_point(**transform, physical_release_position)
    }) else {
        return;
    };

    if let Some(tab) = tab {
        if navigation.current_screen == Some(ScreenId::Options) {
            gameplay_queue.0.push(crate::input::InputEvent::SelectTab(tab.index));
        }
        return;
    }

    let queue = if navigation.current_screen == Some(ScreenId::Lifecycle) {
        &mut lifecycle_queue.0
    } else {
        &mut gameplay_queue.0
    };
    queue_choice_activation(queue, choice.index);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn developer_console_long_press_uses_a_deliberate_threshold() {
        assert!(!should_open_developer_console(10.64, 10.0, false));
        assert!(should_open_developer_console(10.66, 10.0, false));
        assert!(!should_open_developer_console(10.80, 10.0, true));
    }
}
