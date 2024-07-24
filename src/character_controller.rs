use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Component, Clone)]
pub struct CharacterController {
    pub desired_direction: Vec2,
    pub max_speed: f32,
    pub acceleration: f32,
}

impl Default for CharacterController {
    fn default() -> Self {
        CharacterController {
            desired_direction: Vec2::ZERO,
            acceleration: 15.0,
            max_speed: 128.0,
        }
    }
}

fn accelerate_character_controllers(
    mut query: Query<(&mut Velocity, &CharacterController)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (mut velocity, controller) in query.iter_mut() {
        // Allow less-than-full-speed movement, but still normalize if necessary so things don't move
        // faster diagonally
        let desired_movement = if controller.desired_direction.length_squared() > 1.0 {
            controller.desired_direction.normalize()
        } else {
            controller.desired_direction
        };

        let vel = velocity.linvel;

        let desired_velocity = desired_movement * controller.max_speed;
        let diff = desired_velocity - vel;

        velocity.linvel += diff * controller.acceleration * dt;
    }
}

#[derive(Debug, Default)]
pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            accelerate_character_controllers.run_if(in_state(crate::states::GameState::InGame)),
        );
    }
}
