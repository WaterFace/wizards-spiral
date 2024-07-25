use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(Debug, Default)]
pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.enable_state_scoped_entities::<crate::states::GameState>()
            .enable_state_scoped_entities::<crate::states::AppState>()
            .add_loading_state(
                LoadingState::new(crate::states::AppState::CoreLoading)
                    .continue_to_state(crate::states::AppState::RoomLoading)
                    .on_failure_continue_to_state(crate::states::AppState::AppClosing)
                    .load_collection::<UiAssets>()
                    .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                        "sprites/ui/ui.assets.ron",
                    ),
            )
            .add_systems(
                OnEnter(crate::states::AppState::RoomLoading),
                (|| crate::states::AppState::RoomLoading).pipe(loading_screen),
            )
            .add_systems(
                OnEnter(crate::states::GameState::RoomTransition),
                (|| crate::states::GameState::RoomTransition).pipe(loading_screen),
            )
            .add_systems(OnEnter(crate::states::GameState::MainMenu), main_menu);
    }
}

#[derive(Debug, Clone, AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(key = "panel")]
    pub panel: Handle<Image>,
}

fn main_menu(mut commands: Commands, fonts: Res<crate::text::Fonts>, ui_assets: Res<UiAssets>) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1000,
                ..Default::default()
            },
            ..Default::default()
        },
        StateScoped(crate::states::GameState::MainMenu),
    ));

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
            StateScoped(crate::states::GameState::MainMenu),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        image: UiImage {
                            texture: ui_assets.panel.clone().into(),
                            color: bevy::color::palettes::basic::BLUE.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ImageScaleMode::Sliced(TextureSlicer {
                        border: BorderRect::square(16.0),
                        ..Default::default()
                    }),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Play",
                            TextStyle {
                                font: fonts.normal.clone(),
                                font_size: 72.0,
                                ..Default::default()
                            },
                        ),
                        ..Default::default()
                    });
                });
        });
}

fn loading_screen<S: States>(
    In(state): In<S>,
    mut commands: Commands,
    fonts: Res<crate::text::Fonts>,
    ui_assets: Res<UiAssets>,
) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 1000,
                ..Default::default()
            },
            ..Default::default()
        },
        StateScoped(state.clone()),
    ));

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
            StateScoped(state.clone()),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        image: ui_assets.panel.clone().into(),
                        ..Default::default()
                    },
                    ImageScaleMode::Sliced(TextureSlicer {
                        border: BorderRect::square(16.0),
                        ..Default::default()
                    }),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "Loading...",
                            TextStyle {
                                font: fonts.normal.clone(),
                                font_size: 72.0,
                                ..Default::default()
                            },
                        ),
                        ..Default::default()
                    });
                });
        });
}
