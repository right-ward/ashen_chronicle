//! Touch gesture handling for graphical UI controls.
//!
//! A touch that begins on a choice is treated as a tap only when it remains
//! within a small movement threshold. Once it becomes a drag, the choice is
//! suppressed so the surrounding scrollable panel can consume the gesture.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::bevy_presentation::{
    physical_touch_position, ChoiceButton, GameplayInputQueue, NavigationState, ScreenId,
    SemanticInputQueue,
};
use crate::input::InputEvent;

const TOUCH_TAP_THRESHOLD: f32 = 12.0;

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

pub fn install(app: &mut App) {
    app.init_resource::<TouchChoiceState>()
        .add_systems(
            PreUpdate,
            suppress_touch_choice_press.after(bevy::ui::UiSystems::Focus),
        )
        .add_systems(Update, complete_touch_choice);
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

fn complete_touch_choice(
    touches: Res<Touches>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut state: ResMut<TouchChoiceState>,
    buttons: Query<(Entity, &ChoiceButton, &ComputedNode, &UiGlobalTransform), With<ChoiceButton>>,
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
    let Some(index) = buttons
        .iter()
        .find(|(entity, _, computed, transform)| {
            Some(*entity) == active.button
                && computed.contains_point(**transform, physical_release_position)
        })
        .map(|(_, choice, _, _)| choice.index)
    else {
        return;
    };

    let queue = if navigation.current_screen == Some(ScreenId::Lifecycle) {
        &mut lifecycle_queue.0
    } else {
        &mut gameplay_queue.0
    };
    queue.push(InputEvent::Home);
    for _ in 0..index {
        queue.push(InputEvent::Down);
    }
    queue.push(InputEvent::Confirm);
}
