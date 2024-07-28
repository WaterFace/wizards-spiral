use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_math::vec3;

#[derive(Debug, Default)]
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            follow_player.run_if(in_state(crate::states::GameState::InGame)),
        );
    }
}

#[derive(Debug, Default, Component)]
pub struct GameCamera;

#[derive(Debug, Default, Component)]
pub struct MenuCamera;

pub fn game_camera() -> impl Bundle {
    (
        Camera2dBundle {
            projection: OrthographicProjection {
                scale: 1.0,
                near: -1000.0,
                far: 1000.0,
                scaling_mode: ScalingMode::FixedVertical(350.0),
                ..Default::default()
            },
            ..Default::default()
        },
        GameCamera,
        // so we can parent visible things to the camera
        VisibilityBundle::default(),
        Name::new("Game Camera"),
    )
}

pub fn menu_camera() -> impl Bundle {
    (
        Camera2dBundle {
            camera: Camera {
                order: 1000,
                ..Default::default()
            },
            projection: OrthographicProjection {
                scale: 1.0,
                near: -1000.0,
                far: 1000.0,
                scaling_mode: ScalingMode::AutoMax {
                    max_height: 720.0,
                    max_width: 1280.0,
                },
                ..Default::default()
            },
            ..Default::default()
        },
        MenuCamera,
        Name::new("Menu Camera"),
    )
}

pub fn spawn_game_camera(mut commands: Commands) {
    commands.spawn(game_camera());
}

pub fn destroy_game_camera(mut commands: Commands, query: Query<Entity, With<GameCamera>>) {
    info!("destroy_game_camera: despawning game cameras");
    for e in query.iter() {
        commands.entity(e).despawn_recursive()
    }
}

fn follow_player(
    player_query: Query<&GlobalTransform, With<crate::player::Player>>,
    mut camera_query: Query<(&mut Transform, &OrthographicProjection), With<GameCamera>>,
    current_room: Option<Res<crate::room::CurrentRoom>>,
) {
    let room_info = current_room.as_ref().map(|r| &r.info);

    let player_transform = player_query.get_single().ok();

    for (mut camera_transform, ortho_projection) in camera_query.iter_mut() {
        match (room_info, player_transform) {
            (_, None) => {
                // for whatever reason, we don't have a player so just stick to (0, 0)
                camera_transform.translation = Vec3::ZERO;
            }
            (None, Some(player_transform)) => {
                // we have a player but not a RoomInfo, so stick to the player without anything fancy
                camera_transform.translation = player_transform.translation();
            }
            (Some(room_info), Some(player_transform)) => {
                // we have both, so stick to the player but try to stay within the bounds of the room (unless the room isn't big enough)
                let room_rect = room_info.rect;
                let view_rect = ortho_projection.area;
                let player_pos = player_transform.translation();

                let min_x = room_rect.min.x + view_rect.half_size().x;
                let max_x = room_rect.max.x - view_rect.half_size().x;

                let min_y = room_rect.min.y + view_rect.half_size().y;
                let max_y = room_rect.max.y - view_rect.half_size().y;

                let mut x = player_pos.x;
                let mut y = player_pos.y;

                if min_x > max_x {
                    x = 0.0;
                } else {
                    x = x.clamp(min_x, max_x);
                }

                if min_y > max_y {
                    y = 0.0;
                } else {
                    y = y.clamp(min_y, max_y);
                }

                camera_transform.translation = vec3(x, y, 0.0);
            }
        }
    }
}
