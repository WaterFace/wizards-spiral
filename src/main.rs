use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod character_controller;
mod input;
mod physics;
mod player;
mod states;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(physics::PhysicsPlugin)
        .add_plugins(states::StatesPlugin)
        .add_plugins(character_controller::CharacterControllerPlugin)
        .add_plugins(input::InputPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_systems(OnEnter(states::GameState::InGame), setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        ..Default::default()
    });
    commands.spawn((
        character_controller::CharacterController::default(),
        player::Player,
        RigidBody::Dynamic,
        Collider::ball(16.0),
        ColliderMassProperties::Density(0.0),
        AdditionalMassProperties::MassProperties(MassProperties {
            mass: 1.0,
            ..Default::default()
        }),
        ExternalImpulse::default(),
        Velocity::default(),
        TransformBundle::default(),
    ));
}
