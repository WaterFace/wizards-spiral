use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Debug, Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .init_resource::<ActionState<PlayerAction>>()
            .insert_resource(PlayerAction::mkb_input_map());
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    Move,
    ManuallyRestart,
}

impl PlayerAction {
    fn mkb_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::new([
            (PlayerAction::Move, VirtualDPad::wasd()),
            (PlayerAction::Move, VirtualDPad::arrow_keys()),
        ]);
        input_map.insert_multiple([(PlayerAction::ManuallyRestart, KeyCode::KeyK)]);

        input_map
    }
}
