use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(Debug, Default)]
pub struct TextPlugin;

impl Plugin for TextPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnTransition {
                exited: crate::states::GameState::RoomTransition,
                entered: crate::states::GameState::InGame,
            },
            room_text,
        )
        .add_systems(Update, floating_text)
        .add_systems(
            Update,
            handle_text_markers.run_if(not(in_state(crate::states::AppState::CoreLoading))),
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

fn room_text(
    mut commands: Commands,
    current_room: Res<crate::room::CurrentRoom>,
    camera_query: Query<Entity, With<crate::camera::GameCamera>>,
) {
    let Ok(camera_entity) = camera_query.get_single() else {
        warn!("room_text: can't find game camera");
        return;
    };

    let room_name = &current_room.info.name;
    let text = commands
        .spawn((
            SpatialBundle {
                ..Default::default()
            },
            TextMarker {
                fancy: true,
                font_size: 48.0,
                text: room_name.clone(),
                ..Default::default()
            },
            FloatingText {
                timer: Timer::from_seconds(3.5, TimerMode::Once),
                velocity: Vec2::ZERO,
            },
        ))
        .id();
    commands.entity(camera_entity).add_child(text);
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

fn handle_text_markers(
    mut commands: Commands,
    query: Query<(Entity, &TextMarker), Added<TextMarker>>,
    fonts: Res<Fonts>,
) {
    const BASE_Z: f32 = 100.0;

    // workaround for bevy's blurry text rendering.
    // use a bigger font size and scale the transform down by the same amount
    const SCALE: f32 = 4.0;
    for (e, marker) in query.iter() {
        let font = if marker.fancy {
            &fonts.fancy
        } else {
            &fonts.normal
        };

        commands.entity(e).with_children(|parent| {
            parent.spawn(Text2dBundle {
                transform: Transform::from_xyz(0.0, 0.0, BASE_Z)
                    .with_scale(Vec3::splat(1.0 / SCALE)),
                text: Text::from_section(
                    marker.text.clone(),
                    TextStyle {
                        color: marker
                            .color
                            .unwrap_or(bevy::color::palettes::basic::WHITE.into()),
                        font: font.clone(),
                        font_size: marker.font_size * SCALE,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            });
            // drop shadow
            const OFFSET: f32 = 1.0;
            parent.spawn(Text2dBundle {
                transform: Transform::from_xyz(OFFSET, -OFFSET, BASE_Z - 1.0)
                    .with_scale(Vec3::splat(1.0 / SCALE)),
                text: Text::from_section(
                    marker.text.clone(),
                    TextStyle {
                        color: bevy::color::palettes::basic::BLACK.into(),
                        font: font.clone(),
                        font_size: marker.font_size * SCALE,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            });
        });
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
}

#[derive(Debug, Default, Component)]
pub struct FloatingText {
    pub timer: Timer,
    pub velocity: Vec2,
}
