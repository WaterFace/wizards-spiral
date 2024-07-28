use bevy::prelude::*;

mod assets;
mod audio;
mod camera;
mod character_controller;
mod cycles;
mod damage;
mod enemy;
mod healthbars;
mod input;
mod menus;
mod physics;
mod player;
mod projectiles;
mod rand;
mod room;
mod save_data;
mod skills;
mod states;
mod text;
mod util;

fn main() {
    let mut app = App::new();

    #[cfg(target_arch = "wasm32")]
    app.add_plugins(bevy_embedded_assets::EmbeddedAssetPlugin {
        mode: bevy_embedded_assets::PluginMode::ReplaceDefault,
    });

    app.add_plugins(DefaultPlugins.set(bevy::audio::AudioPlugin {
        default_spatial_scale: bevy::audio::SpatialScale::new_2d(1.0 / 128.0), // vec3(-1.0, 1.0, 0.0) * (1.0 / 128.0)
        ..Default::default()
    }))
    .add_plugins(save_data::SaveDataPlugin)
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
    .add_plugins(skills::SkillsPlugin)
    .add_plugins(cycles::CyclePlugin)
    .add_plugins(healthbars::HealthbarsPlugin)
    .add_plugins(projectiles::ProjectilesPlugin)
    .add_plugins(audio::AudioPlugin)
    .run();
}
