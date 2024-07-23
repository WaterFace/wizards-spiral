use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Component, Clone)]
pub struct CharacterController {
    pub desired_direction: Vec2,
    pub max_speed: f32,
    pub acceleration: f32,
    pub drag: f32,
}

impl Default for CharacterController {
    fn default() -> Self {
        CharacterController {
            desired_direction: Vec2::ZERO,
            acceleration: 15.0,
            max_speed: 3.0,
            drag: 1.0,
        }
    }
}

fn accelerate_character_controllers(
    mut query: Query<(&mut ExternalImpulse, &CharacterController)>,
) {
    for (mut impulse, character_controller) in query.iter_mut() {
        impulse.impulse +=
            character_controller.desired_direction * character_controller.acceleration;
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
