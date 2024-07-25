use bevy::prelude::*;

mod assets;
mod character_controller;
mod damage;
mod enemy;
mod input;
mod physics;
mod player;
mod rand;
mod room;
mod states;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(assets::AssetsPlugin)
        .add_plugins(rand::RandPlugin)
        .add_plugins(physics::PhysicsPlugin)
        .add_plugins(states::StatesPlugin)
        .add_plugins(character_controller::CharacterControllerPlugin)
        .add_plugins(input::InputPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(enemy::EnemyPlugin)
        .add_plugins(damage::DamagePlugin)
        .add_plugins(room::RoomPlugin)
        .add_systems(
            OnEnter(states::GameState::InGame),
            (setup, (|| Vec2::ZERO).pipe(player::spawn_player)),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1.0,
            near: -1000.0,
            far: 1000.0,
            ..Default::default()
        },
        ..Default::default()
    });
}
