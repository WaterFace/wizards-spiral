use bevy::prelude::*;
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
