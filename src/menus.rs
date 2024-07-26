use bevy::{ecs::system::SystemId, prelude::*, utils::HashMap};
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
            .add_systems(OnEnter(crate::states::GameState::MainMenu), main_menu)
            .add_systems(Update, process_button_interactions);

        // Menu systems
        let play_id = app.register_system(play);
        app.insert_resource(MainMenuSystems {
            // TODO: new game, resume game, exit, etc.
            map: [("play".to_string(), play_id)].into(),
        });
    }
}

const BASE_BUTTON_COLOR: Color = Color::Srgba(bevy::color::palettes::basic::AQUA);
const HOVERED_BUTTON_COLOR: Color = Color::Srgba(bevy::color::palettes::css::ORANGE_RED);
const PRESSED_BUTTON_COLOR: Color = Color::Srgba(bevy::color::palettes::css::CRIMSON);

#[derive(Debug, Clone, AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(key = "panel")]
    pub panel: Handle<Image>,
}

#[derive(Debug, Resource)]
pub struct MainMenuSystems {
    pub map: HashMap<String, SystemId>,
}

#[derive(Debug, Default, Component)]
struct ButtonSystem(&'static str);

fn play(mut change_room: EventWriter<crate::room::ChangeRoom>) {
    change_room.send(crate::room::ChangeRoom {
        next_room_name: "Lovely Cottage".to_string(),
        coming_from: None,
    });
}

fn process_button_interactions(
    mut commands: Commands,
    mut query: Query<(&Interaction, &mut UiImage, &ButtonSystem)>,
    menu_systems: Res<MainMenuSystems>,
) {
    for (interaction, mut ui_image, button_system) in query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                ui_image.color = HOVERED_BUTTON_COLOR;
            }
            Interaction::Pressed => {
                ui_image.color = PRESSED_BUTTON_COLOR;
                let Some(system_id) = menu_systems.map.get(button_system.0) else {
                    error!("Menu system not found: {}", button_system.0);
                    continue;
                };
                commands.run_system(*system_id);
            }
            Interaction::None => {
                ui_image.color = BASE_BUTTON_COLOR;
            }
        }
    }
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
                        style: Style {
                            padding: UiRect::all(Val::Px(16.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        image: UiImage {
                            texture: ui_assets.panel.clone().into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ImageScaleMode::Sliced(TextureSlicer {
                        border: BorderRect::square(16.0),
                        ..Default::default()
                    }),
                    ButtonSystem("play"),
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle {
                        text: Text::from_section(
                            "PLAY",
                            TextStyle {
                                font: fonts.normal.clone(),
                                font_size: 54.0,
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
                        style: Style {
                            padding: UiRect::all(Val::Px(16.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..Default::default()
                        },
                        image: UiImage {
                            texture: ui_assets.panel.clone(),
                            color: BASE_BUTTON_COLOR,
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
