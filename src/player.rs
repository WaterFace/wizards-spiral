use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::input::PlayerAction;

#[derive(Debug, Default, Component)]
pub struct Player;

#[derive(Debug, Default)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSpawnPosition>()
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
                move_player.run_if(in_state(crate::states::GameState::InGame)),
            );
    }
}

#[derive(Debug, Resource, AssetCollection)]
pub struct PlayerAssets {
    #[asset(key = "sword_shield_texture")]
    pub sword_shield_texture: Handle<Image>,
}

fn move_player(
    mut query: Query<&mut crate::character_controller::CharacterController, With<Player>>,
    player_action: Res<ActionState<PlayerAction>>,
) {
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
    player_assets: Res<PlayerAssets>,
) {
    let spawn_position = player_spawn_pos.map(|a| a.pos).unwrap_or(Vec2::ZERO);

    commands
        .spawn(SpriteBundle {
            texture: player_assets.sword_shield_texture.clone(),
            ..Default::default()
        })
        .insert((
            crate::character_controller::CharacterController::default(),
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
        ));
}

    info!("Player entity: {player_id:?}");
}
