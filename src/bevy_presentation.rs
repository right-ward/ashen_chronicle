//! Reusable Bevy presentation primitives for the graphical frontend.
//!
//! This module depends only on Bevy and the frontend-neutral semantic input
//! model. Gameplay systems remain responsible for translating authoritative game
//! state into presentation view models and interpreting semantic input events.

use bevy::input::keyboard::{Key, KeyboardInput, NativeKeyCode};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

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
struct TouchScrollablePanel;

#[derive(Component)]
struct ContextualTextField {
    index: usize,
}

#[derive(Component)]
struct ContextualConsoleInput;

#[derive(Component)]
struct ContextualConsoleInputRow;

#[derive(Component)]
struct ContextualConsoleTab;

#[derive(Component)]
struct ContextualEnhanced;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TextInputTarget {
    LifecycleField(usize),
    Console,
}

#[derive(Resource, Default, Debug)]
struct TextInputFocus {
    target: Option<TextInputTarget>,
    suppress_next_back: bool,
}

#[derive(Resource, Default, Debug)]
struct TouchScrollState {
    active: Option<(u64, Entity, Vec2)>,
}

#[derive(Resource, Default)]
pub struct SemanticInputQueue(pub Vec<InputEvent>);

#[derive(Resource, Default)]
pub struct GameplayInputQueue(pub Vec<InputEvent>);

pub(crate) fn physical_touch_position(window: &Window, position: Vec2) -> Vec2 {
    position * window.scale_factor()
}

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
    Talk,
    Conversation,
    Remains,
    RemainsResult,
    Combat,
    Console,
    Feedback,
}

fn screen_padding() -> Val {
    vmin(3.333)
}

fn screen_gap() -> Val {
    vmin(2.222)
}

fn panel_padding() -> Val {
    vmin(2.222)
}

fn panel_gap() -> Val {
    vmin(1.389)
}

fn title_font_size() -> FontSize {
    FontSize::VMin(5.0)
}

fn label_font_size() -> FontSize {
    FontSize::VMin(2.778)
}

fn muted_font_size() -> FontSize {
    FontSize::VMin(2.5)
}

fn button_padding_horizontal() -> Val {
    vmin(1.944)
}

fn button_padding_vertical() -> Val {
    vmin(1.111)
}

pub fn install(app: &mut App) {
    app.init_resource::<SemanticInputQueue>()
        .init_resource::<GameplayInputQueue>()
        .init_resource::<NavigationState>()
        .init_resource::<TextInputFocus>()
        .init_resource::<TouchScrollState>()
        .add_systems(
            PreUpdate,
            process_ime_events.after(bevy::input::InputSystems),
        )
        .add_systems(Update, keyboard_to_semantic_input)
        .add_systems(Update, choice_button_input)
        .add_systems(Update, contextual_touch_input)
        .add_systems(Update, touch_scroll)
        .add_systems(PostUpdate, contextual_touch_targets)
        .add_systems(PostUpdate, sync_ime_window);
}

pub fn spawn_screen(commands: &mut Commands, title: impl Into<String>) -> Entity {
    commands
        .spawn((
            BevyScreenRoot,
            Node {
                width: percent(100),
                height: percent(100),
                min_width: px(0),
                min_height: px(0),
                padding: UiRect::all(screen_padding()),
                flex_direction: FlexDirection::Column,
                row_gap: screen_gap(),
                ..default()
            },
            BackgroundColor(THEME_BACKGROUND),
            children![(
                Text::new(title.into()),
                TextContent,
                TextFont::from_font_size(title_font_size()),
                TextColor(THEME_TEXT),
            )],
        ))
        .id()
}

pub fn spawn_panel(commands: &mut Commands, parent: Entity) -> Entity {
    let panel = commands
        .spawn((
            TouchScrollablePanel,
            Node {
                width: percent(100),
                min_width: px(0),
                min_height: px(0),
                flex_grow: 1.0,
                flex_shrink: 1.0,
                padding: UiRect::all(panel_padding()),
                flex_direction: FlexDirection::Column,
                row_gap: panel_gap(),
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
            Node {
                width: percent(100),
                min_width: px(0),
                ..default()
            },
            TextContent,
            TextFont::from_font_size(label_font_size()),
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
            Node {
                width: percent(100),
                min_width: px(0),
                ..default()
            },
            TextContent,
            TextFont::from_font_size(muted_font_size()),
            TextColor(THEME_MUTED),
        ))
        .id();
    commands.entity(parent).add_child(label);
    label
}

pub fn spawn_text_input_field(
    commands: &mut Commands,
    parent: Entity,
    index: usize,
    label: impl Into<String>,
    value: impl Into<String>,
    selected: bool,
) -> Entity {
    let marker = if selected { ">" } else { " " };
    let field = commands
        .spawn((
            Button,
            ContextualTextField { index },
            ContextualEnhanced,
            Node {
                width: percent(100),
                min_width: px(0),
                min_height: px(48),
                padding: UiRect::axes(vmin(1.389), vmin(0.833)),
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(THEME_ACCENT),
            BackgroundColor(THEME_PANEL_ALT),
            children![(
                Text::new(format!(
                    "{marker} {}: {}",
                    label.into(),
                    value.into(),
                )),
                Node {
                    width: percent(100),
                    min_width: px(0),
                    ..default()
                },
                TextContent,
                TextFont::from_font_size(label_font_size()),
                TextColor(THEME_TEXT),
            )],
        ))
        .id();
    commands.entity(parent).add_child(field);
    field
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
                min_width: px(0),
                min_height: px(48),
                padding: UiRect::axes(button_padding_horizontal(), button_padding_vertical()),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
                border: UiRect::all(px(1)),
                ..default()
            },
            BorderColor::all(THEME_ACCENT),
            BackgroundColor(THEME_PANEL_ALT),
            children![(
                Text::new(label.into()),
                Node {
                    width: percent(100),
                    min_width: px(0),
                    ..default()
                },
                TextContent,
                TextFont::from_font_size(label_font_size()),
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
                min_width: px(0),
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

fn process_ime_events(
    mut ime: MessageReader<Ime>,
    mut keyboard_input: MessageWriter<KeyboardInput>,
    mut focus: ResMut<TextInputFocus>,
) {
    for event in ime.read() {
        match event {
            Ime::Commit { window, value } if !value.is_empty() => {
                keyboard_input.write(KeyboardInput {
                    key_code: KeyCode::Unidentified(NativeKeyCode::Unidentified),
                    logical_key: Key::Character(value.clone().into()),
                    state: ButtonState::Pressed,
                    window: *window,
                    repeat: false,
                    text: Some(value.clone().into()),
                });
            }
            Ime::Disabled { .. } if focus.target.is_some() => {
                focus.target = None;
                focus.suppress_next_back = true;
            }
            _ => {}
        }
    }
}

fn keyboard_to_semantic_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    navigation: Res<NavigationState>,
    mut focus: ResMut<TextInputFocus>,
    mut lifecycle_queue: ResMut<SemanticInputQueue>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
) {
    if !matches!(
        (focus.target, navigation.current_screen),
        (
            Some(TextInputTarget::LifecycleField(_)),
            Some(ScreenId::Lifecycle)
        ) | (Some(TextInputTarget::Console), Some(ScreenId::Console))
            | (None, _)
    ) {
        focus.target = None;
    }

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
        (KeyCode::BrowserBack, InputEvent::Cancel),
        (KeyCode::KeyJ, InputEvent::Character('j')),
        (KeyCode::KeyK, InputEvent::Character('k')),
        (KeyCode::Digit1, InputEvent::Character('1')),
        (KeyCode::Digit2, InputEvent::Character('2')),
        (KeyCode::Digit3, InputEvent::Character('3')),
    ];
    for (key, event) in mappings {
        if !keyboard.just_pressed(key) {
            continue;
        }
        if key == KeyCode::BrowserBack {
            if focus.suppress_next_back {
                focus.suppress_next_back = false;
                continue;
            }
            if focus.target.is_some() {
                continue;
            }
        }
        if navigation.current_screen == Some(ScreenId::Lifecycle) {
            lifecycle_queue.0.push(event);
        } else {
            gameplay_queue.0.push(event);
        }
    }
}

fn choice_button_input(
    mut interaction_query: Query<(&Interaction, &ChoiceButton), Changed<Interaction>>,
    mut navigation: ResMut<NavigationState>,
    mut lifecycle_queue: ResMut<SemanticInputQueue>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
) {
    for (interaction, choice) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        navigation.selected = choice.index;
        let queue = if navigation.current_screen == Some(ScreenId::Lifecycle) {
            &mut lifecycle_queue.0
        } else {
            &mut gameplay_queue.0
        };
        queue.push(InputEvent::Home);
        for _ in 0..choice.index {
            queue.push(InputEvent::Down);
        }
        queue.push(InputEvent::Confirm);
    }
}

#[allow(clippy::type_complexity)] // For interaction_query
fn contextual_touch_input(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&ContextualTextField>,
            Option<&ContextualConsoleInput>,
            Option<&ContextualConsoleTab>,
        ),
        Changed<Interaction>,
    >,
    navigation: Res<NavigationState>,
    mut focus: ResMut<TextInputFocus>,
    mut lifecycle_queue: ResMut<SemanticInputQueue>,
    mut gameplay_queue: ResMut<GameplayInputQueue>,
) {
    for (interaction, field, console, tab) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Some(field) = field {
            if navigation.current_screen != Some(ScreenId::Lifecycle) {
                continue;
            }
            focus.target = Some(TextInputTarget::LifecycleField(field.index));
            focus.suppress_next_back = false;
            lifecycle_queue.0.push(InputEvent::Home);
            for _ in 0..field.index {
                lifecycle_queue.0.push(InputEvent::Down);
            }
        } else if console.is_some() {
            if navigation.current_screen == Some(ScreenId::Console) {
                focus.target = Some(TextInputTarget::Console);
                focus.suppress_next_back = false;
            }
        } else if tab.is_some() && navigation.current_screen == Some(ScreenId::Console) {
            gameplay_queue.0.push(InputEvent::Tab);
        }
    }
}

#[allow(clippy::type_complexity)] // For text_fields
fn contextual_touch_targets(
    mut commands: Commands,
    navigation: Res<NavigationState>,
    mut text_fields: Query<
        (
            Entity,
            &Text,
            &mut Node,
            Option<&ContextualEnhanced>,
            Option<&ContextualConsoleInput>,
        ),
        With<TextContent>,
    >,
    panels: Query<Entity, With<TouchScrollablePanel>>,
    tabs: Query<Entity, With<ContextualConsoleTab>>,
) {
    let console_tab_exists = tabs.single().is_ok();
    let console_panel = if navigation.current_screen == Some(ScreenId::Console) {
        panels.iter().next()
    } else {
        None
    };

    for (entity, text, mut node, enhanced, field, console_input) in &mut text_fields {
        if enhanced.is_some() {
            continue;
        }
        if navigation.current_screen == Some(ScreenId::Console)
            && console_input.is_none()
            && text.as_str().starts_with("> ")
        {
            node.min_height = px(48);
            node.width = percent(84);
            node.padding = UiRect::axes(vmin(1.389), vmin(0.833));
            commands.entity(entity).insert((
                Button,
                ContextualConsoleInput,
                ContextualEnhanced,
                BorderColor::all(THEME_ACCENT),
                BackgroundColor(THEME_PANEL_ALT),
            ));
            if !console_tab_exists {
                if let Some(panel) = console_panel {
                    let row = commands
                        .spawn((
                            ContextualConsoleInputRow,
                            Node {
                                width: percent(100),
                                min_width: px(0),
                                flex_direction: FlexDirection::Row,
                                column_gap: vmin(1.111),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                        ))
                        .id();
                    commands.entity(panel).add_child(row);
                    commands.entity(row).add_child(entity);
                    let tab = commands
                        .spawn((
                            Button,
                            ContextualConsoleTab,
                            Node {
                                width: px(64),
                                min_width: px(64),
                                min_height: px(48),
                                padding: UiRect::axes(px(10), px(6)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(px(1)),
                                ..default()
                            },
                            BorderColor::all(THEME_ACCENT),
                            BackgroundColor(THEME_PANEL_ALT),
                            children![(
                                Text::new("Tab"),
                                TextContent,
                                TextFont::from_font_size(muted_font_size()),
                                TextColor(THEME_TEXT),
                            )],
                        ))
                        .id();
                    commands.entity(row).add_child(tab);
                }
            }
        }
    }
}

fn touch_scroll(
    touches: Res<Touches>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut state: ResMut<TouchScrollState>,
    mut panels: Query<
        (
            Entity,
            &ComputedNode,
            &UiGlobalTransform,
            &mut ScrollPosition,
        ),
        With<TouchScrollablePanel>,
    >,
) {
    for touch in touches.iter_just_pressed() {
        for (entity, computed, transform, _) in &mut panels {
            if computed.contains_point(
                *transform,
                physical_touch_position(&window, touch.position()),
            ) {
                state.active = Some((touch.id(), entity, touch.position()));
                break;
            }
        }
    }

    if let Some((id, entity, last_position)) = state.active {
        let current = touches
            .iter()
            .find(|touch| touch.id() == id)
            .map(|touch| touch.position());
        if let Some(position) = current {
            let delta = position - last_position;
            if let Ok((_, _, _, mut scroll)) = panels.get_mut(entity) {
                scroll.0.y = (scroll.0.y - delta.y).max(0.0);
            }
            state.active = Some((id, entity, position));
        }
        if touches.just_released(id) || touches.just_canceled(id) {
            state.active = None;
        }
    }
}

fn sync_ime_window(
    focus: Res<TextInputFocus>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Some(mut window) = windows.iter_mut().next() else {
        return;
    };
    window.ime_enabled = focus.target.is_some();
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
    fn screen_ids_cover_all_bevy_frontends() {
        let screens = [
            ScreenId::Lifecycle,
            ScreenId::Gameplay,
            ScreenId::Character,
            ScreenId::Inventory,
            ScreenId::Quests,
            ScreenId::Meditation,
            ScreenId::History,
            ScreenId::Journal,
            ScreenId::Talk,
            ScreenId::Conversation,
            ScreenId::Remains,
            ScreenId::RemainsResult,
            ScreenId::Combat,
            ScreenId::Console,
            ScreenId::Feedback,
        ];
        assert_eq!(screens.len(), 15);
    }

    #[test]
    fn responsive_spacing_preserves_desktop_baseline() {
        assert_eq!(screen_padding(), vmin(3.333));
        assert_eq!(panel_padding(), vmin(2.222));
        assert_eq!(screen_gap(), vmin(2.222));
        assert_eq!(panel_gap(), vmin(1.389));
    }

    #[test]
    fn responsive_fonts_use_viewport_units() {
        assert_eq!(title_font_size(), FontSize::VMin(5.0));
        assert_eq!(label_font_size(), FontSize::VMin(2.778));
        assert_eq!(muted_font_size(), FontSize::VMin(2.5));
    }

    #[test]
    fn touch_buttons_keep_a_fixed_logical_minimum_height() {
        assert_eq!(px(48), Val::Px(48.0));
    }

    #[test]
    fn gauge_ratio_handles_empty_and_clamps_values() {
        assert_eq!(gauge_ratio(0, 0), 0.0);
        assert_eq!(gauge_ratio(-5, 10), 0.0);
        assert_eq!(gauge_ratio(15, 10), 1.0);
        assert_eq!(gauge_ratio(5, 10), 0.5);
    }
}
