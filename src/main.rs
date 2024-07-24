use bevy::prelude::*;

mod character_controller;
mod damage;
mod enemy;
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
        .add_plugins(enemy::EnemyPlugin)
        .add_plugins(damage::DamagePlugin)
        .add_systems(
            OnEnter(states::GameState::InGame),
            (
                setup,
                (|| Vec2::ZERO).pipe(player::spawn_player),
                (|| {
                    vec![
                        (
                            Vec2::new(100.0, 100.0),
                            enemy::EnemyStats {
                                enemy_type: enemy::EnemyType::Melee,
                                alert_radius: 75.0,
                                chase_radius: 100.0,
                                desired_distance: 0.0,
                            },
                        ),
                        (
                            Vec2::new(-100.0, 100.0),
                            enemy::EnemyStats {
                                enemy_type: enemy::EnemyType::Ranged,
                                alert_radius: 75.0,
                                chase_radius: 100.0,
                                desired_distance: 50.0,
                            },
                        ),
                        (
                            Vec2::new(-100.0, -100.0),
                            enemy::EnemyStats {
                                enemy_type: enemy::EnemyType::Ranged,
                                alert_radius: 75.0,
                                chase_radius: 100.0,
                                desired_distance: f32::INFINITY,
                            },
                        ),
                    ]
                })
                .pipe(enemy::spawn_melee_enemies),
            ),
        )
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.5,
            near: -1000.0,
            far: 1000.0,
            ..Default::default()
        },
        ..Default::default()
    });
}
