use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use serde::Deserialize;

#[derive(Debug, Default)]
pub struct LoadAllRoomAssetsPlugin;

// I know this is super cursed but bear with me

impl Plugin for LoadAllRoomAssetsPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_arch = "wasm32"))]
        let str = {
            use std::io::Read;
            let mut file = std::fs::File::open("assets/rooms/room_list.ron")
                .inspect_err(|e| {
                    panic!("Failed to open file 'assets/rooms/room_list.ron': {e}");
                })
                .expect("If it couldn't be opened we would have panicked already");
            let mut str = String::new();
            let _ = file.read_to_string(&mut str);
            str
        };

        #[cfg(target_arch = "wasm32")]
        let str = include_str!("../../assets/rooms/room_list.ron");

        let rooms_to_load: RoomsToLoad = ron::from_str(&str)
            .inspect_err(|e| {
                panic!("Failed to parse file 'assets/rooms/room_list.ron': {e}");
            })
            .expect("If it couldn't be parsed we would have panicked already");

        app.insert_resource(rooms_to_load.clone());
        app.init_state::<State>();
        for room_file in rooms_to_load.rooms.iter() {
            app.add_loading_state(
                LoadingState::new(State::Loading(room_file.to_string()))
                    .continue_to_state(State::Waiting)
                    .load_collection::<super::RoomInfoAsset>()
                    .load_collection::<crate::room::RoomAssets>()
                    .with_dynamic_assets_file::<StandardDynamicAssetCollection>(room_file),
            );
        }
        app.add_systems(OnEnter(State::Waiting), queue_next_room);
        app.add_systems(OnEnter(State::Failed), on_fail);
        app.add_systems(OnEnter(State::AllFinished), on_finish);
        app.add_systems(
            OnEnter(crate::states::AppState::RoomLoading),
            start_loading_rooms,
        );
    }
}

#[derive(Debug, Default, States, Clone, Hash, Eq, PartialEq)]
enum State {
    #[default]
    None,
    Waiting,
    Loading(String),
    AllFinished,
    Failed,
}

#[derive(Debug, Clone, Default, Resource, Deserialize)]
struct RoomsToLoad {
    rooms: Vec<String>,
}

fn start_loading_rooms(mut next_state: ResMut<NextState<State>>) {
    next_state.set(State::Waiting);
}

fn on_finish(
    mut next_app_state: ResMut<NextState<crate::states::AppState>>,
    mut next_game_state: ResMut<NextState<crate::states::GameState>>,
) {
    next_app_state.set(crate::states::AppState::InMenu);
    next_game_state.set(crate::states::GameState::MainMenu);
}

fn on_fail(mut exit: EventWriter<AppExit>) {
    exit.send(AppExit::Error(
        std::num::NonZero::<u8>::new(1).expect("1 is nonzero"),
    ));
}

fn queue_next_room(
    mut commands: Commands,
    mut rooms: ResMut<crate::room::Rooms>,
    mut rooms_to_load: ResMut<RoomsToLoad>,
    room_assets: Option<Res<crate::room::RoomAssets>>,
    room_info_asset: Option<Res<crate::assets::RoomInfoAsset>>,
    room_infos: Res<Assets<crate::room::RoomInfo>>,
    mut next_state: ResMut<NextState<State>>,
) {
    if let Some(room_assets) = room_assets {
        if let Some(room_info_asset) = room_info_asset {
            let room_info = room_infos.get(&room_info_asset.info).unwrap();

            let room_name = room_info.name.clone();

            info!("{} successfully loaded.", room_name);
            rooms
                .map
                .insert(room_name, (room_info.clone(), room_assets.clone()));

            // The loading system loop will replace these values, but this way it's cleaned up at the end
            // The handles are being kept alive by the Rooms resource
            commands.remove_resource::<crate::room::RoomAssets>();
            commands.remove_resource::<crate::assets::RoomInfoAsset>();
        }
    }

    if let Some(next) = rooms_to_load.rooms.pop() {
        info!("Preparing to load {}...", next);
        next_state.set(State::Loading(next));
    } else {
        info!("All rooms loaded!");
        next_state.set(State::AllFinished);
    }
}
