use bevy::{ecs::system::SystemId, prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::*;
use bevy_math::vec2;
use leafwing_input_manager::prelude::ActionState;

#[derive(Debug, Default)]
pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.enable_state_scoped_entities::<crate::states::GameState>()
            .enable_state_scoped_entities::<crate::states::AppState>()
            .enable_state_scoped_entities::<crate::states::MenuState>()
            .add_loading_state(
                LoadingState::new(crate::states::AppState::CoreLoading)
                    .continue_to_state(crate::states::AppState::RoomLoading)
                    .on_failure_continue_to_state(crate::states::AppState::AppClosing)
                    .load_collection::<UiAssets>()
                    .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                        "sprites/ui/ui.assets.ron",
                    ),
            )
            .add_loading_state(
                LoadingState::new(crate::states::AppState::CoreLoading)
                    .continue_to_state(crate::states::AppState::RoomLoading)
                    .on_failure_continue_to_state(crate::states::AppState::AppClosing)
                    .load_collection::<Stories>()
                    .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                        "stories/stories.assets.ron",
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
            .add_systems(Update, process_button_interactions)
            .add_systems(OnEnter(crate::states::MenuState::SkillsMenu), skills_menu)
            .add_systems(
                OnEnter(crate::states::GameState::Intro),
                (|| crate::states::GameState::Intro).pipe(spawn_story_menu),
            )
            .add_systems(
                OnEnter(crate::states::GameState::Outro),
                (
                    setup_outro,
                    (|| crate::states::GameState::Outro).pipe(spawn_story_menu),
                )
                    .chain(),
            )
            .add_systems(
                Update,
                open_close_skills_menu.run_if(in_state(crate::states::GameState::InGame)),
            );

        // Menu systems
        let continue_game_id = app.register_system(continue_game);
        let new_game_id = app.register_system(new_game);

        let next_page_id = app.register_system(next_page);
        let prev_page_id = app.register_system(prev_page);

        app.insert_resource(MainMenuSystems {
            map: [
                ("continue_game".to_string(), continue_game_id),
                ("new_game".to_string(), new_game_id),
                ("next_page".to_string(), next_page_id),
                ("prev_page".to_string(), prev_page_id),
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

    #[asset(key = "locked_icon")]
    pub locked_icon: Handle<Image>,
    #[asset(key = "armor_icon")]
    pub armor_icon: Handle<Image>,
    #[asset(key = "sword_icon")]
    pub sword_icon: Handle<Image>,
    #[asset(key = "shield_icon")]
    pub shield_icon: Handle<Image>,
    #[asset(key = "pants_icon")]
    pub pants_icon: Handle<Image>,
    #[asset(key = "mirror_icon")]
    pub mirror_icon: Handle<Image>,
    #[asset(key = "healing_icon")]
    pub healing_icon: Handle<Image>,
    #[asset(key = "speed_icon")]
    pub speed_icon: Handle<Image>,
}

#[derive(Debug, AssetCollection, Resource)]
pub struct Stories {
    #[asset(key = "intro")]
    pub intro: Handle<Story>,
    #[asset(key = "outro")]
    pub outro: Handle<Story>,
}

#[derive(Debug, Default, Clone, Resource, Asset, Reflect, serde::Deserialize)]
pub struct Story {
    pub pages: Vec<String>,
}

#[derive(Debug, Resource)]
pub struct StoryState<S> {
    pub page: usize,
    pub story: Story,
    pub prev_state: Option<S>,
    pub next_state: S,
}

#[derive(Debug, Resource)]
pub struct MainMenuSystems {
    pub map: HashMap<String, SystemId>,
}

#[derive(Debug, Default, Component)]
struct ButtonSystem(&'static str);

#[derive(Debug, Component, Default)]
struct ButtonPreviousInteraction(Option<Interaction>);

/// when inserted, indicates that we want to start a new game, deleting the previous save data
#[derive(Debug, Default, Resource)]
pub struct NewGame;

#[derive(Debug, Default, Component)]
pub struct StoryTextDisplay;

#[derive(Debug, Default, Component)]
pub struct PageCount;

fn next_page(
    mut commands: Commands,
    mut next_state: ResMut<NextState<crate::states::GameState>>,
    mut display_query: Query<&mut Text, With<StoryTextDisplay>>,
    mut page_count_query: Query<&mut Text, (With<PageCount>, Without<StoryTextDisplay>)>,
    mut story_state: ResMut<StoryState<crate::states::GameState>>,
    fonts: Res<crate::text::Fonts>,
) {
    if story_state.page + 1 >= story_state.story.pages.len() {
        next_state.set(story_state.next_state);
        commands.remove_resource::<StoryState<crate::states::GameState>>();
    } else {
        story_state.page += 1;
        let mut text = display_query
            .get_single_mut()
            .expect("There should only be one StoryTextDisplay");

        let sections = crate::util::highlight_text(
            &story_state.story.pages[story_state.page],
            bevy::color::palettes::basic::WHITE.into(),
            HOVERED_BUTTON_COLOR,
            36.0,
            fonts.normal.clone(),
        );
        *text = Text::from_sections(sections);

        let mut text = page_count_query
            .get_single_mut()
            .expect("There should only be one PageCount");

        *text = Text::from_section(
            format!(
                "{} / {}",
                story_state.page + 1,
                story_state.story.pages.len()
            ),
            TextStyle {
                color: bevy::color::palettes::basic::WHITE.into(),
                font: fonts.normal.clone(),
                font_size: 36.0,
            },
        );
    }
}

fn prev_page(
    mut commands: Commands,
    mut next_state: ResMut<NextState<crate::states::GameState>>,
    mut display_query: Query<&mut Text, With<StoryTextDisplay>>,
    mut page_count_query: Query<&mut Text, (With<PageCount>, Without<StoryTextDisplay>)>,
    mut story_state: ResMut<StoryState<crate::states::GameState>>,
    fonts: Res<crate::text::Fonts>,
) {
    if story_state.page == 0 {
        if let Some(prev_state) = story_state.prev_state {
            next_state.set(prev_state);
            commands.remove_resource::<StoryState<crate::states::GameState>>();
        }
        // otherwise do nothing
    } else {
        story_state.page -= 1;
        let mut text = display_query
            .get_single_mut()
            .expect("There should only be one StoryTextDisplay");

        let sections = crate::util::highlight_text(
            &story_state.story.pages[story_state.page],
            bevy::color::palettes::basic::WHITE.into(),
            HOVERED_BUTTON_COLOR,
            36.0,
            fonts.normal.clone(),
        );
        *text = Text::from_sections(sections);

        let mut text = page_count_query
            .get_single_mut()
            .expect("There should only be one PageCount");

        *text = Text::from_section(
            format!(
                "{} / {}",
                story_state.page + 1,
                story_state.story.pages.len()
            ),
            TextStyle {
                color: bevy::color::palettes::basic::WHITE.into(),
                font: fonts.normal.clone(),
                font_size: 36.0,
            },
        );
    }
}

fn continue_game(mut next_state: ResMut<NextState<crate::states::GameState>>) {
    next_state.set(crate::states::GameState::RestartCycle);
}

fn new_game(
    mut commands: Commands,
    mut next_state: ResMut<NextState<crate::states::GameState>>,
    stories: Res<Stories>,
    story_assets: Res<Assets<Story>>,
) {
    commands.init_resource::<NewGame>();
    let intro = story_assets
        .get(&stories.intro)
        .expect("Story assets have loaded by this point");
    commands.insert_resource(StoryState {
        page: 0,
        next_state: crate::states::GameState::RestartCycle,
        prev_state: Some(crate::states::GameState::MainMenu),
        story: intro.clone(),
    });
    next_state.set(crate::states::GameState::Intro);
}

fn setup_outro(mut commands: Commands, stories: Res<Stories>, story_assets: Res<Assets<Story>>) {
    let outro = story_assets
        .get(&stories.outro)
        .expect("Story assets have loaded by this point");
    commands.insert_resource(StoryState {
        page: 0,
        next_state: crate::states::GameState::MainMenu,
        prev_state: None,
        story: outro.clone(),
    });
}

fn spawn_story_menu<S: States + Clone>(
    In(state): In<S>,
    mut commands: Commands,
    fonts: Res<crate::text::Fonts>,
    story_state: Res<StoryState<crate::states::GameState>>,
    ui_assets: Res<UiAssets>,
) {
    commands.spawn((crate::camera::menu_camera(), StateScoped(state.clone())));

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
        Name::new("Menu Background"),
    ));
    let prev_page = commands.spawn_button(
        crate::states::GameState::MainMenu,
        "Previous",
        Some("prev_page"),
        fonts.normal.clone(),
        ui_assets.panel.clone(),
    );
    let page_count = commands
        .spawn((
            TextBundle {
                style: Style {
                    margin: UiRect::all(Val::Px(16.0)),
                    justify_self: JustifySelf::Center,
                    align_self: AlignSelf::Center,
                    ..Default::default()
                },
                text: Text::from_section(
                    format!(
                        "{} / {}",
                        story_state.page + 1,
                        story_state.story.pages.len() + 1
                    ),
                    TextStyle {
                        color: bevy::color::palettes::basic::WHITE.into(),
                        font: fonts.normal.clone(),
                        font_size: 36.0,
                    },
                ),
                ..Default::default()
            },
            PageCount,
        ))
        .id();
    let next_page = commands.spawn_button(
        crate::states::GameState::MainMenu,
        "Next",
        Some("next_page"),
        fonts.normal.clone(),
        ui_assets.panel.clone(),
    );

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(50.0),
                    height: Val::Percent(75.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
            StateScoped(state.clone()),
            Name::new("Story Menu Root"),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ImageBundle {
                        style: Style {
                            padding: UiRect::all(Val::Px(16.0)),
                            width: Val::Percent(100.0),
                            flex_wrap: FlexWrap::Wrap,
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::Start,
                            align_content: AlignContent::Center,
                            min_height: Val::Px(400.0),
                            ..Default::default()
                        },
                        image: UiImage {
                            texture: ui_assets.panel.clone(),
                            color: bevy::color::palettes::tailwind::GRAY_500.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ImageScaleMode::Sliced(TextureSlicer {
                        border: BorderRect::square(16.0),
                        ..Default::default()
                    }),
                    Name::new("Story Menu Text Container"),
                ))
                .with_children(|parent| {
                    let sections = crate::util::highlight_text(
                        &story_state.story.pages[story_state.page],
                        bevy::color::palettes::basic::WHITE.into(),
                        HOVERED_BUTTON_COLOR,
                        36.0,
                        fonts.normal.clone(),
                    );
                    parent.spawn((
                        TextBundle {
                            style: Style {
                                margin: UiRect::all(Val::Px(16.0)),
                                ..Default::default()
                            },
                            text: Text::from_sections(sections),
                            ..Default::default()
                        },
                        StoryTextDisplay,
                        Name::new("Story Menu Text Display"),
                    ));
                });
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .add_child(prev_page)
                .add_child(page_count)
                .add_child(next_page);
        });
}

fn process_button_interactions(
    mut commands: Commands,
    mut query: Query<(
        &Interaction,
        &mut ButtonPreviousInteraction,
        &mut UiImage,
        &ButtonSystem,
    )>,
    menu_systems: Res<MainMenuSystems>,
) {
    for (interaction, mut previous_interaction, mut ui_image, button_system) in query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                ui_image.color = HOVERED_BUTTON_COLOR;
                if matches!(previous_interaction.0, Some(Interaction::Pressed)) {
                    let Some(system_id) = menu_systems.map.get(button_system.0) else {
                        error!("Menu system not found: {}", button_system.0);
                        continue;
                    };
                    commands.run_system(*system_id);
                }
            }
            Interaction::Pressed => {
                ui_image.color = PRESSED_BUTTON_COLOR;
            }
            Interaction::None => {
                ui_image.color = BASE_BUTTON_COLOR;
            }
        }
        previous_interaction.0 = Some(*interaction);
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
        Name::new("Menu Background"),
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
            Name::new("Main Menu Root"),
        ))
        .with_children(|parent| {
            parent.spawn((
                ImageBundle {
                    image: ui_assets.title.clone().into(),
                    style: Style {
                        height: Val::Percent(50.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Name::new("Main Menu Title"),
            ));
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
        Name::new("Loading Screen Background"),
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
            Name::new("Loading Screen Root"),
        ))
        .add_child(loading_button);
}

fn open_close_skills_menu(
    menu_actions: Res<ActionState<crate::input::MenuAction>>,
    current_state: Res<State<crate::states::MenuState>>,
    mut next_state: ResMut<NextState<crate::states::MenuState>>,
) {
    match current_state.get() {
        &crate::states::MenuState::None => {
            if menu_actions.pressed(&crate::input::MenuAction::SkillsMenu) {
                info!("open_close_skills_menu: opening skills menu");
                next_state.set(crate::states::MenuState::SkillsMenu);
            }
        }
        &crate::states::MenuState::SkillsMenu => {
            if menu_actions.released(&crate::input::MenuAction::SkillsMenu) {
                info!("open_close_skills_menu: closing skills menu");
                next_state.set(crate::states::MenuState::None);
            }
        }
    }
}

fn skills_menu(
    mut commands: Commands,
    player_skills: Res<crate::skills::PlayerSkills>,
    fonts: Res<crate::text::Fonts>,
    ui_assets: Res<UiAssets>,
) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Vw(50.0),
                    height: Val::Vh(75.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
            StateScoped(crate::states::MenuState::SkillsMenu),
            Name::new("Skills Menu Root"),
        ))
        .with_children(|parent| {
            for skill in crate::skills::Skill::iter() {
                let skill_icon = match skill {
                    crate::skills::Skill::Armor => &ui_assets.armor_icon,
                    crate::skills::Skill::Sword => &ui_assets.sword_icon,
                    crate::skills::Skill::Shield => &ui_assets.shield_icon,
                    crate::skills::Skill::Pants => &ui_assets.pants_icon,
                    crate::skills::Skill::Mirror => &ui_assets.mirror_icon,
                    crate::skills::Skill::Healing => &ui_assets.healing_icon,
                    crate::skills::Skill::Speed => &ui_assets.speed_icon,
                };
                parent
                    .spawn((
                        ImageBundle {
                            style: Style {
                                padding: UiRect::all(Val::Px(16.0)),
                                width: Val::Percent(100.0),
                                height: Val::Vh(10.0),
                                flex_wrap: FlexWrap::Wrap,
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::Start,
                                align_content: AlignContent::Center,
                                ..Default::default()
                            },
                            image: UiImage {
                                texture: ui_assets.panel.clone(),
                                color: bevy::color::palettes::tailwind::GRAY_500.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ImageScaleMode::Sliced(TextureSlicer {
                            border: BorderRect::square(16.0),
                            ..Default::default()
                        }),
                        Name::new("Skills Menu Entry"),
                    ))
                    .with_children(|parent| {
                        let icon = if player_skills.get_unlocked(skill) {
                            skill_icon.clone()
                        } else {
                            ui_assets.locked_icon.clone()
                        };
                        parent.spawn((
                            ImageBundle {
                                image: icon.into(),
                                background_color: bevy::color::palettes::tailwind::GRAY_800.into(),
                                ..Default::default()
                            },
                            Name::new("Skills Menu Icon"),
                        ));
                        let text_sections = crate::util::highlight_text(
                            &if player_skills.get_unlocked(skill) {
                                player_skills.description(skill)
                            } else {
                                "???".to_string()
                            },
                            bevy::color::palettes::basic::WHITE.into(),
                            HOVERED_BUTTON_COLOR,
                            24.0,
                            fonts.normal.clone(),
                        );
                        parent.spawn((
                            TextBundle {
                                style: Style {
                                    margin: UiRect::all(Val::Px(16.0)),
                                    ..Default::default()
                                },
                                text: Text::from_sections(text_sections),
                                ..Default::default()
                            },
                            Name::new("Skills Menu Text"),
                        ));
                    });
            }
        });
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
            .spawn((
                TextBundle {
                    text: Text::from_section(
                        button_text,
                        TextStyle {
                            font,
                            font_size: 54.0,
                            ..Default::default()
                        },
                    ),
                    ..Default::default()
                },
                Name::new("Button Text"),
            ))
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
            ButtonPreviousInteraction::default(),
            Name::new("Button"),
        ));

        if let Some(system_name) = system_name {
            button.insert(ButtonSystem(system_name));
        }

        button.add_child(text_id);

        return button.id();
    }
}
