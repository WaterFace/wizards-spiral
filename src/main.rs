use bevy::prelude::*;

mod assets;
mod camera;
mod character_controller;
mod damage;
mod enemy;
mod input;
mod menus;
mod physics;
mod player;
mod rand;
mod room;
mod states;
mod text;

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
        .add_plugins(text::TextPlugin)
        .add_plugins(menus::MenusPlugin)
        .add_plugins(camera::CameraPlugin)
        .run();
}
