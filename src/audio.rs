use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Debug, Default)]
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalVolume::new(0.5))
            .insert_resource(Muted { muted: false })
            .add_loading_state(
                LoadingState::new(crate::states::AppState::CoreLoading)
                    .continue_to_state(crate::states::AppState::RoomLoading)
                    .on_failure_continue_to_state(crate::states::AppState::AppClosing)
                    .load_collection::<SoundAssets>()
                    .with_dynamic_assets_file::<StandardDynamicAssetCollection>(
                        "sounds/sounds.assets.ron",
                    ),
            )
            .add_systems(Update, (add_spatial_listener, handle_mute_button))
            .add_systems(
                Update,
                (
                    melee_attack_sounds,
                    projectile_hit_sounds,
                    shield_block_sounds,
                    death_sounds,
                    new_skill_sounds,
                    projectile_reflect_sounds,
                    heal_sounds,
                    update_running_sound_emitter,
                )
                    .run_if(in_state(crate::states::GameState::InGame)),
            )
            // The player is spawned in the room transition state
            .add_systems(
                Update,
                spawn_running_sound_emitter
                    .run_if(in_state(crate::states::GameState::RoomTransition)),
            );
    }
}

#[derive(Debug, Resource, AssetCollection)]
pub struct SoundAssets {
    #[asset(key = "melee_hit")]
    pub melee_hit: Handle<AudioSource>,
    #[asset(key = "projectile_hit")]
    pub projectile_hit: Handle<AudioSource>,
    #[asset(key = "shield_block")]
    pub shield_block: Handle<AudioSource>,
    #[asset(key = "death")]
    pub death: Handle<AudioSource>,
    #[asset(key = "new_skill")]
    pub new_skill: Handle<AudioSource>,
    #[asset(key = "projectile_reflect")]
    pub projectile_reflect: Handle<AudioSource>,
    #[asset(key = "running")]
    pub running: Handle<AudioSource>,
    #[asset(key = "heal")]
    pub heal: Handle<AudioSource>,
}

#[derive(Debug, Resource)]
pub struct Muted {
    pub muted: bool,
}

fn handle_mute_button(
    mut muted: ResMut<Muted>,
    mut global_volume: ResMut<GlobalVolume>,
    menu_action_state: Res<ActionState<crate::input::MenuAction>>,
) {
    if menu_action_state.just_pressed(&crate::input::MenuAction::MuteSounds) {
        if muted.muted {
            muted.muted = false;
            global_volume.volume = bevy::audio::Volume::new(0.5);
        } else {
            muted.muted = true;
            global_volume.volume = bevy::audio::Volume::new(0.0);
        }
    }
}

fn add_spatial_listener(
    mut commands: Commands,
    camera_query: Query<Entity, Added<crate::camera::GameCamera>>,
) {
    for camera_entity in camera_query.iter() {
        commands
            .entity(camera_entity)
            // for some reason bevy's default is backwards
            .insert(SpatialListener::new(-16.0));
    }
}

fn melee_attack_sounds(
    mut commands: Commands,
    sound_assets: Res<SoundAssets>,
    mut melee_attacks: EventReader<crate::damage::MeleeAttackEvent>,
) {
    for crate::damage::MeleeAttackEvent { .. } in melee_attacks.read() {
        commands.spawn(AudioSourceBundle {
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                ..Default::default()
            },
            source: sound_assets.melee_hit.clone(),
        });
    }
}

fn projectile_hit_sounds(
    mut commands: Commands,
    sound_assets: Res<SoundAssets>,
    mut projectile_hits: EventReader<crate::projectiles::ProjectileHitEvent>,
) {
    for crate::projectiles::ProjectileHitEvent { target, .. } in projectile_hits.read() {
        if let Some(mut entity_commands) = commands.get_entity(*target) {
            entity_commands.with_children(|parent| {
                parent.spawn((
                    SpatialBundle::default(),
                    AudioSourceBundle {
                        settings: PlaybackSettings {
                            mode: bevy::audio::PlaybackMode::Despawn,
                            spatial: true,
                            ..Default::default()
                        },
                        source: sound_assets.projectile_hit.clone(),
                    },
                ));
            });
        }
    }
}

fn shield_block_sounds(
    mut commands: Commands,
    sound_assets: Res<SoundAssets>,
    mut shield_blocks: EventReader<crate::damage::DamageBlockedEvent>,
) {
    for crate::damage::DamageBlockedEvent { .. } in shield_blocks.read() {
        commands.spawn(AudioSourceBundle {
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                ..Default::default()
            },
            source: sound_assets.shield_block.clone(),
        });
    }
}

fn death_sounds(
    mut commands: Commands,
    sound_assets: Res<SoundAssets>,
    mut enemy_deaths: EventReader<crate::enemy::EnemyDeathEvent>,
    mut player_deaths: EventReader<crate::player::PlayerDeathEvent>,
) {
    for crate::enemy::EnemyDeathEvent { pos, .. } in enemy_deaths.read() {
        commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(pos.extend(0.0)),
                ..Default::default()
            },
            AudioSourceBundle {
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    spatial: true,
                    ..Default::default()
                },
                source: sound_assets.death.clone(),
            },
        ));
    }

    for crate::player::PlayerDeathEvent { pos, .. } in player_deaths.read() {
        commands.spawn((
            SpatialBundle {
                transform: Transform::from_translation(pos.extend(0.0)),
                ..Default::default()
            },
            AudioSourceBundle {
                settings: PlaybackSettings {
                    mode: bevy::audio::PlaybackMode::Despawn,
                    spatial: true,
                    ..Default::default()
                },
                source: sound_assets.death.clone(),
            },
        ));
    }
}

fn new_skill_sounds(
    mut commands: Commands,
    sound_assets: Res<SoundAssets>,
    mut new_skills: EventReader<crate::skills::SkillFirstUnlockedEvent>,
) {
    for crate::skills::SkillFirstUnlockedEvent { .. } in new_skills.read() {
        commands.spawn(AudioSourceBundle {
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                ..Default::default()
            },
            source: sound_assets.new_skill.clone(),
        });
    }
}

fn projectile_reflect_sounds(
    mut commands: Commands,
    sound_assets: Res<SoundAssets>,
    mut projectile_reflects: EventReader<crate::projectiles::ProjectileReflectEvent>,
) {
    for crate::projectiles::ProjectileReflectEvent { .. } in projectile_reflects.read() {
        commands.spawn(AudioSourceBundle {
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                ..Default::default()
            },
            source: sound_assets.projectile_reflect.clone(),
        });
    }
}

fn heal_sounds(
    mut commands: Commands,
    sound_assets: Res<SoundAssets>,
    mut heals: EventReader<crate::skills::HealEvent>,
) {
    for crate::skills::HealEvent { .. } in heals.read() {
        commands.spawn(AudioSourceBundle {
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                ..Default::default()
            },
            source: sound_assets.heal.clone(),
        });
    }
}

#[derive(Debug, Default, Component)]
struct RunningSoundEmitter;

fn spawn_running_sound_emitter(
    mut commands: Commands,
    player_query: Query<Entity, Added<crate::player::Player>>,
    sound_assets: Res<SoundAssets>,
) {
    for player_entity in player_query.iter() {
        info!("spawning running sound emitter");
        commands.entity(player_entity).with_children(|parent| {
            parent.spawn((
                SpatialBundle::default(),
                AudioSourceBundle {
                    settings: PlaybackSettings {
                        mode: bevy::audio::PlaybackMode::Loop,
                        paused: true,
                        spatial: true,
                        ..Default::default()
                    },
                    source: sound_assets.running.clone(),
                },
                RunningSoundEmitter,
            ));
        });
    }
}

fn update_running_sound_emitter(
    query: Query<&SpatialAudioSink, With<RunningSoundEmitter>>,
    player_skills: Res<crate::skills::PlayerSkills>,
    action_state: Res<ActionState<crate::input::PlayerAction>>,
    muted: Res<Muted>,
) {
    let is_running = {
        if let Some(axis_pair) = action_state.axis_pair(&crate::input::PlayerAction::Move) {
            axis_pair.length() > 0.1
        } else {
            false
        }
    };
    for audio_sink in query.iter() {
        if is_running && !muted.muted {
            audio_sink.play();
        } else {
            audio_sink.pause();
        }
        audio_sink.set_speed(player_skills.speed());
    }
}
