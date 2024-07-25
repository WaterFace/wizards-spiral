use bevy::prelude::*;

#[derive(Debug, Default, States, Hash, Eq, PartialEq, Clone, Copy)]
pub enum AppState {
    #[default]
    CoreLoading,
    RoomLoading,
    InMenu,
    AppRunning,
    AppClosing,
}

#[derive(Debug, Default, States, Hash, Eq, PartialEq, Clone, Copy)]
pub enum GameState {
    #[default]
    None,
    Loading,
    MainMenu,
    InGame,
    RoomTransition,
}

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_state::<GameState>()
            .add_systems(OnEnter(AppState::AppRunning), app_running);
    }
}

fn app_running(mut next_game_state: ResMut<NextState<GameState>>) {
    info!("Starting game");
    next_game_state.set(GameState::InGame)
}
