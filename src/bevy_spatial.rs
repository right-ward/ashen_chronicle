//! First playable local spatial-exploration view for dangerous locations.
//!
//! The map and session remain separate from Bevy presentation. This prototype
//! keeps them in a runtime resource; durable save/load integration belongs to
//! the follow-up persistence task.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
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

const TILE_HALF_WIDTH: f32 = 24.0;
const TILE_HALF_HEIGHT: f32 = 12.0;
const WALL_HEIGHT: f32 = 24.0;
const PLAYER_SPRITE_HEIGHT: f32 = 24.0;
const PLAYER_COLLISION_HALF_EXTENT: f32 = 0.15;
const INTERACTION_RANGE_PX: f32 = 34.0;
const PLAYER_SPEED: f32 = 180.0; // Screen-space pixels per second.
const CAMERA_LERP_SPEED: f32 = 8.0;
const CAMERA_DEAD_ZONE_FRACTION: f32 = 0.15;
const DEPTH_STEP: f32 = 0.08;
const PLAYER_DEPTH_OFFSET: f32 = 0.04;

#[derive(Component)]
struct SpatialWorldEntity;

#[derive(Component)]
struct SpatialPlayer;

#[derive(Component, Clone, Copy)]
struct SpatialTile(GridPosition);

#[derive(Component, Clone, Copy)]
struct SpatialWallSide {
    tile: GridPosition,
    right_face: bool,
}

/// A screen-space direction from desktop keys or the touch D-pad.
#[derive(Component, Clone, Copy)]
struct SpatialDirection(Vec2);

#[derive(Component)]
struct SpatialControlButton;

#[derive(Component)]
struct SpatialInteractionPrompt;

#[derive(Resource)]
struct SpatialRenderAssets {
    diamond_mesh: Handle<Mesh>,
    wall_left_mesh: Handle<Mesh>,
    wall_right_mesh: Handle<Mesh>,
    tile_materials: [Handle<ColorMaterial>; 5],
    wall_face_materials: [Handle<ColorMaterial>; 3],
}

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
    app.init_resource::<SpatialExplorationState>()
        .add_systems(Startup, setup_spatial_render_assets)
        .add_systems(
            Update,
            (
                spatial_input,
                update_spatial_world,
                update_spatial_context_prompt,
                update_spatial_control_feedback,
                render_spatial_screen,
            )
                .chain(),
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

fn setup_spatial_render_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let diamond_mesh = meshes.add(mesh_from_triangles(
        [
            [0.0, TILE_HALF_HEIGHT, 0.0],
            [0.0, -TILE_HALF_HEIGHT, 0.0],
            [TILE_HALF_WIDTH, 0.0, 0.0],
            [0.0, TILE_HALF_HEIGHT, 0.0],
            [-TILE_HALF_WIDTH, 0.0, 0.0],
            [0.0, -TILE_HALF_HEIGHT, 0.0],
        ],
        [
            [0.5, 1.0], [0.5, 0.0], [1.0, 0.5],
            [0.5, 1.0], [0.0, 0.5], [0.5, 0.0],
        ],
    ));
    let wall_left_mesh = meshes.add(mesh_from_triangles(
        [
            [-TILE_HALF_WIDTH, WALL_HEIGHT, 0.0],
            [-TILE_HALF_WIDTH, 0.0, 0.0],
            [0.0, -TILE_HALF_HEIGHT, 0.0],
            [-TILE_HALF_WIDTH, WALL_HEIGHT, 0.0],
            [0.0, -TILE_HALF_HEIGHT, 0.0],
            [0.0, WALL_HEIGHT - TILE_HALF_HEIGHT, 0.0],
        ],
        [
            [0.0, 1.0], [0.0, 0.0], [1.0, 0.0],
            [0.0, 1.0], [1.0, 0.0], [1.0, 1.0],
        ],
    ));
    let wall_right_mesh = meshes.add(mesh_from_triangles(
        [
            [TILE_HALF_WIDTH, WALL_HEIGHT, 0.0],
            [0.0, WALL_HEIGHT - TILE_HALF_HEIGHT, 0.0],
            [0.0, -TILE_HALF_HEIGHT, 0.0],
            [TILE_HALF_WIDTH, WALL_HEIGHT, 0.0],
            [0.0, -TILE_HALF_HEIGHT, 0.0],
            [TILE_HALF_WIDTH, 0.0, 0.0],
        ],
        [
            [1.0, 1.0], [0.0, 1.0], [0.0, 0.0],
            [1.0, 1.0], [0.0, 0.0], [1.0, 0.0],
        ],
    ));

    let unseen = terrain_color(Terrain::Floor, false);
    let tile_materials = [
        materials.add(ColorMaterial::from(unseen)),
        materials.add(ColorMaterial::from(terrain_color(Terrain::Floor, true))),
        materials.add(ColorMaterial::from(terrain_color(Terrain::Ash, true))),
        materials.add(ColorMaterial::from(terrain_color(Terrain::Rubble, true))),
        materials.add(ColorMaterial::from(terrain_color(Terrain::Wall, true))),
    ];
    let wall_face_materials = [
        materials.add(ColorMaterial::from(unseen)),
        materials.add(ColorMaterial::from(Color::srgb(0.16, 0.115, 0.10))),
        materials.add(ColorMaterial::from(Color::srgb(0.105, 0.075, 0.075))),
    ];

    commands.insert_resource(SpatialRenderAssets {
        diamond_mesh,
        wall_left_mesh,
        wall_right_mesh,
        tile_materials,
        wall_face_materials,
    });
}

fn mesh_from_triangles(vertices: [[f32; 3]; 6], uvs: [[f32; 2]; 6]) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.to_vec());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.to_vec());
    mesh
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
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
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
        let world_direction = screen_direction_to_world(direction);
        let next = advance_position(
            map,
            state.position,
            world_direction,
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
    let Some(map) = state.maps.get(&location_id) else {
        return;
    };
    if map.points_of_interest.is_empty() {
        state.message = Some("There is nothing here that answers your attention.".to_string());
        state.dirty = true;
        return;
    }
    if !poi_interaction_available(map, position) {
        state.message = Some("Move closer to the remains and find a clear approach.".to_string());
        state.dirty = true;
        return;
    }
    let Some(map) = state.maps.get_mut(&location_id) else {
        return;
    };
    let Some(poi) = map.points_of_interest.first_mut() else {
        return;
    };
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
    assets: Res<SpatialRenderAssets>,
    mut camera: Query<(&mut Transform, &Camera), (With<Camera2d>, Without<SpatialPlayer>)>,
    mut player: Query<&mut Transform, With<SpatialPlayer>>,
    mut tiles: Query<(&SpatialTile, &mut MeshMaterial2d<ColorMaterial>), Without<SpatialWallSide>>,
    mut wall_sides: Query<(&SpatialWallSide, &mut MeshMaterial2d<ColorMaterial>), Without<SpatialTile>>,
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
    let position = session_position(&state);
    let target = map_to_world(position, Some(map));
    for (tile, mut material) in &mut tiles {
        let terrain = map.terrain_at(tile.0).unwrap_or(Terrain::Wall);
        let desired = assets.tile_materials[
            terrain_material_index(terrain, map.is_tile_discovered(tile.0))
        ].clone();
        if material.0 != desired {
            material.0 = desired;
        }
    }
    for (side, mut material) in &mut wall_sides {
        let discovered = map.is_tile_discovered(side.tile);
        let index = if !discovered { 0 } else if side.right_face { 2 } else { 1 };
        let desired = assets.wall_face_materials[index].clone();
        if material.0 != desired {
            material.0 = desired;
        }
    }
    for mut transform in &mut player {
        transform.translation.x = target.x;
        transform.translation.y = target.y + PLAYER_SPRITE_HEIGHT * 0.5;
        transform.translation.z = depth_for(position) + PLAYER_DEPTH_OFFSET;
    }
    for (mut transform, camera_component) in &mut camera {
        let viewport = camera_component.logical_viewport_size().unwrap_or(Vec2::ZERO);
        let current = transform.translation.truncate();
        let dead_zone_target = camera_target_with_dead_zone(target, current, viewport);
        let camera_target = clamp_camera_target(dead_zone_target, map, viewport);
        let amount = (CAMERA_LERP_SPEED * time.delta_secs()).clamp(0.0, 1.0);
        transform.translation.x += (camera_target.x - transform.translation.x) * amount;
        transform.translation.y += (camera_target.y - transform.translation.y) * amount;
    }
}

fn render_spatial_screen(
    mut commands: Commands,
    lifecycle: Res<LifecycleState>,
    navigation: Res<NavigationState>,
    assets: Res<SpatialRenderAssets>,
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
            let discovered = map.is_tile_discovered(position);
            let tile_center = Vec2::new(f32::from(x) + 0.5, f32::from(y) + 0.5);
            let world_position = map_to_world(tile_center, Some(map));
            let depth = depth_for(tile_center);

            if terrain == Terrain::Wall {
                let wall_material_index = if discovered { 1 } else { 0 };
                commands.spawn((
                    SpatialWorldEntity,
                    SpatialWallSide { tile: position, right_face: false },
                    Mesh2d(assets.wall_left_mesh.clone()),
                    MeshMaterial2d(assets.wall_face_materials[wall_material_index].clone()),
                    Transform::from_xyz(world_position.x, world_position.y, depth + 0.01),
                ));
                commands.spawn((
                    SpatialWorldEntity,
                    SpatialWallSide { tile: position, right_face: true },
                    Mesh2d(assets.wall_right_mesh.clone()),
                    MeshMaterial2d(assets.wall_face_materials[if discovered { 2 } else { 0 }].clone()),
                    Transform::from_xyz(world_position.x, world_position.y, depth + 0.01),
                ));
            }

            let material_index = terrain_material_index(terrain, discovered);
            let top_offset = if terrain == Terrain::Wall { WALL_HEIGHT } else { 0.0 };
            let top_depth = depth + if terrain == Terrain::Wall { 0.02 } else { 0.0 };
            commands.spawn((
                SpatialWorldEntity,
                SpatialTile(position),
                Mesh2d(assets.diamond_mesh.clone()),
                MeshMaterial2d(assets.tile_materials[material_index].clone()),
                Transform::from_xyz(
            world_position.x,
            world_position.y + top_offset,
            top_depth,
        ),
            ));
        }
    }

    if let Some(poi) = map.points_of_interest.first() {
        let poi_grid = Vec2::new(
            f32::from(poi.position.x) + 0.5,
            f32::from(poi.position.y) + 0.5,
        );
        let poi_position = map_to_world(poi_grid, Some(map));
        commands.spawn((
            SpatialWorldEntity,
            Sprite::from_color(
                if poi.interacted {
                    Color::srgb(0.38, 0.27, 0.35)
                } else {
                    Color::srgb(0.75, 0.45, 0.68)
                },
                Vec2::new(18.0, 24.0),
            ),
            Transform::from_xyz(
                poi_position.x,
                poi_position.y + PLAYER_SPRITE_HEIGHT * 0.5,
                depth_for(poi_grid) + PLAYER_DEPTH_OFFSET,
            ),
        ));
    }

    let player_ground_position = session_position(&state);
    let player_position = map_to_world(player_ground_position, Some(map));
    commands.spawn((
        SpatialWorldEntity,
        SpatialPlayer,
        Sprite::from_color(
            Color::srgb(0.88, 0.83, 0.78),
            Vec2::new(12.0, PLAYER_SPRITE_HEIGHT),
        ),
        Transform::from_xyz(
            player_position.x,
            player_position.y + PLAYER_SPRITE_HEIGHT * 0.5,
            depth_for(player_ground_position) + PLAYER_DEPTH_OFFSET,
        ),
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
        Display::Flex,
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
    spawn_control(&mut commands, pad, "W", bevy_spatial_direction(Vec2::Y), Display::Flex);
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
        "A",
        bevy_spatial_direction(Vec2::NEG_X),
        Display::Flex,
    );
    spawn_control(&mut commands, middle, "S", bevy_spatial_direction(Vec2::NEG_Y), Display::Flex);
    spawn_control(&mut commands, middle, "D", bevy_spatial_direction(Vec2::X), Display::Flex);

    let interaction_display = if poi_interaction_available(map, session_position(&state)) {
        Display::Flex
    } else {
        Display::None
    };
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
        interaction_display,
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
    display: Display,
) -> Entity {
    let mut entity = commands.spawn((
        Button,
        SpatialControlButton,
        Node {
            display,
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
        SpatialControlKind::Action(SpatialAction::Interact) => {
            entity.insert((SpatialAction::Interact, SpatialInteractionPrompt));
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

fn update_spatial_context_prompt(
    state: Res<SpatialExplorationState>,
    navigation: Res<NavigationState>,
    lifecycle: Res<LifecycleState>,
    mut prompts: Query<&mut Node, With<SpatialInteractionPrompt>>,
) {
    if lifecycle.phase != LifecyclePhase::Complete
        || navigation.current_screen != Some(ScreenId::SpatialExploration)
    {
        return;
    }
    let available = state
        .session
        .as_ref()
        .and_then(|session| state.maps.get(&session.location_id))
        .is_some_and(|map| poi_interaction_available(map, session_position(&state)));
    let display = if available { Display::Flex } else { Display::None };
    for mut node in &mut prompts {
        if node.display != display {
            node.display = display;
        }
    }
}

fn update_spatial_control_feedback(
    mut controls: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<SpatialControlButton>),
    >,
) {
    for (interaction, mut background) in &mut controls {
        background.0 = match *interaction {
            Interaction::Pressed => bevy_presentation::THEME_PRESSED,
            Interaction::Hovered => bevy_presentation::THEME_HOVER,
            Interaction::None => bevy_presentation::THEME_SURFACE_STRONG,
        };
    }
}

fn terrain_color(terrain: Terrain, discovered: bool) -> Color {
    if !discovered {
        return Color::srgb(0.055, 0.045, 0.07);
    }
    match terrain {
        Terrain::Floor => Color::srgb(0.30, 0.23, 0.16),
        Terrain::Ash => Color::srgb(0.27, 0.25, 0.29),
        Terrain::Rubble => Color::srgb(0.22, 0.19, 0.17),
        Terrain::Wall => Color::srgb(0.13, 0.10, 0.095),
    }
}

fn terrain_material_index(terrain: Terrain, discovered: bool) -> usize {
    if !discovered {
        return 0;
    }
    match terrain {
        Terrain::Floor => 1,
        Terrain::Ash => 2,
        Terrain::Rubble => 3,
        Terrain::Wall => 4,
    }
}

fn map_to_world(position: Vec2, map: Option<&LocalMap>) -> Vec2 {
    let (width, height) = map
        .map(|map| (f32::from(map.width), f32::from(map.height)))
        .unwrap_or((24.0, 16.0));
    Vec2::new(
        (position.x - position.y - (width - height) * 0.5) * TILE_HALF_WIDTH,
        ((width + height) * 0.5 - position.x - position.y) * TILE_HALF_HEIGHT,
    )
}

fn depth_for(position: Vec2) -> f32 {
    (position.x + position.y) * DEPTH_STEP
}

fn screen_direction_to_world(screen_direction: Vec2) -> Vec2 {
    if screen_direction.length_squared() == 0.0 {
        return Vec2::ZERO;
    }
    let direction = screen_direction.normalize();
    Vec2::new(
        direction.x / (2.0 * TILE_HALF_WIDTH) - direction.y / (2.0 * TILE_HALF_HEIGHT),
        -direction.x / (2.0 * TILE_HALF_WIDTH) - direction.y / (2.0 * TILE_HALF_HEIGHT),
    )
}

fn camera_target_with_dead_zone(player: Vec2, camera: Vec2, viewport: Vec2) -> Vec2 {
    if viewport.x <= 0.0 || viewport.y <= 0.0 {
        return player;
    }

    let half_dead_zone = viewport * (CAMERA_DEAD_ZONE_FRACTION * 0.5);
    let offset = player - camera;
    let correction = Vec2::new(
        if offset.x > half_dead_zone.x {
            offset.x - half_dead_zone.x
        } else if offset.x < -half_dead_zone.x {
            offset.x + half_dead_zone.x
        } else {
            0.0
        },
        if offset.y > half_dead_zone.y {
            offset.y - half_dead_zone.y
        } else if offset.y < -half_dead_zone.y {
            offset.y + half_dead_zone.y
        } else {
            0.0
        },
    );
    camera + correction
}

fn clamp_camera_axis(target: f32, min: f32, max: f32, viewport_half_extent: f32) -> f32 {
    let lower = min + viewport_half_extent;
    let upper = max - viewport_half_extent;
    if lower > upper {
        (min + max) * 0.5
    } else {
        target.clamp(lower, upper)
    }
}

fn clamp_camera_target(target: Vec2, map: &LocalMap, viewport: Vec2) -> Vec2 {
    let map_half_width =
        (f32::from(map.width) + f32::from(map.height)) * TILE_HALF_WIDTH * 0.5;
    let map_half_height =
        (f32::from(map.width) + f32::from(map.height)) * TILE_HALF_HEIGHT * 0.5;
    // Wall tops extend farther upward than the floor tiles extend downward.
    let min_y = -map_half_height - (WALL_HEIGHT - TILE_HALF_HEIGHT);
    let max_y = map_half_height;
    Vec2::new(
        clamp_camera_axis(
            target.x,
            -map_half_width,
            map_half_width,
            viewport.x * 0.5,
        ),
        clamp_camera_axis(target.y, min_y, max_y, viewport.y * 0.5),
    )
}

fn poi_interaction_available(map: &LocalMap, position: Vec2) -> bool {
    let Some(poi) = map.points_of_interest.first() else {
        return false;
    };
    let target = Vec2::new(
        f32::from(poi.position.x) + 0.5,
        f32::from(poi.position.y) + 0.5,
    );
    if map_to_world(position, Some(map)).distance(map_to_world(target, Some(map)))
        > INTERACTION_RANGE_PX
    {
        return false;
    }
    has_line_of_sight(map, position, target)
}

fn has_line_of_sight(map: &LocalMap, start: Vec2, target: Vec2) -> bool {
    let distance = start.distance(target);
    let steps = (distance * 8.0).ceil() as u32;
    for step in 1..steps {
        let sample = start.lerp(target, step as f32 / steps.max(1) as f32);
        if sample.x < 0.0 || sample.y < 0.0 {
            return false;
        }
        let tile = GridPosition {
            x: sample.x.floor() as u16,
            y: sample.y.floor() as u16,
        };
        if !map.is_walkable(tile) {
            return false;
        }
    }
    true
}

fn advance_position(
    map: &LocalMap,
    position: Vec2,
    world_units_per_pixel: Vec2,
    delta_secs: f32,
    speed_pixels_per_second: f32,
) -> Vec2 {
    if world_units_per_pixel.length_squared() == 0.0 {
        return position;
    }
    let delta =
        world_units_per_pixel * speed_pixels_per_second * delta_secs.clamp(0.0, 0.1);
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
    let radius = PLAYER_COLLISION_HALF_EXTENT;
    let samples = [
        Vec2::ZERO,
        Vec2::new(-radius, 0.0),
        Vec2::new(radius, 0.0),
        Vec2::new(0.0, -radius),
        Vec2::new(0.0, radius),
        Vec2::new(-radius, -radius),
        Vec2::new(-radius, radius),
        Vec2::new(radius, -radius),
        Vec2::new(radius, radius),
    ];
    samples.into_iter().all(|offset| {
        let sample = position + offset;
        if sample.x < 0.0
            || sample.y < 0.0
            || sample.x >= f32::from(map.width)
            || sample.y >= f32::from(map.height)
        {
            return false;
        }
        map.is_walkable(GridPosition {
            x: sample.x.floor() as u16,
            y: sample.y.floor() as u16,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angled_projection_offsets_neighboring_tiles_consistently() {
        let map = generate_local_map(4, "location.test", LocalMapConfig::default());
        let origin = map_to_world(Vec2::new(5.5, 5.5), Some(&map));
        let x_neighbor = map_to_world(Vec2::new(6.5, 5.5), Some(&map));
        let y_neighbor = map_to_world(Vec2::new(5.5, 6.5), Some(&map));
        assert!((x_neighbor.x - origin.x - TILE_HALF_WIDTH).abs() < 0.001);
        assert!((x_neighbor.y - origin.y + TILE_HALF_HEIGHT).abs() < 0.001);
        assert!((y_neighbor.x - origin.x + TILE_HALF_WIDTH).abs() < 0.001);
        assert!((y_neighbor.y - origin.y + TILE_HALF_HEIGHT).abs() < 0.001);
    }

    #[test]
    fn projected_input_axes_match_screen_directions() {
        for screen_direction in [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y] {
            let world = screen_direction_to_world(screen_direction);
            let projected = Vec2::new(
                (world.x - world.y) * TILE_HALF_WIDTH,
                -(world.x + world.y) * TILE_HALF_HEIGHT,
            );
            assert!((projected - screen_direction).length() < 0.001);
        }
    }

    #[test]
    fn camera_dead_zone_does_not_follow_small_player_movements() {
        let camera = Vec2::new(20.0, -10.0);
        let viewport = Vec2::new(800.0, 450.0);
        let player = camera + Vec2::new(40.0, -20.0);
        assert_eq!(camera_target_with_dead_zone(player, camera, viewport), camera);
    }

    #[test]
    fn camera_dead_zone_moves_only_by_the_excess_offset() {
        let camera = Vec2::ZERO;
        let viewport = Vec2::new(800.0, 450.0);
        let player = Vec2::new(200.0, -100.0);
        let target = camera_target_with_dead_zone(player, camera, viewport);
        assert!((target.x - 140.0).abs() < 0.001);
        assert!((target.y + 66.25).abs() < 0.001);
    }

    #[test]
    fn camera_bounds_center_maps_smaller_than_the_viewport() {
        let map = generate_local_map(4, "location.test", LocalMapConfig {
            width: 8,
            height: 8,
        });
        let target = clamp_camera_target(Vec2::new(200.0, -100.0), &map, Vec2::new(2000.0, 1200.0));
        assert_eq!(target, Vec2::new(0.0, 0.0));
    }

    #[test]
    fn camera_bounds_keep_the_viewport_inside_large_map_extents() {
        let map = generate_local_map(4, "location.test", LocalMapConfig::default());
        let viewport = Vec2::new(320.0, 180.0);
        let target = clamp_camera_target(Vec2::new(10000.0, -10000.0), &map, viewport);
        let half_width =
            (f32::from(map.width) + f32::from(map.height)) * TILE_HALF_WIDTH * 0.5;
        let half_height =
            (f32::from(map.width) + f32::from(map.height)) * TILE_HALF_HEIGHT * 0.5;
        assert!(target.x <= half_width - viewport.x * 0.5 + 0.001);
        assert!(target.x >= -half_width + viewport.x * 0.5 - 0.001);
        assert!(target.y <= half_height - viewport.y * 0.5 + 0.001);
        assert!(target.y >= -half_height - (WALL_HEIGHT - TILE_HALF_HEIGHT) + viewport.y * 0.5 - 0.001);
    }

    #[test]
    fn contextual_interaction_requires_range_and_clear_approach() {
        let map = generate_local_map(4, "location.test", LocalMapConfig::default());
        let poi = &map.points_of_interest[0];
        let near = Vec2::new(f32::from(poi.position.x) + 0.5, f32::from(poi.position.y) + 0.5);
        assert!(poi_interaction_available(&map, near));
        assert!(!poi_interaction_available(&map, near + Vec2::new(3.0, 0.0)));
    }

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
