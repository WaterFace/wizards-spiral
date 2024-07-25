use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
};

use bevy_asset_loader::prelude::*;

use bevy_common_assets::ron::RonAssetPlugin;

use serde::Deserialize;

use thiserror::Error;

#[derive(Debug, Default)]
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RonAssetPlugin::<EnemyStats>::new(&["enemy.ron"]),
            RonAssetPlugin::<RoomInfo>::new(&["info.ron"]),
        ))
        .add_loading_state(
            LoadingState::new(crate::states::AppState::CoreLoading)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "rooms/forest.assets.ron",
                )
                .load_collection::<RoomInfoAsset>()
                .load_collection::<RoomAssets>()
                .continue_to_state(crate::states::AppState::AppRunning),
        );
    }
}

#[derive(Debug, AssetCollection, Asset, Reflect, Resource)]
pub struct RoomAssets {
    #[asset(key = "background_texture")]
    pub background_texture: Handle<Image>,
    #[asset(key = "obstacle_texture")]
    pub obstacle_texture: Handle<Image>,

    #[asset(key = "melee_enemy_texture")]
    pub melee_enemy_texture: Handle<Image>,
    #[asset(key = "melee_enemy_stats")]
    pub melee_enemy_stats: Handle<crate::assets::EnemyStats>,

    #[asset(key = "ranged_enemy_texture")]
    pub ranged_enemy_texture: Handle<Image>,
    #[asset(key = "ranged_enemy_stats")]
    pub ranged_enemy_stats: Handle<crate::assets::EnemyStats>,
}

#[derive(Debug, Clone, Resource, Reflect, AssetCollection)]
pub struct RoomInfoAsset {
    #[asset(key = "info")]
    pub info: Handle<RoomInfo>,
}

#[derive(Debug, Clone, Resource, Asset, Reflect, Deserialize)]
pub struct RoomInfo {
    pub name: String,
    pub rect: Rect,
    pub num_enemies: usize,
    pub num_obstacles: usize,

    // links
    pub north: Option<String>,
    pub south: Option<String>,
    pub east: Option<String>,
    pub west: Option<String>,
}

#[derive(Debug, Component, Asset, Reflect, Deserialize)]
pub struct EnemyStats {
    /// Type of enemy; either Melee or Ranged
    pub enemy_type: EnemyType,
    /// Radius at which the enemy is alerted to the player. If the player gets this close, the enemy begins to chase
    pub alert_radius: f32,
    /// Radius at which the enemy will stop chasing the player. If the player is this far away, the enemy will stop chasing
    pub chase_radius: f32,
    /// How far the enemy will try to stay from the player.
    pub desired_distance: f32,
}

#[derive(Debug, Component, Clone, Copy, Reflect, Deserialize)]
pub enum EnemyType {
    Melee,
    Ranged,
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
    type Asset = EnemyStats;
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
        let enemy_stats = ron::de::from_bytes::<EnemyStats>(&bytes)?;
        Ok(enemy_stats)
    }

    fn extensions(&self) -> &[&str] {
        &["enemy.ron"]
    }
}
