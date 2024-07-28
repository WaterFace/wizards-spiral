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
    MainMenu,
    InGame,
    RoomTransition,
    RestartCycle,
    Intro,
    Outro,
}

#[derive(Debug, Default, States, Hash, Eq, PartialEq, Clone, Copy)]
pub enum MenuState {
    #[default]
    None,
    SkillsMenu,
}

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .init_state::<GameState>()
            .init_state::<MenuState>()
            .add_systems(OnEnter(AppState::AppRunning), app_running);
    }
}

fn app_running(mut next_game_state: ResMut<NextState<GameState>>) {
    info!("Starting game");
    next_game_state.set(GameState::InGame)
}
