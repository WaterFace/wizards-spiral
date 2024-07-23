use bevy::prelude::*;

#[derive(Debug, Default, States, Hash, Eq, PartialEq, Clone, Copy)]
#[allow(unused)]
pub enum AppState {
    #[default]
    CoreLoading,
    InMenu,
    AppRunning,
    AppClosing,
}

#[derive(Debug, Default, States, Hash, Eq, PartialEq, Clone, Copy)]
#[allow(unused)]
pub enum GameState {
    None,
    MainMenu,
    #[default]
    InGame,
}

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>().init_state::<GameState>();
    }
}
