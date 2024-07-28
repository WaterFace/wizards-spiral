use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_math::vec3;

#[derive(Debug, Default)]
pub struct HealthbarsPlugin;

impl Plugin for HealthbarsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(crate::states::AppState::CoreLoading)
                .continue_to_state(crate::states::AppState::RoomLoading)
                .on_failure_continue_to_state(crate::states::AppState::AppClosing)
                .load_collection::<HealthbarAssets>()
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                    "sprites/healthbars/healthbars.assets.ron",
                ),
        )
        .add_systems(
            Update,
            (spawn_healthbars, update_healthbars)
                .run_if(in_state(crate::states::GameState::InGame)),
        );
    }
}

#[derive(Debug, AssetCollection, Resource)]
pub struct HealthbarAssets {
    #[asset(key = "healthbar_empty")]
    pub healthbar_empty: Handle<Image>,

    #[asset(key = "healthbar_full")]
    pub healthbar_full: Handle<Image>,
}

#[derive(Debug, Default, Component)]
pub struct HealthbarRoot;

#[derive(Debug, Clone, Component)]
pub enum Healthbar {
    Enemy { entity: Entity },
    Player,
}

#[derive(Debug, Component)]
struct HealthbarSpriteRect {
    rect: Rect,
}

fn update_healthbars(
    enemy_query: Query<&crate::enemy::EnemyHealth>,
    mut healthbar_query: Query<(
        &Healthbar,
        Option<&mut Visibility>,
        Option<&mut Sprite>,
        &HealthbarSpriteRect,
    )>,
    player_health: Res<crate::player::PlayerHealth>,
) {
    for (healthbar, visibility, sprite, sprite_rect) in healthbar_query.iter_mut() {
        let health_fraction = match healthbar {
            Healthbar::Player => player_health.current / player_health.maximum,
            Healthbar::Enemy { entity } => {
                let Ok(enemy_health) = enemy_query.get(*entity) else {
                    continue;
                };
                enemy_health.current / enemy_health.maximum
            }
        };

        if let Some(mut visibility) = visibility {
            if health_fraction < 1.0 && health_fraction > 0.0 {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }

        if let Some(mut sprite) = sprite {
            let mut new_rect = sprite_rect.rect;
            new_rect.max.x *= health_fraction;
            sprite.rect = Some(new_rect);
        }
    }
}

fn spawn_healthbars(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            Option<&crate::player::Player>,
            Option<&crate::enemy::Enemy>,
            Option<&crate::enemy::Boss>,
        ),
        Or<(
            Added<crate::enemy::Enemy>,
            Added<crate::player::Player>,
            Added<crate::enemy::Boss>,
        )>,
    >,
    camera_query: Query<(Entity, &OrthographicProjection), With<crate::camera::GameCamera>>,
    current_room: Res<crate::room::CurrentRoom>,
    healthbar_assets: Res<HealthbarAssets>,
    image_assets: Res<Assets<Image>>,
) {
    let healthbar_size = image_assets
        .get(&healthbar_assets.healthbar_full)
        .expect("This asset is definitely loaded at this point")
        .size_f32();
    const OFFSET: Vec3 = vec3(0.0, -18.0, 10.0);
    for (entity, player, enemy, boss) in query.iter() {
        let healthbar_component = match (player, enemy) {
            (None, None) => unreachable!(),
            (Some(_), None) => Healthbar::Player,
            (None, Some(_)) => Healthbar::Enemy { entity },
            (Some(_), Some(_)) => {
                error!(
                    "Entity {entity} has both a player and enemy component. This shouldn't happen"
                );
                continue;
            }
        };
        let root = if boss.is_some() {
            let Ok((camera_entity, ortho_proj)) = camera_query.get_single() else {
                error!("spawn_healthbars: failed to get single game camera");
                continue;
            };
            let camera_rect = ortho_proj.area;
            let boss_name = current_room
                .boss_stats
                .as_ref()
                .expect(
                    "If we're spawning a healthbar for a boss, the current room must have a boss",
                )
                .name
                .clone();
            let boss_healthbar_scale = vec3(7.5, 7.5, 1.0);
            commands
                .spawn((
                    SpatialBundle {
                        transform: Transform::from_translation(vec3(
                            0.0,
                            -camera_rect.half_size().y * 0.75,
                            500.0,
                        ))
                        .with_scale(boss_healthbar_scale),
                        ..Default::default()
                    },
                    HealthbarRoot,
                    healthbar_component.clone(),
                    // We don't really need it in this case but it makes the update_healthbars system simpler
                    HealthbarSpriteRect {
                        rect: Rect::from_corners(Vec2::ZERO, healthbar_size),
                    },
                    Name::new("Boss Healthbar Root"),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        SpatialBundle {
                            transform: Transform::from_scale(1.0 / boss_healthbar_scale),
                            ..Default::default()
                        },
                        crate::text::TextMarker {
                            font_size: 18.0,
                            text: boss_name,
                            ..Default::default()
                        },
                        Name::new("Boss Healthbar Text"),
                    ));
                })
                .set_parent(camera_entity)
                .id()
        } else {
            commands
                .spawn((
                    SpatialBundle {
                        transform: Transform::from_translation(OFFSET),
                        ..Default::default()
                    },
                    HealthbarRoot,
                    healthbar_component.clone(),
                    // We don't really need it in this case but it makes the update_healthbars system simpler
                    HealthbarSpriteRect {
                        rect: Rect::from_corners(Vec2::ZERO, healthbar_size),
                    },
                    Name::new("Enemy Healthbar Root"),
                ))
                .set_parent(entity)
                .id()
        };

        commands.entity(root).with_children(|parent| {
            parent.spawn((
                SpriteBundle {
                    transform: Transform::from_xyz(-healthbar_size.x / 2.0, 0.0, 0.0),
                    texture: healthbar_assets.healthbar_empty.clone(),
                    sprite: Sprite {
                        anchor: bevy::sprite::Anchor::CenterLeft,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Name::new("Healthbar Background"),
            ));

            parent.spawn((
                SpriteBundle {
                    transform: Transform::from_xyz(-healthbar_size.x / 2.0, 0.0, 1.0),
                    texture: healthbar_assets.healthbar_full.clone(),
                    sprite: Sprite {
                        anchor: bevy::sprite::Anchor::CenterLeft,
                        rect: Some(Rect::from_corners(Vec2::ZERO, healthbar_size)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                healthbar_component.clone(),
                HealthbarSpriteRect {
                    rect: Rect::from_corners(Vec2::ZERO, healthbar_size),
                },
                Name::new("Healthbar Foreground"),
            ));
        });
    }
}
