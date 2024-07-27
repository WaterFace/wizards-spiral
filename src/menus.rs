use bevy::{ecs::system::SystemId, prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::*;
use bevy_math::vec2;

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
        let continue_game_id = app.register_system(continue_game);
        let new_game_id = app.register_system(new_game);
        app.insert_resource(MainMenuSystems {
            map: [
                ("continue_game".to_string(), continue_game_id),
                ("new_game".to_string(), new_game_id),
            ]
            .into(),
        });
    }
}

const BASE_BUTTON_COLOR: Color = Color::Srgba(bevy::color::palettes::basic::GRAY);
const HOVERED_BUTTON_COLOR: Color = Color::Srgba(bevy::color::palettes::css::SKY_BLUE);
const PRESSED_BUTTON_COLOR: Color = Color::Srgba(bevy::color::palettes::css::ORANGE_RED);

fn button_style() -> Style {
    Style {
        padding: UiRect::all(Val::Px(16.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        max_width: Val::Px(300.0),
        min_width: Val::Px(300.0),
        min_height: Val::Px(65.0),
        max_height: Val::Px(65.0),
        ..Default::default()
    }
}

#[derive(Debug, Clone, AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(key = "panel")]
    pub panel: Handle<Image>,

    #[asset(key = "menu_background")]
    pub menu_background: Handle<Image>,

    #[asset(key = "title")]
    pub title: Handle<Image>,
}

#[derive(Debug, Resource)]
pub struct MainMenuSystems {
    pub map: HashMap<String, SystemId>,
}

#[derive(Debug, Default, Component)]
struct ButtonSystem(&'static str);

/// when inserted, indicates that we want to start a new game, deleting the previous save data
#[derive(Debug, Default, Resource)]
pub struct NewGame;

fn continue_game(mut next_state: ResMut<NextState<crate::states::GameState>>) {
    next_state.set(crate::states::GameState::RestartCycle);
}

fn new_game(mut commands: Commands, mut next_state: ResMut<NextState<crate::states::GameState>>) {
    commands.init_resource::<NewGame>();
    next_state.set(crate::states::GameState::RestartCycle);
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

fn main_menu(
    mut commands: Commands,
    fonts: Res<crate::text::Fonts>,
    ui_assets: Res<UiAssets>,
    save_data: Option<Res<crate::save_data::SaveData>>,
) {
    commands.spawn((
        crate::camera::menu_camera(),
        StateScoped(crate::states::GameState::MainMenu),
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(vec2(1280.0, 720.0)),
                ..Default::default()
            },
            texture: ui_assets.menu_background.clone(),
            ..Default::default()
        },
        StateScoped(crate::states::GameState::MainMenu),
    ));

    let base = commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                ..Default::default()
            },
            StateScoped(crate::states::GameState::MainMenu),
        ))
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image: ui_assets.title.clone().into(),
                style: Style {
                    height: Val::Percent(50.0),
                    ..Default::default()
                },
                ..Default::default()
            });
        })
        .id();

    let new_game_button = commands.spawn_button(
        crate::states::GameState::MainMenu,
        "New Game",
        Some("new_game"),
        fonts.normal.clone(),
        ui_assets.panel.clone(),
    );
    if save_data.is_some() {
        let continue_button = commands.spawn_button(
            crate::states::GameState::MainMenu,
            "Continue Game",
            Some("continue_game"),
            fonts.normal.clone(),
            ui_assets.panel.clone(),
        );
        commands.entity(base).add_child(continue_button);
    }
    commands.entity(base).add_child(new_game_button);
}

fn loading_screen<S: States>(
    In(state): In<S>,
    mut commands: Commands,
    fonts: Res<crate::text::Fonts>,
    ui_assets: Res<UiAssets>,
) {
    commands.spawn((crate::camera::menu_camera(), StateScoped(state.clone())));

    let loading_button = commands.spawn_button(
        state.clone(),
        "Loading...",
        None,
        fonts.normal.clone(),
        ui_assets.panel.clone(),
    );

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(vec2(1280.0, 720.0)),
                ..Default::default()
            },
            texture: ui_assets.menu_background.clone(),
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
        .add_child(loading_button);
}

trait MenuCommandsExt {
    fn spawn_button<S: States>(
        &mut self,
        state: S,
        button_text: impl Into<String>,
        system_name: Option<&'static str>,
        font: Handle<Font>,
        texture: Handle<Image>,
    ) -> Entity;
}

impl MenuCommandsExt for Commands<'_, '_> {
    fn spawn_button<S: States>(
        &mut self,
        state: S,
        button_text: impl Into<String>,
        system_name: Option<&'static str>,
        font: Handle<Font>,
        texture: Handle<Image>,
    ) -> Entity {
        let text_id = self
            .spawn(TextBundle {
                text: Text::from_section(
                    button_text,
                    TextStyle {
                        font,
                        font_size: 54.0,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            })
            .id();
        let mut button = self.spawn((
            ButtonBundle {
                style: button_style(),
                image: UiImage {
                    color: BASE_BUTTON_COLOR,
                    texture,
                    ..Default::default()
                },
                ..Default::default()
            },
            ImageScaleMode::Sliced(TextureSlicer {
                border: BorderRect::square(16.0),
                ..Default::default()
            }),
            StateScoped(state),
        ));

        if let Some(system_name) = system_name {
            button.insert(ButtonSystem(system_name));
        }

        button.add_child(text_id);

        return button.id();
    }
}
