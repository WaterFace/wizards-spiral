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
        app.add_systems(
            Update,
            move_player.run_if(in_state(crate::states::GameState::InGame)),
        );
    }
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
    In(spawn_position): In<Vec2>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // TEMPORARY!!
    let player_id = commands
        .spawn(SpriteBundle {
            texture: asset_server.load("sprites/Hero.png"),
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
        ))
        .id();

    info!("Player entity: {player_id:?}");
}
