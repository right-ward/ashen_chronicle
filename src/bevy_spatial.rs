//! First playable local spatial-exploration view for dangerous locations.
//!
//! The map and session remain separate from Bevy presentation. This prototype
//! keeps them in a runtime resource; durable save/load integration belongs to
//! the follow-up persistence task.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::bevy_lifecycle::{LifecyclePhase, LifecycleState};
use crate::bevy_presentation::{
    self, BevyScreenRoot, GameplayInputQueue, NavigationState, ScreenId,
};
use crate::input::InputEvent;
use crate::model::{ExplorationPosition, GameState};
use crate::spatial::{
    generate_local_map, ExplorationSession, GridPosition, LocalMap, LocalMapConfig, Terrain,
};

const TILE_SIZE: f32 = 32.0;
const PLAYER_SPEED: f32 = 4.0;
const CAMERA_LERP_SPEED: f32 = 8.0;

#[derive(Component)]
struct SpatialWorldEntity;

#[derive(Component)]
struct SpatialPlayer;

#[derive(Component, Clone, Copy)]
struct SpatialTile(GridPosition);

#[derive(Component, Clone, Copy)]
struct SpatialDirection(Vec2);

#[derive(Component, Clone, Copy)]
enum SpatialAction {
    Interact,
    Exit,
}

#[derive(Resource, Default)]
pub(crate) struct SpatialExplorationState {
    maps: HashMap<String, LocalMap>,
    session: Option<ExplorationSession>,
    position: Vec2,
    message: Option<String>,
    dirty: bool,
}

pub(crate) fn install(app: &mut App) {
    app.init_resource::<SpatialExplorationState>().add_systems(
        Update,
        (spatial_input, update_spatial_world, render_spatial_screen).chain(),
    );
}

pub(crate) fn enter(
    state: &mut SpatialExplorationState,
    game: &GameState,
    navigation: &mut NavigationState,
) {
    let Some(location) = game.world.location_by_id(game.character.location_id) else {
        return;
    };
    let location_id = format!("location.{}", location.id);
    let world_seed = game
        .world
        .generation
        .map(|generation| generation.seed)
        .unwrap_or(game.world.id);

    let map = game
        .world
        .local_maps
        .iter()
        .find(|map| map.location_id == location_id)
        .cloned()
        .unwrap_or_else(|| {
            generate_local_map(world_seed, location_id.clone(), LocalMapConfig::default())
        });
    state.maps.insert(location_id.clone(), map);
    let map = state.maps.get(&location_id).expect("map was just inserted");
    let saved_position = game
        .character
        .exploration_session
        .as_ref()
        .filter(|session| session.location_id == location_id)
        .and(game.character.exploration_position)
        .map(|position| Vec2::new(position.x, position.y))
        .or_else(|| {
            game.character
                .exploration_session
                .as_ref()
                .filter(|session| session.location_id == location_id)
                .map(|session| {
                    Vec2::new(
                        f32::from(session.position.x) + 0.5,
                        f32::from(session.position.y) + 0.5,
                    )
                })
        });
    let position = saved_position
        .filter(|position| {
            position.x >= 0.0
                && position.y >= 0.0
                && map.is_walkable(GridPosition {
                    x: position.x.floor() as u16,
                    y: position.y.floor() as u16,
                })
        })
        .unwrap_or(Vec2::new(
            f32::from(map.entry.x) + 0.5,
            f32::from(map.entry.y) + 0.5,
        ));
    state.position = position;
    state.session = Some(ExplorationSession {
        location_id,
        position: GridPosition {
            x: position.x.floor().max(0.0) as u16,
            y: position.y.floor().max(0.0) as u16,
        },
    });
    map_mark_discovered(state, position);
    state.message = Some("The air grows still. Something waits among the ash.".to_string());
    state.dirty = true;
    navigation.return_screen = Some(ScreenId::Gameplay);
    navigation.current_screen = Some(ScreenId::SpatialExploration);
    navigation.selected = 0;
}

#[allow(clippy::too_many_arguments)] // Bevy systems require independent resources and queries.
fn spatial_input(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut lifecycle: ResMut<LifecycleState>,
    mut navigation: ResMut<NavigationState>,
    mut queue: ResMut<GameplayInputQueue>,
    mut state: ResMut<SpatialExplorationState>,
    direction_buttons: Query<(&Interaction, &SpatialDirection)>,
    action_buttons: Query<(&Interaction, &SpatialAction), Changed<Interaction>>,
) {
    if lifecycle.phase != LifecyclePhase::Complete
        || navigation.current_screen != Some(ScreenId::SpatialExploration)
    {
        return;
    }

    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyW) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    for (interaction, control) in &direction_buttons {
        if *interaction == Interaction::Pressed {
            direction += control.0;
        }
    }
    if direction.length_squared() > 0.0 {
        let Some(location_id) = state
            .session
            .as_ref()
            .map(|session| session.location_id.clone())
        else {
            return;
        };
        let Some(map) = state.maps.get(&location_id) else {
            return;
        };
        let next = advance_position(
            map,
            state.position,
            direction,
            time.delta_secs(),
            PLAYER_SPEED,
        );
        state.position = next;
        if let Some(session) = state.session.as_mut() {
            session.position = GridPosition {
                x: next.x.floor().max(0.0) as u16,
                y: next.y.floor().max(0.0) as u16,
            };
        }
        map_mark_discovered(&mut state, next);
    }

    if keyboard.just_pressed(KeyCode::KeyE) || keyboard.just_pressed(KeyCode::Enter) {
        interact_with_poi(&mut state, &mut lifecycle);
    }
    if keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::Backspace) {
        exit_spatial(&mut state, &mut navigation);
    }

    for (interaction, action) in &action_buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            SpatialAction::Interact => interact_with_poi(&mut state, &mut lifecycle),
            SpatialAction::Exit => exit_spatial(&mut state, &mut navigation),
        }
    }

    let events = std::mem::take(&mut queue.0);
    for event in events {
        if event == InputEvent::Cancel {
            exit_spatial(&mut state, &mut navigation);
        }
    }

    sync_spatial_to_game(&state, &mut lifecycle);
}

fn sync_spatial_to_game(state: &SpatialExplorationState, lifecycle: &mut LifecycleState) {
    let (Some(session), Some(game_session)) = (state.session.as_ref(), lifecycle.session.as_mut())
    else {
        return;
    };
    let Some(map) = state.maps.get(&session.location_id) else {
        return;
    };
    if let Some(existing) = game_session
        .state
        .world
        .local_maps
        .iter_mut()
        .find(|existing| existing.location_id == map.location_id)
    {
        if existing != map {
            *existing = map.clone();
        }
    } else {
        game_session.state.world.local_maps.push(map.clone());
    }
    if game_session.state.character.exploration_session.as_ref() != Some(session) {
        game_session.state.character.exploration_session = Some(session.clone());
    }
    let position = ExplorationPosition {
        x: state.position.x,
        y: state.position.y,
    };
    if game_session.state.character.exploration_position != Some(position) {
        game_session.state.character.exploration_position = Some(position);
    }
}

fn session_position(state: &SpatialExplorationState) -> Vec2 {
    state.position
}

fn map_mark_discovered(state: &mut SpatialExplorationState, position: Vec2) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    if let Some(map) = state.maps.get_mut(&session.location_id) {
        let center = GridPosition {
            x: position.x.floor().max(0.0) as u16,
            y: position.y.floor().max(0.0) as u16,
        };
        for y in center.y.saturating_sub(2)..=center.y.saturating_add(2).min(map.height - 1) {
            for x in center.x.saturating_sub(2)..=center.x.saturating_add(2).min(map.width - 1) {
                map.mark_tile_discovered(GridPosition { x, y });
            }
        }
    }
}

fn interact_with_poi(state: &mut SpatialExplorationState, lifecycle: &mut LifecycleState) {
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let location_id = session.location_id.clone();
    let position = session_position(state);
    let Some(map) = state.maps.get_mut(&location_id) else {
        return;
    };
    let Some(poi) = map.points_of_interest.first_mut() else {
        state.message = Some("There is nothing here that answers your attention.".to_string());
        state.dirty = true;
        return;
    };
    let poi_position = Vec2::new(
        f32::from(poi.position.x) + 0.5,
        f32::from(poi.position.y) + 0.5,
    );
    if position.distance(poi_position) > 1.25 {
        state.message =
            Some("Nothing close enough to examine. Move toward the marked remains.".to_string());
        state.dirty = true;
        return;
    }
    if poi.interacted {
        state.message =
            Some("The remains offer nothing more. The wrongness is still there.".to_string());
        state.dirty = true;
        return;
    }

    poi.discovered = true;
    poi.interacted = true;
    let title = poi.title.clone();
    let description = poi.description.clone();
    if let Some(session) = lifecycle.session.as_mut() {
        let turn = session.state.character.turn;
        session.state.world.record_history(
            turn,
            format!(
                "{}: {} {}",
                title, description, "The discovery has been recorded."
            ),
        );
    }
    state.message = Some(format!(
        "{}. {} You have the distinct impression that something noticed.",
        title, description
    ));
    state.dirty = true;
}

fn exit_spatial(state: &mut SpatialExplorationState, navigation: &mut NavigationState) {
    state.message = None;
    state.dirty = true;
    navigation.current_screen = Some(ScreenId::Gameplay);
    navigation.return_screen = None;
    navigation.selected = 0;
}

#[allow(clippy::too_many_arguments)] // Bevy systems require independent resources and queries.
fn update_spatial_world(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<SpatialExplorationState>,
    navigation: Res<NavigationState>,
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<SpatialPlayer>)>,
    mut player: Query<&mut Transform, With<SpatialPlayer>>,
    mut tiles: Query<(&SpatialTile, &mut Sprite)>,
    world_entities: Query<Entity, With<SpatialWorldEntity>>,
) {
    if navigation.current_screen != Some(ScreenId::SpatialExploration) {
        for entity in &world_entities {
            commands.entity(entity).despawn();
        }
        return;
    }
    let Some(session) = state.session.as_ref() else {
        return;
    };
    let Some(map) = state.maps.get(&session.location_id) else {
        return;
    };
    let target = map_to_world(session_position(&state), Some(map));
    for (tile, mut sprite) in &mut tiles {
        let terrain = map.terrain_at(tile.0).unwrap_or(Terrain::Wall);
        sprite.color = terrain_color(terrain, map.is_tile_discovered(tile.0));
    }
    for mut transform in &mut player {
        transform.translation.x = target.x;
        transform.translation.y = target.y;
    }
    for mut transform in &mut camera {
        let amount = (CAMERA_LERP_SPEED * time.delta_secs()).clamp(0.0, 1.0);
        transform.translation.x += (target.x - transform.translation.x) * amount;
        transform.translation.y += (target.y - transform.translation.y) * amount;
    }
}

fn render_spatial_screen(
    mut commands: Commands,
    lifecycle: Res<LifecycleState>,
    navigation: Res<NavigationState>,
    mut state: ResMut<SpatialExplorationState>,
    roots: Query<Entity, With<BevyScreenRoot>>,
    world_entities: Query<Entity, With<SpatialWorldEntity>>,
) {
    if lifecycle.phase != LifecyclePhase::Complete
        || navigation.current_screen != Some(ScreenId::SpatialExploration)
        || !state.dirty
    {
        return;
    }
    for root in &roots {
        commands.entity(root).despawn();
    }
    for entity in &world_entities {
        commands.entity(entity).despawn();
    }

    let Some(session) = state.session.as_ref() else {
        return;
    };
    let Some(map) = state.maps.get(&session.location_id) else {
        return;
    };

    for y in 0..map.height {
        for x in 0..map.width {
            let position = GridPosition { x, y };
            let terrain = map.terrain_at(position).unwrap_or(Terrain::Wall);
            let color = terrain_color(terrain, map.is_tile_discovered(position));
            let world_position =
                map_to_world(Vec2::new(f32::from(x) + 0.5, f32::from(y) + 0.5), Some(map));
            commands.spawn((
                SpatialWorldEntity,
                SpatialTile(position),
                Sprite::from_color(color, Vec2::splat(TILE_SIZE - 1.0)),
                Transform::from_xyz(world_position.x, world_position.y, 0.0),
            ));
        }
    }

    if let Some(poi) = map.points_of_interest.first() {
        let poi_position = map_to_world(
            Vec2::new(
                f32::from(poi.position.x) + 0.5,
                f32::from(poi.position.y) + 0.5,
            ),
            Some(map),
        );
        commands.spawn((
            SpatialWorldEntity,
            Sprite::from_color(
                if poi.interacted {
                    Color::srgb(0.38, 0.27, 0.35)
                } else {
                    Color::srgb(0.75, 0.45, 0.68)
                },
                Vec2::splat(TILE_SIZE * 0.55),
            ),
            Transform::from_xyz(poi_position.x, poi_position.y, 1.0),
        ));
    }

    let player_position = map_to_world(session_position(&state), Some(map));
    commands.spawn((
        SpatialWorldEntity,
        SpatialPlayer,
        Sprite::from_color(Color::srgb(0.88, 0.83, 0.92), Vec2::splat(TILE_SIZE * 0.52)),
        Transform::from_xyz(player_position.x, player_position.y, 2.0),
    ));

    let root = commands
        .spawn((
            BevyScreenRoot,
            Node {
                width: percent(100),
                height: percent(100),
                min_width: px(0),
                min_height: px(0),
                padding: UiRect::all(vmin(2.0)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .id();

    let header = commands
        .spawn(Node {
            width: percent(100),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .id();
    commands.entity(root).add_child(header);
    let location_name = lifecycle
        .session
        .as_ref()
        .and_then(|session| {
            session
                .state
                .world
                .location_by_id(session.state.character.location_id)
        })
        .map(|location| location.name.to_uppercase())
        .unwrap_or_else(|| "DANGEROUS LOCATION".to_string());
    spawn_label(
        &mut commands,
        header,
        format!("{location_name} — ASHEN GROUND"),
    );
    spawn_control(
        &mut commands,
        header,
        "Leave",
        bevy_spatial_action(SpatialAction::Exit),
    );

    let status = commands
        .spawn((
            Node {
                width: percent(100),
                padding: UiRect::all(px(8)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.07, 0.045, 0.08, 0.88)),
        ))
        .id();
    commands.entity(root).add_child(status);
    spawn_label(
        &mut commands,
        status,
        state
            .message
            .as_deref()
            .unwrap_or("Move carefully. The walls are not all stone."),
    );

    let footer = commands
        .spawn(Node {
            width: percent(100),
            flex_grow: 1.0,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::End,
            ..default()
        })
        .id();
    commands.entity(root).add_child(footer);

    let pad = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4),
            ..default()
        })
        .id();
    commands.entity(footer).add_child(pad);
    spawn_control(&mut commands, pad, "↑", bevy_spatial_direction(Vec2::NEG_Y));
    let middle = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: px(4),
            ..default()
        })
        .id();
    commands.entity(pad).add_child(middle);
    spawn_control(
        &mut commands,
        middle,
        "←",
        bevy_spatial_direction(Vec2::NEG_X),
    );
    spawn_control(&mut commands, middle, "↓", bevy_spatial_direction(Vec2::Y));
    spawn_control(&mut commands, middle, "→", bevy_spatial_direction(Vec2::X));

    spawn_control(
        &mut commands,
        footer,
        if map
            .points_of_interest
            .first()
            .is_some_and(|poi| poi.interacted)
        {
            "Examine again (E)"
        } else {
            "Examine (E)"
        },
        bevy_spatial_action(SpatialAction::Interact),
    );
    state.dirty = false;
}

fn bevy_spatial_direction(direction: Vec2) -> SpatialControlKind {
    SpatialControlKind::Direction(direction)
}

fn bevy_spatial_action(action: SpatialAction) -> SpatialControlKind {
    SpatialControlKind::Action(action)
}

#[derive(Clone, Copy)]
enum SpatialControlKind {
    Direction(Vec2),
    Action(SpatialAction),
}

fn spawn_control(
    commands: &mut Commands,
    parent: Entity,
    label: impl Into<String>,
    kind: SpatialControlKind,
) -> Entity {
    let mut entity = commands.spawn((
        Button,
        Node {
            min_width: px(52),
            min_height: px(48),
            padding: UiRect::axes(px(10), px(6)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(1)),
            ..default()
        },
        BorderColor::all(bevy_presentation::THEME_BORDER),
        BackgroundColor(bevy_presentation::THEME_SURFACE_STRONG),
    ));
    match kind {
        SpatialControlKind::Direction(direction) => {
            entity.insert(SpatialDirection(direction));
        }
        SpatialControlKind::Action(action) => {
            entity.insert(action);
        }
    }
    let entity = entity.id();
    let text = commands
        .spawn((
            Text::new(label.into()),
            TextFont::from_font_size(FontSize::VMin(2.5)),
            TextColor(bevy_presentation::THEME_TEXT),
        ))
        .id();
    commands.entity(entity).add_child(text);
    commands.entity(parent).add_child(entity);
    entity
}

fn spawn_label(commands: &mut Commands, parent: Entity, label: impl Into<String>) {
    let label = commands
        .spawn((
            Text::new(label.into()),
            TextFont::from_font_size(FontSize::VMin(2.5)),
            TextColor(bevy_presentation::THEME_TEXT),
        ))
        .id();
    commands.entity(parent).add_child(label);
}

fn terrain_color(terrain: Terrain, discovered: bool) -> Color {
    if !discovered {
        return Color::srgb(0.055, 0.045, 0.07);
    }
    match terrain {
        Terrain::Floor => Color::srgb(0.25, 0.19, 0.27),
        Terrain::Ash => Color::srgb(0.33, 0.26, 0.34),
        Terrain::Rubble => Color::srgb(0.21, 0.17, 0.24),
        Terrain::Wall => Color::srgb(0.10, 0.075, 0.13),
    }
}

fn map_to_world(position: Vec2, map: Option<&LocalMap>) -> Vec2 {
    let (width, height) = map
        .map(|map| (f32::from(map.width), f32::from(map.height)))
        .unwrap_or((24.0, 16.0));
    Vec2::new(
        (position.x - width / 2.0) * TILE_SIZE,
        (height / 2.0 - position.y) * TILE_SIZE,
    )
}

fn advance_position(
    map: &LocalMap,
    position: Vec2,
    direction: Vec2,
    delta_secs: f32,
    speed: f32,
) -> Vec2 {
    if direction.length_squared() == 0.0 {
        return position;
    }
    let delta = direction.normalize() * speed * delta_secs.clamp(0.0, 0.1);
    let mut result = position;
    let candidate_x = result.x + delta.x;
    if is_position_walkable(map, Vec2::new(candidate_x, result.y)) {
        result.x = candidate_x;
    }
    let candidate_y = result.y + delta.y;
    if is_position_walkable(map, Vec2::new(result.x, candidate_y)) {
        result.y = candidate_y;
    }
    result
}

fn is_position_walkable(map: &LocalMap, position: Vec2) -> bool {
    if position.x < 0.0 || position.y < 0.0 {
        return false;
    }
    map.is_walkable(GridPosition {
        x: position.x.floor() as u16,
        y: position.y.floor() as u16,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_is_continuous_within_walkable_tiles() {
        let map = generate_local_map(4, "location.test", LocalMapConfig::default());
        let start = Vec2::new(f32::from(map.entry.x) + 0.5, f32::from(map.entry.y) + 0.5);
        let moved = advance_position(&map, start, Vec2::X, 0.1, PLAYER_SPEED);
        assert!(moved.x > start.x);
        assert!((moved.x - start.x - 0.4).abs() < 0.001);
    }

    #[test]
    fn movement_does_not_cross_blocked_terrain() {
        let mut map = generate_local_map(4, "location.test", LocalMapConfig::default());
        let start = Vec2::new(f32::from(map.entry.x) + 0.8, f32::from(map.entry.y) + 0.5);
        let wall = GridPosition {
            x: map.entry.x + 1,
            y: map.entry.y,
        };
        map.set_terrain(wall, Terrain::Wall);
        let moved = advance_position(&map, start, Vec2::X, 0.1, PLAYER_SPEED);
        assert_eq!(moved, start);
    }
}
