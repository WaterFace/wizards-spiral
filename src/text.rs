use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(Debug, Default)]
pub struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(crate::states::AppState::CoreLoading)
                .continue_to_state(crate::states::AppState::RoomLoading)
                .on_failure_continue_to_state(crate::states::AppState::AppClosing)
                .load_collection::<Fonts>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "fonts/fonts.assets.ron",
                ),
        );
    }
}

#[derive(Resource, AssetCollection, Debug)]
pub struct Fonts {
    #[asset(key = "normal_font")]
    pub normal: Handle<Font>,
    #[asset(key = "fancy_font")]
    pub fancy: Handle<Font>,
}

// TODO: floating text
