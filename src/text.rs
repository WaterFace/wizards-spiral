use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_math::{bounding::BoundingVolume, vec2};

#[derive(Debug, Default)]
pub struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnTransition {
                exited: crate::states::GameState::RoomTransition,
                entered: crate::states::GameState::InGame,
            },
            (room_text, spawn_next_room_text),
        )
        .add_systems(Update, floating_text)
        .add_systems(
            Update,
            update_next_room_text.run_if(in_state(crate::states::GameState::InGame)),
        )
        .add_systems(
            Update,
            (handle_text_markers, update_aabb_fields)
                .run_if(not(in_state(crate::states::AppState::CoreLoading))),
        )
        .add_loading_state(
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

fn spawn_next_room_text(mut commands: Commands, current_room: Res<crate::room::CurrentRoom>) {
    let directions = [
        crate::room::CardinalDirection::North,
        crate::room::CardinalDirection::South,
        crate::room::CardinalDirection::East,
        crate::room::CardinalDirection::West,
    ];

    let room_info = &current_room.info;
    for dir in directions {
        let Some(next_room_name) = (match dir {
            crate::room::CardinalDirection::North => &room_info.north,
            crate::room::CardinalDirection::South => &room_info.south,
            crate::room::CardinalDirection::East => &room_info.east,
            crate::room::CardinalDirection::West => &room_info.west,
        }) else {
            // There's no room in that direction, no need for a label
            continue;
        };

        commands.spawn((
            SpatialBundle {
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            NextRoomText { side: dir },
            TextMarker {
                font_size: 18.0,
                text: format!("To {}", next_room_name),
                ..Default::default()
            },
            crate::room::RoomObject,
        ));
    }
}

fn update_next_room_text(
    mut query: Query<(&mut Transform, &TextMarker, &mut Visibility, &NextRoomText)>,
    camera_query: Query<&GlobalTransform, With<crate::camera::GameCamera>>,
    current_room: Res<crate::room::CurrentRoom>,
) {
    let Ok(camera_global_transform) = camera_query.get_single() else {
        warn!("update_next_room_text: Couldn't find GameCamera");
        return;
    };
    let camera_pos = camera_global_transform.translation().truncate();
    let room_rect = current_room.info.rect;

    for (mut transform, text_marker, mut visibility, next_room_text) in query.iter_mut() {
        let Some(aabb2d) = text_marker.text_aabb else {
            continue;
        };

        // the aabb is set at this point, so we can make it visible and position it correctly
        *visibility = Visibility::Inherited;

        const PADDING: f32 = 16.0;

        let pos = match next_room_text.side {
            crate::room::CardinalDirection::North => vec2(
                camera_pos.x,
                room_rect.half_size().y - aabb2d.half_size().y - PADDING,
            ),
            crate::room::CardinalDirection::South => vec2(
                camera_pos.x,
                -room_rect.half_size().y + aabb2d.half_size().y + PADDING,
            ),
            crate::room::CardinalDirection::East => vec2(
                room_rect.half_size().x - aabb2d.half_size().x - PADDING,
                camera_pos.y,
            ),
            crate::room::CardinalDirection::West => vec2(
                -room_rect.half_size().x + aabb2d.half_size().x + PADDING,
                camera_pos.y,
            ),
        };

        transform.translation = pos.extend(transform.translation.z);
    }
}

fn room_text(
    mut commands: Commands,
    current_room: Res<crate::room::CurrentRoom>,
    camera_query: Query<(Entity, &OrthographicProjection), With<crate::camera::GameCamera>>,
    cycle_counter: Res<crate::cycles::CycleCounter>,
    mut texts_to_spawn: Local<Vec<String>>,
) {
    let Ok((camera_entity, ortho_proj)) = camera_query.get_single() else {
        warn!("room_text: can't find game camera");
        return;
    };

    texts_to_spawn.push(current_room.info.name.clone());
    if current_room.info.name == "Lovely Cottage" {
        texts_to_spawn.push(format!("Cycle {}", cycle_counter.count + 1));
    }

    let n = texts_to_spawn.len();
    let camera_rect = ortho_proj.area;
    for (i, text) in texts_to_spawn.drain(..).enumerate() {
        let t = (i + 1) as f32 / (n + 1) as f32;
        let y = camera_rect.max.y * (1.0 - t) + camera_rect.min.y * t;
        let text_entity = commands
            .spawn((
                SpatialBundle {
                    transform: Transform::from_xyz(0.0, y, 0.0),
                    ..Default::default()
                },
                TextMarker {
                    fancy: i == 0,
                    font_size: 48.0,
                    text,
                    ..Default::default()
                },
                FloatingText {
                    timer: Timer::from_seconds(3.5, TimerMode::Once),
                    velocity: Vec2::ZERO,
                },
            ))
            .id();
        commands.entity(camera_entity).add_child(text_entity);
    }
}

fn floating_text(
    mut commands: Commands,
    mut query: Query<(Entity, &mut FloatingText, &mut Transform)>,
    time: Res<Time>,
) {
    for (e, mut floating_text, mut transform) in query.iter_mut() {
        floating_text.timer.tick(time.delta());
        transform.translation += (floating_text.velocity * time.delta_seconds()).extend(0.0);

        if floating_text.timer.finished() {
            commands.entity(e).despawn_recursive();
        }
    }
}

// workaround for bevy's blurry text rendering.
// use a bigger font size and scale the transform down by the same amount
const FONT_SIZE_SCALE: f32 = 4.0;

fn handle_text_markers(
    mut commands: Commands,
    query: Query<(Entity, &TextMarker), Added<TextMarker>>,
    fonts: Res<Fonts>,
) {
    const BASE_Z: f32 = 100.0;

    for (e, marker) in query.iter() {
        let font = if marker.fancy {
            &fonts.fancy
        } else {
            &fonts.normal
        };

        commands.entity(e).with_children(|parent| {
            parent.spawn((
                Text2dBundle {
                    transform: Transform::from_xyz(0.0, 0.0, BASE_Z)
                        .with_scale(Vec3::splat(1.0 / FONT_SIZE_SCALE)),
                    text: Text::from_section(
                        marker.text.clone(),
                        TextStyle {
                            color: marker
                                .color
                                .unwrap_or(bevy::color::palettes::basic::WHITE.into()),
                            font: font.clone(),
                            font_size: marker.font_size * FONT_SIZE_SCALE,
                            ..Default::default()
                        },
                    ),
                    ..Default::default()
                },
                TextAabbMarker,
            ));
            // drop shadow
            const OFFSET: f32 = 1.0;
            parent.spawn(Text2dBundle {
                transform: Transform::from_xyz(OFFSET, -OFFSET, BASE_Z - 1.0)
                    .with_scale(Vec3::splat(1.0 / FONT_SIZE_SCALE)),
                text: Text::from_section(
                    marker.text.clone(),
                    TextStyle {
                        color: bevy::color::palettes::basic::BLACK.into(),
                        font: font.clone(),
                        font_size: marker.font_size * FONT_SIZE_SCALE,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            });
        });
    }
}

fn update_aabb_fields(
    text_query: Query<
        (Entity, &Parent, &bevy::render::primitives::Aabb),
        (
            With<TextAabbMarker>,
            Changed<bevy::render::primitives::Aabb>,
        ),
    >,
    mut marker_query: Query<&mut TextMarker>,
) {
    for (e, parent, aabb) in text_query.iter() {
        let parent = parent.get();
        let Ok(mut text_marker) = marker_query.get_mut(parent) else {
            error!("Entity {e:?} with TextAabbMarker doesn't have a parent!");
            continue;
        };

        let origin: Vec3 = aabb.center.into();
        let half_extents: Vec3 = aabb.half_extents.into();

        let aabb2d = bevy_math::bounding::Aabb2d::new(
            origin.truncate(),
            half_extents.truncate() / FONT_SIZE_SCALE,
        );

        text_marker.text_aabb = Some(aabb2d);
    }
}

#[derive(Resource, AssetCollection, Debug)]
pub struct Fonts {
    #[asset(key = "normal_font")]
    pub normal: Handle<Font>,
    #[asset(key = "fancy_font")]
    pub fancy: Handle<Font>,
}

#[derive(Debug, Default, Component)]
pub struct TextMarker {
    pub text: String,
    pub fancy: bool,
    pub font_size: f32,
    pub color: Option<Color>,

    /// This will be filled out after the text is spawned. It shouldn't be changed manually
    pub text_aabb: Option<bevy_math::bounding::Aabb2d>,
}

#[derive(Debug, Default, Component)]
struct TextAabbMarker;

#[derive(Debug, Default, Component)]
pub struct FloatingText {
    pub timer: Timer,
    pub velocity: Vec2,
}

#[derive(Debug, Component)]
pub struct NextRoomText {
    pub side: crate::room::CardinalDirection,
}
