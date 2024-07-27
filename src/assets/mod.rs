use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
};

use bevy_asset_loader::prelude::*;

use bevy_common_assets::ron::RonAssetPlugin;

use thiserror::Error;

mod load_all_room_assets;

#[derive(Debug, Default)]
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RonAssetPlugin::<crate::enemy::EnemyStats>::new(&["enemy.ron"]),
            RonAssetPlugin::<crate::enemy::BossStats>::new(&["boss.ron"]),
            RonAssetPlugin::<crate::room::RoomInfo>::new(&["info.ron"]),
            load_all_room_assets::LoadAllRoomAssetsPlugin,
        ))
        .init_resource::<crate::room::Rooms>();
    }
}

#[derive(Debug, Clone, Resource, Reflect, AssetCollection)]
pub struct RoomInfoAsset {
    #[asset(key = "info")]
    pub info: Handle<crate::room::RoomInfo>,
}

#[derive(Default)]
struct EnemyStatsLoader;

/// Possible errors that can be produced by [`EnemyStatsLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
enum EnemyStatsLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

impl AssetLoader for EnemyStatsLoader {
    type Asset = crate::enemy::EnemyStats;
    type Settings = ();
    type Error = EnemyStatsLoaderError;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let enemy_stats = ron::de::from_bytes::<crate::enemy::EnemyStats>(&bytes)?;
        Ok(enemy_stats)
    }

    fn extensions(&self) -> &[&str] {
        &["enemy.ron"]
    }
}
