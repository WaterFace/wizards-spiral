use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Debug, Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .init_resource::<ActionState<PlayerAction>>()
            .insert_resource(PlayerAction::mkb_input_map())
            .add_plugins(InputManagerPlugin::<MenuAction>::default())
            .init_resource::<ActionState<MenuAction>>()
            .insert_resource(MenuAction::menu_input_map());
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

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum MenuAction {
    SkillsMenu,
}

impl MenuAction {
    fn menu_input_map() -> InputMap<Self> {
        InputMap::new([
            (MenuAction::SkillsMenu, KeyCode::Space),
            (MenuAction::SkillsMenu, KeyCode::Tab),
            (MenuAction::SkillsMenu, KeyCode::KeyI),
        ])
    }
}
