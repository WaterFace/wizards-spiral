use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::input::PlayerAction;

#[derive(Debug, Default)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSpawnPosition>()
            .add_event::<PlayerDeathEvent>()
            .add_loading_state(
                LoadingState::new(crate::states::AppState::CoreLoading)
                    .continue_to_state(crate::states::AppState::RoomLoading)
                    .on_failure_continue_to_state(crate::states::AppState::AppClosing)
                    .load_collection::<PlayerAssets>()
                    .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                        "sprites/player/player.assets.ron",
                    ),
            )
            .add_systems(
                Update,
                (move_player, handle_player_death, manually_restart)
                    .run_if(in_state(crate::states::GameState::InGame)),
            );
    }
}

#[derive(Debug, Default, Component)]
pub struct Player;

#[derive(Debug, Resource)]
pub struct PlayerHealth {
    pub current: f32,
    pub maximum: f32,
    pub dead: bool,
}

impl PlayerHealth {
    pub fn new(maximum: f32) -> Self {
        PlayerHealth {
            current: maximum,
            maximum,
            dead: maximum <= 0.0,
        }
    }
}

impl Default for PlayerHealth {
    fn default() -> Self {
        PlayerHealth::new(100.0)
    }
}

#[derive(Debug, Resource, AssetCollection)]
pub struct PlayerAssets {
    #[asset(key = "sword_shield_texture")]
    pub sword_shield_texture: Handle<Image>,
}

#[derive(Debug, Default, Resource)]
pub struct PlayerSpawnPosition {
    pub pos: Vec2,
}

#[derive(Debug, Default, Clone, Event)]
pub struct PlayerDeathEvent;

#[derive(Debug, Resource)]
struct PlayerDeathTimer(Timer);

impl Default for PlayerDeathTimer {
    fn default() -> Self {
        PlayerDeathTimer(Timer::from_seconds(2.0, TimerMode::Once))
    }
}

fn manually_restart(
    mut player_health: ResMut<PlayerHealth>,
    player_action: Res<ActionState<PlayerAction>>,
    time: Res<Time>,
    mut time_held: Local<f32>,
) {
    if player_action.pressed(&PlayerAction::ManuallyRestart) {
        *time_held += time.delta_seconds();
    } else {
        *time_held = 0.0;
    }

    if *time_held > 2.0 {
        player_health.current = 0.0;
        player_health.dead = true;
    }
}

fn handle_player_death(
    mut commands: Commands,
    player_health: Res<PlayerHealth>,
    player_death_timer: Option<ResMut<PlayerDeathTimer>>,
    mut player_query: Query<(Entity, &mut Transform), With<Player>>,
    mut cycle_counter: ResMut<crate::cycles::CycleCounter>,
    mut next_state: ResMut<NextState<crate::states::GameState>>,
    time: Res<Time>,
) {
    if !player_health.dead {
        return;
    }

    if let Some(mut player_death_timer) = player_death_timer {
        player_death_timer.0.tick(time.delta());
        if player_death_timer.0.finished() {
            commands.remove_resource::<PlayerDeathTimer>();

            cycle_counter.count += 1;
            next_state.set(crate::states::GameState::RestartCycle);
        }
    } else {
        // player just died, do stuff here
        commands.init_resource::<PlayerDeathTimer>();

        let Ok((player_entity, mut transform)) = player_query.get_single_mut() else {
            panic!("handle_player_death")
        };
        commands
            .entity(player_entity)
            .remove::<Collider>()
            .remove::<RigidBody>();
        transform.rotation = Quat::from_rotation_z(PI / 2.0);
    }
}

fn move_player(
    mut query: Query<&mut crate::character_controller::CharacterController, With<Player>>,
    player_health: Res<PlayerHealth>,
    player_action: Res<ActionState<PlayerAction>>,
) {
    if player_health.dead {
        // don't accept input while the player's dead
        return;
    }

    for mut controller in query.iter_mut() {
        let axis_pair = player_action
            .clamped_axis_pair(&PlayerAction::Move)
            .unwrap_or_default();
        controller.desired_direction = axis_pair.into();
    }
}

pub fn spawn_player(
    mut commands: Commands,
    player_spawn_pos: Option<Res<PlayerSpawnPosition>>,
    player_skills: Res<crate::skills::PlayerSkills>,
    player_assets: Res<PlayerAssets>,
) {
    let spawn_position = player_spawn_pos.map(|a| a.pos).unwrap_or(Vec2::ZERO);

    commands
        .spawn(SpriteBundle {
            texture: player_assets.sword_shield_texture.clone(),
            ..Default::default()
        })
        .insert((
            crate::character_controller::CharacterController {
                max_speed: player_skills.get_total_speed(),
                ..Default::default()
            },
            Player,
            RigidBody::Dynamic,
            Collider::ball(16.0),
            ColliderMassProperties::Density(0.0),
            AdditionalMassProperties::MassProperties(MassProperties {
                mass: 1.0,
                ..Default::default()
            }),
            Velocity::default(),
            ExternalImpulse::default(),
            TransformBundle::from_transform(Transform::from_translation(
                spawn_position.extend(0.0),
            )),
            ActiveEvents::COLLISION_EVENTS,
            CollisionGroups::new(
                crate::physics::COLLISION_GROUP_PLAYER,
                crate::physics::COLLISION_GROUP_ENEMY
                    | crate::physics::COLLISION_GROUP_OBSTACLE
                    | crate::physics::COLLISION_GROUP_PLAYER
                    | crate::physics::COLLISION_GROUP_PROJECTILE,
            ),
            Name::new("Player"),
        ));
}

pub fn destroy_player(mut commands: Commands, player_query: Query<Entity, With<Player>>) {
    info!("destroy_player: despawning player");
    for e in player_query.iter() {
        commands.entity(e).despawn_recursive()
    }
}
