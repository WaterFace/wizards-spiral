use bevy::{prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;

mod events;
mod spawn;

#[derive(Debug, Default)]
pub struct RoomPlugin;

impl Plugin for RoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<events::ChangeRoom>()
            .init_resource::<PersistentRoomState>()
            .add_systems(
                Update,
                events::handle_change_room.run_if(
                    in_state(crate::states::GameState::InGame)
                        .or_else(in_state(crate::states::GameState::MainMenu)),
                ),
            )
            .add_systems(
                OnEnter(crate::states::GameState::RoomTransition),
                (spawn::destroy_room, spawn::spawn_room, spawn::spawn_enemies).chain(),
            );
    }
}

#[derive(Debug, Resource)]
pub struct CurrentRoom {
    info: RoomInfo,
    melee_enemy_stats: crate::enemy::EnemyStats,
    ranged_enemy_stats: crate::enemy::EnemyStats,
    assets: RoomAssets,
}

#[derive(Debug, Default, Resource)]
pub struct Rooms {
    pub map: HashMap<String, (RoomInfo, RoomAssets)>,
}

#[derive(Debug, Clone, AssetCollection, Asset, Reflect, Resource)]
pub struct RoomAssets {
    #[asset(key = "background_texture")]
    pub background_texture: Handle<Image>,
    #[asset(key = "obstacle_texture")]
    pub obstacle_texture: Handle<Image>,

    #[asset(key = "melee_enemy_texture")]
    pub melee_enemy_texture: Handle<Image>,
    #[asset(key = "melee_enemy_stats")]
    pub melee_enemy_stats: Handle<crate::enemy::EnemyStats>,

    #[asset(key = "ranged_enemy_texture")]
    pub ranged_enemy_texture: Handle<Image>,
    #[asset(key = "ranged_enemy_stats")]
    pub ranged_enemy_stats: Handle<crate::enemy::EnemyStats>,
}

#[derive(Debug, Clone, Resource, Asset, Reflect, serde::Deserialize)]
pub struct RoomInfo {
    pub name: String,
    pub rect: Rect,
    pub num_enemies: usize,
    pub num_obstacles: usize,

    // links
    pub north: Option<String>,
    pub south: Option<String>,
    pub east: Option<String>,
    pub west: Option<String>,
}

#[derive(Debug, Default, Component)]
struct RoomObject;

#[derive(Debug, Default, Component)]
struct Floor;

#[derive(Debug, Default, Component)]
struct Obstacle;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Debug, Component)]
struct Wall(Direction);

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

#[derive(Debug, Component)]
pub struct SpawnerIndex(pub usize);

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
