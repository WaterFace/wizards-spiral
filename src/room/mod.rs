use bevy::{math::vec2, prelude::*, utils::HashMap};
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

#[derive(Debug, Default)]
pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PersistentRoomState>()
            .add_systems(OnEnter(crate::states::GameState::InGame), spawn_room);
    }
}

#[derive(Debug, Default, Component)]
struct RoomObject;

#[derive(Debug, Default, Component)]
struct Floor;

#[derive(Debug, Default, Component)]
struct Obstacle;

#[derive(Debug, Component, Default)]
struct Spawner {
    ty: SpawnerType,
    index: usize,
    active: bool,
}

#[derive(Debug, Default, Resource)]
struct PersistentRoomState {
    rooms: HashMap<String, RoomState>,
}
#[derive(Debug, Default)]
struct RoomState {
    obstacles: Vec<ObstacleState>,
    spawners: Vec<SpawnerState>,
}

#[derive(Debug)]
struct ObstacleState {
    position: Vec2,
}

#[derive(Debug, Default, Clone, Copy)]
enum SpawnerType {
    #[default]
    Melee,
    Ranged,
}

/// Stores information about a spawner so it will behave consistently during a cycle
#[derive(Debug)]
struct SpawnerState {
    /// Where the spawner was placed
    position: Vec2,
    /// Is this spawner still active?
    /// A spawner is active if its corresponding enemy has not been killed this cycle
    active: bool,
    /// Which type of enemy this spawner spawns
    ty: SpawnerType,
}

#[derive(Bundle, Default)]
struct SpawnerBundle {
    transform: Transform,
    global_transform: GlobalTransform,
    spawner: Spawner,
    room_object: RoomObject,
}

#[derive(Bundle, Default)]
struct ObstacleBundle {
    sprite: Sprite,
    transform: Transform,
    global_transform: GlobalTransform,
    texture: Handle<Image>,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
    obstacle: Obstacle,
    room_object: RoomObject,
    collider: Collider,
}

#[derive(Bundle, Default)]
struct FloorBundle {
    sprite: Sprite,
    transform: Transform,
    global_transform: GlobalTransform,
    texture: Handle<Image>,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
    room_object: RoomObject,
}

fn spawn_room(
    mut commands: Commands,
    room_assets: Res<crate::assets::RoomAssets>,
    room_info_asset: Res<crate::assets::RoomInfoAsset>,
    room_infos: Res<Assets<crate::assets::RoomInfo>>,
    mut room_state: ResMut<PersistentRoomState>,
    mut rng: ResMut<crate::rand::GlobalRng>,
    mut working: Local<Vec<Vec2>>,
) {
    let Some(room_info) = room_infos.get(&room_info_asset.info) else {
        error!("spawn_room: RoomInfo should be loaded by now");
        panic!();
    };

    let current_room = room_info.name.as_str();
    commands.spawn((
        FloorBundle {
            texture: room_assets.background_texture.clone(),
            sprite: Sprite {
                custom_size: Some(room_info.rect.size()),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -10.0),
            ..Default::default()
        },
        ImageScaleMode::Tiled {
            tile_x: true,
            tile_y: true,
            stretch_value: 1.0,
        },
    ));

    if let Some(room_state) = room_state.rooms.get(current_room) {
        // room state found, spawn things according to the cached data
        for (index, spawner_state) in room_state.spawners.iter().enumerate() {
            commands.spawn(SpawnerBundle {
                transform: Transform::from_translation(spawner_state.position.extend(0.0)),
                spawner: Spawner {
                    index,
                    ty: spawner_state.ty,
                    active: spawner_state.active,
                },
                ..Default::default()
            });
        }
        for obstacle_state in room_state.obstacles.iter() {
            commands.spawn(ObstacleBundle {
                texture: room_assets.obstacle_texture.clone(),
                sprite: Sprite {
                    // custom_size: Some(vec2(32.0, 64.0)),
                    ..Default::default()
                },
                transform: Transform::from_translation(obstacle_state.position.extend(0.0)),
                collider: Collider::capsule_y(32.0, 16.0),
                ..Default::default()
            });
        }
    } else {
        // room state not found, spawn things freshly and cache the data
        let spawning_rectangle = {
            let room_rect = room_info.rect;
            // don't spawn things on the outer 10% of the room
            Rectangle {
                half_size: room_rect.half_size() * 0.95,
            }
        };

        let mut this_room_state = RoomState::default();

        // spawners:
        working.clear();
        working.extend(
            spawning_rectangle
                .interior_dist()
                .sample_iter(rng.as_deref_mut().as_mut())
                .take(room_info.num_enemies),
        );
        for (index, pos) in working.drain(..).enumerate() {
            let ty = SpawnerType::Melee;
            commands.spawn(SpawnerBundle {
                transform: Transform::from_translation(pos.extend(0.0)),
                spawner: Spawner {
                    index,
                    // TODO: generate both types of enemies
                    ty,
                    active: true,
                },
                ..Default::default()
            });
            this_room_state.spawners.push(SpawnerState {
                active: true,
                position: pos,
                ty,
            });
        }

        // obstacles:
        working.clear();
        working.extend(
            spawning_rectangle
                .interior_dist()
                .sample_iter(rng.as_deref_mut().as_mut())
                .take(room_info.num_obstacles),
        );
        for pos in working.drain(..) {
            commands.spawn(ObstacleBundle {
                texture: room_assets.obstacle_texture.clone(),
                sprite: Sprite {
                    custom_size: Some(vec2(32.0, 64.0)),
                    ..Default::default()
                },
                transform: Transform::from_translation(pos.extend(0.0)),
                collider: Collider::capsule_y(16.0, 16.0),
                ..Default::default()
            });

            this_room_state
                .obstacles
                .push(ObstacleState { position: pos });
        }

        let None = room_state
            .rooms
            .insert(room_info.name.clone(), this_room_state)
        else {
            panic!(
                "room_state map already has entry for {}, but we checked that it didn't!",
                room_info.name
            );
        };
    }

    // TODO: send event to say we're finished spawning?
}
