//! Touch gesture handling for graphical UI controls.
//!
//! A touch that begins on a choice is treated as a tap only when it remains
//! within a small movement threshold. Once it becomes a drag, the choice is
//! suppressed so the surrounding scrollable panel can consume the gesture.

use bevy::prelude::*;

use crate::bevy_presentation::{ChoiceButton, GameplayInputQueue, NavigationState, ScreenId, SemanticInputQueue};
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
    mut state: ResMut<TouchChoiceState>,
    mut buttons: Query<
        (
            Entity,
            &Interaction,
            &ComputedNode,
            &UiGlobalTransform,
        ),
        With<ChoiceButton>,
    >,
) {
    if let Some(active) = state.active {
        if let Some(touch) = touches.iter().find(|touch| touch.id() == active.id) {
            let distance = touch.position().distance(active.start);
            let dragging = active.dragging || distance > TOUCH_TAP_THRESHOLD;
            state.active = Some(ActiveTouch { dragging, ..active });
        } else if touches.just_released(active.id) || touches.just_canceled(active.id) {
            state.active = None;
            return;
        }
    }

    for touch in touches.iter_just_pressed() {
        let button = buttons
            .iter()
            .find(|(_, _, computed, transform)| computed.contains_point(**transform, touch.position()))
            .map(|(entity, _, _, _)| entity);
        state.active = Some(ActiveTouch {
            id: touch.id(),
            start: touch.position(),
            button,
            dragging: false,
        });
    }

    if let Some(active) = state.active {
        for (entity, interaction, _, _) in &mut buttons {
            if *interaction != Interaction::Pressed {
                continue;
            }
            if active.dragging || active.button != Some(entity) {
                // Touch presses are confirmed on release by complete_touch_choice.
                // Clearing Interaction::Pressed here prevents the existing generic
                // button handler from treating a touch as an immediate click.
                if let Ok((_, mut interaction, _, _)) = buttons.get_mut(entity) {
                    *interaction = Interaction::None;
                }
            } else if let Ok((_, mut interaction, _, _)) = buttons.get_mut(entity) {
                *interaction = Interaction::None;
            }
        }
    }
}

fn complete_touch_choice(
    touches: Res<Touches>,
    mut state: ResMut<TouchChoiceState>,
    buttons: Query<
        (Entity, &ChoiceButton, &ComputedNode, &UiGlobalTransform),
        With<ChoiceButton>,
    >,
    navigation: Res<NavigationState>,
    mut lifecycle_queue: ResMut<SemanticInputQueue>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
) {
    let Some(active) = state.active else {
        return;
    };

    let released = touches.just_released(active.id) || touches.just_canceled(active.id);
    if !released {
        return;
    }

    state.active = None;
    if active.dragging || active.button.is_none() {
        return;
    }

    let release_position = buttons
        .iter()
        .find(|(entity, _, computed, transform)| {
            Some(*entity) == active.button && computed.contains_point(**transform, release_position(active.id, &touches))
        })
        .map(|(_, choice, _, _)| choice.index);

    let Some(index) = release_position else {
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

fn release_position(id: u64, touches: &Touches) -> Vec2 {
    touches
        .iter_just_released()
        .find(|touch| touch.id() == id)
        .map(|touch| touch.position())
        .unwrap_or(Vec2::ZERO)
}

#[cfg(test)]
mod tests {
    use super::TOUCH_TAP_THRESHOLD;

    #[test]
    fn touch_threshold_is_large_enough_to_ignore_small_jitter() {
        assert!(TOUCH_TAP_THRESHOLD >= 8.0);
        assert!(TOUCH_TAP_THRESHOLD <= 24.0);
    }
}
