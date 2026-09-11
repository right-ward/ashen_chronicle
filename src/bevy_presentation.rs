//! Reusable Bevy presentation primitives for the graphical frontend.
//!
//! This module depends only on Bevy and the frontend-neutral semantic input
//! model. Gameplay systems remain responsible for translating authoritative game
//! state into presentation view models and interpreting semantic input events.

use bevy::prelude::*;

use crate::input::InputEvent;

pub const THEME_BACKGROUND: Color = Color::srgb(0.055, 0.045, 0.065);
pub const THEME_PANEL: Color = Color::srgb(0.105, 0.085, 0.12);
pub const THEME_PANEL_ALT: Color = Color::srgb(0.135, 0.11, 0.15);
pub const THEME_TEXT: Color = Color::srgb(0.9, 0.88, 0.82);
pub const THEME_MUTED: Color = Color::srgb(0.62, 0.6, 0.58);
pub const THEME_ACCENT: Color = Color::srgb(0.72, 0.62, 0.46);

#[derive(Component)]
pub struct BevyScreenRoot;

#[derive(Component)]
pub struct ChoiceButton {
    pub index: usize,
}

#[derive(Component)]
pub struct TextContent;

#[derive(Component)]
pub struct GaugeFill;

#[derive(Component)]
pub struct ScrollViewport;

#[derive(Resource, Default)]
pub struct SemanticInputQueue(pub Vec<InputEvent>);

#[derive(Resource, Default)]
pub struct GameplayInputQueue(pub Vec<InputEvent>);

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NavigationState {
    pub current_screen: Option<ScreenId>,
    pub return_screen: Option<ScreenId>,
    pub selected: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenId {
    Lifecycle,
    Gameplay,
    Character,
    Inventory,
    Quests,
    Meditation,
    History,
    Journal,
    Combat,
    Console,
}

pub fn install(app: &mut App) {
    app.init_resource::<SemanticInputQueue>()
        .init_resource::<GameplayInputQueue>()
        .init_resource::<NavigationState>()
        .add_systems(Update, keyboard_to_semantic_input)
        .add_systems(Update, choice_button_input);
}

pub fn spawn_screen(commands: &mut Commands, title: impl Into<String>) -> Entity {
    commands
        .spawn((
            BevyScreenRoot,
            Node {
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(24)),
                flex_direction: FlexDirection::Column,
                row_gap: px(16),
                ..default()
            },
            BackgroundColor(THEME_BACKGROUND),
            children![(
                Text::new(title.into()),
                TextContent,
                TextFont::from_font_size(FontSize::Px(36.0)),
                TextColor(THEME_TEXT),
            )],
        ))
        .id()
}

pub fn spawn_panel(commands: &mut Commands, parent: Entity) -> Entity {
    let panel = commands
        .spawn((
            Node {
                width: percent(100),
                padding: UiRect::all(px(16)),
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                overflow: Overflow::scroll(),
                ..default()
            },
            BackgroundColor(THEME_PANEL),
        ))
        .id();
    commands.entity(parent).add_child(panel);
    panel
}

pub fn spawn_label(commands: &mut Commands, parent: Entity, text: impl Into<String>) -> Entity {
    let label = commands
        .spawn((
            Text::new(text.into()),
            TextContent,
            TextFont::from_font_size(FontSize::Px(20.0)),
            TextColor(THEME_TEXT),
        ))
        .id();
    commands.entity(parent).add_child(label);
    label
}

pub fn spawn_muted_label(
    commands: &mut Commands,
    parent: Entity,
    text: impl Into<String>,
) -> Entity {
    let label = commands
        .spawn((
            Text::new(text.into()),
            TextContent,
            TextFont::from_font_size(FontSize::Px(18.0)),
            TextColor(THEME_MUTED),
        ))
        .id();
    commands.entity(parent).add_child(label);
    label
}

pub fn spawn_choice_button(
    commands: &mut Commands,
    parent: Entity,
    index: usize,
    label: impl Into<String>,
) -> Entity {
    let button = commands
        .spawn((
            Button,
            ChoiceButton { index },
            Node {
                width: percent(100),
                min_height: px(48),
                padding: UiRect::axes(px(14), px(8)),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(THEME_ACCENT),
            BackgroundColor(THEME_PANEL_ALT),
            children![(
                Text::new(label.into()),
                TextContent,
                TextFont::from_font_size(FontSize::Px(20.0)),
                TextColor(THEME_TEXT),
            )],
        ))
        .id();
    commands.entity(parent).add_child(button);
    button
}

pub fn spawn_gauge(commands: &mut Commands, parent: Entity, current: i32, maximum: i32) -> Entity {
    let ratio = gauge_ratio(current, maximum);
    let gauge = commands
        .spawn((
            Node {
                width: percent(100),
                height: px(18),
                ..default()
            },
            BackgroundColor(THEME_PANEL_ALT),
            children![(
                GaugeFill,
                Node {
                    width: percent(ratio * 100.0),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(THEME_ACCENT),
            )],
        ))
        .id();
    commands.entity(parent).add_child(gauge);
    gauge
}

pub(crate) fn gauge_ratio(current: i32, maximum: i32) -> f32 {
    if maximum <= 0 {
        0.0
    } else {
        (current.max(0) as f32 / maximum as f32).clamp(0.0, 1.0)
    }
}

fn keyboard_to_semantic_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    navigation: Res<NavigationState>,
    mut lifecycle_queue: ResMut<SemanticInputQueue>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
) {
    let mappings = [
        (KeyCode::ArrowUp, InputEvent::Up),
        (KeyCode::ArrowDown, InputEvent::Down),
        (KeyCode::Home, InputEvent::Home),
        (KeyCode::End, InputEvent::End),
        (KeyCode::PageUp, InputEvent::PageUp),
        (KeyCode::PageDown, InputEvent::PageDown),
        (KeyCode::Enter, InputEvent::Confirm),
        (KeyCode::Escape, InputEvent::Cancel),
        (KeyCode::Tab, InputEvent::Tab),
        (KeyCode::Backspace, InputEvent::Backspace),
        (KeyCode::Delete, InputEvent::Delete),
    ];
    for (key, event) in mappings {
        if keyboard.just_pressed(key) {
            if navigation.current_screen == Some(ScreenId::Lifecycle) {
                lifecycle_queue.0.push(event);
            } else {
                gameplay_queue.0.push(event);
            }
        }
    }
}

fn choice_button_input(
    mut interaction_query: Query<
        (&Interaction, &ChoiceButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut navigation: ResMut<NavigationState>,
    mut lifecycle_queue: ResMut<SemanticInputQueue>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
) {
    for (interaction, choice) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            navigation.selected = choice.index;
            if navigation.current_screen == Some(ScreenId::Lifecycle) {
                lifecycle_queue.0.push(InputEvent::Confirm);
            } else {
                gameplay_queue.0.push(InputEvent::Confirm);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigation_state_starts_without_a_screen_or_selection() {
        let state = NavigationState::default();
        assert_eq!(state.current_screen, None);
        assert_eq!(state.return_screen, None);
        assert_eq!(state.selected, 0);
    }

    #[test]
    fn screen_ids_cover_migrated_frontend_domains() {
        assert_ne!(ScreenId::Lifecycle, ScreenId::Gameplay);
        assert_ne!(ScreenId::Inventory, ScreenId::Quests);
        assert_ne!(ScreenId::History, ScreenId::Journal);
        assert_ne!(ScreenId::Combat, ScreenId::Console);
    }

    #[test]
    fn gauge_ratio_is_clamped_to_valid_bounds() {
        assert_eq!(gauge_ratio(-2, 10), 0.0);
        assert_eq!(gauge_ratio(5, 10), 0.5);
        assert_eq!(gauge_ratio(20, 10), 1.0);
        assert_eq!(gauge_ratio(5, 0), 0.0);
    }
}
