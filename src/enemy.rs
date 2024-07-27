use bevy::prelude::*;

#[derive(Debug, Default)]
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemyAlertEvent>()
            .add_event::<EnemyDeathEvent>()
            .add_systems(
                Update,
                (
                    move_enemies,
                    alert_enemies,
                    alert_visual,
                    handle_enemy_death,
                )
                    .run_if(in_state(crate::states::GameState::InGame)),
            );
    }
}

#[derive(Debug, Clone, Component, Asset, Reflect, serde::Deserialize)]
pub struct BossStats {
    /// The boss's name
    pub name: String,
    /// Which skill, if any, is unlocked by defeating this boss
    pub skill_unlocked: Option<crate::skills::Skill>,
    /// scalar on the boss's size
    pub scale: f32,
    /// the rest of the stats
    pub stats: EnemyStats,
}

#[derive(Debug, Clone, Component, Asset, Reflect, serde::Deserialize)]
pub struct EnemyStats {
    /// Type of enemy; either Melee or Ranged
    pub enemy_type: EnemyType,

    /// Base health. will be multiplied by a per-room difficulty scalar
    pub health: f32,
    /// movement speed
    pub speed: f32,
    /// mass controls how much knockback this enemy causes, and how much it recieves
    pub mass: f32,

    /// Radius at which the enemy is alerted to the player. If the player gets this close, the enemy begins to chase
    pub alert_radius: f32,
    /// Radius at which the enemy will stop chasing the player. If the player is this far away, the enemy will stop chasing
    pub chase_radius: f32,
    /// How far the enemy will try to stay from the player.
    pub desired_distance: f32,
}

impl EnemyStats {
    pub fn melee_damage(&self) -> f32 {
        match self.enemy_type {
            EnemyType::Melee { melee_damage } => melee_damage,
            EnemyType::Ranged { melee_damage, .. } => melee_damage,
        }
    }
}

#[derive(Debug, Component, Clone, Copy, Reflect, serde::Deserialize)]
pub enum EnemyType {
    Melee {
        melee_damage: f32,
    },
    Ranged {
        melee_damage: f32,
        projectile_damage: f32,
        projectile_speed: f32,
        projectile_lifetime: f32,
        homing: bool,
        delay: f32,
    },
}

#[derive(Debug, Default, Component, Clone, Copy)]
pub struct Enemy;

#[derive(Debug, Default, Component, Clone, Copy)]
pub struct Boss;

#[derive(Debug, Event, Clone)]
pub struct EnemyDeathEvent {
    pub entity: Entity,
}

#[derive(Debug, Component)]
pub struct EnemyHealth {
    pub maximum: f32,
    pub current: f32,
}

impl EnemyHealth {
    pub fn new(max: f32) -> Self {
        EnemyHealth {
            maximum: max,
            current: max,
        }
    }
}

#[derive(Debug, Default, Component)]
pub enum EnemyState {
    #[default]
    Wander,
    Chase,
}

#[derive(Debug, Component)]
pub struct WanderState {
    pub timer: Timer,
    pub min_delay: f32,
    pub max_delay: f32,
    pub target: Option<Vec2>,
}

impl WanderState {
    pub fn new<R: rand::Rng + ?Sized>(min: f32, max: f32, rng: &mut R) -> Self {
        let mut wander_state = Self {
            min_delay: min,
            max_delay: max,
            timer: Timer::default(),
            target: None,
        };
        wander_state.reset(rng);
        wander_state
    }

    pub fn reset<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) {
        let t = rng.gen_range(self.min_delay..self.max_delay);
        self.timer.reset();
        self.timer
            .set_duration(std::time::Duration::from_secs_f32(t));
    }
}

#[derive(Debug, Clone, Event)]
pub struct EnemyAlertEvent {
    pub enemy: Entity,
    pub ty: EnemyAlertEventType,
}

#[derive(Debug, Clone)]
pub enum EnemyAlertEventType {
    Alert,
    TooFar,
}

fn handle_enemy_death(
    mut commands: Commands,
    mut events: EventReader<EnemyDeathEvent>,
    enemy_query: Query<(
        &GlobalTransform,
        &crate::room::SpawnerIndex,
        &Sprite,
        &Handle<Image>,
    )>,
    current_room: Res<crate::room::CurrentRoom>,
    mut room_state: ResMut<crate::room::PersistentRoomState>,
) {
    let Some(current_room_state) = room_state.rooms.get_mut(&current_room.info.name) else {
        error!(
            "handle_enemy_death: Couldn't find persistent room state for room {}!",
            current_room.info.name
        );
        return;
    };

    for EnemyDeathEvent { entity } in events.read() {
        let Ok((global_transform, spawner_index, sprite, texture)) = enemy_query.get(*entity)
        else {
            warn!(
                "handle_enemy_death: Got EnemyDeathEvent for non-existent enemy {:?}",
                entity
            );
            continue;
        };

        // set the persistent state so this enemy won't spawn anymore for this cycle
        current_room_state.spawners[spawner_index.0].active = false;

        // despawn the enemy
        commands.entity(*entity).despawn_recursive();

        // spawn a corpse
        commands.spawn((
            SpriteBundle {
                sprite: sprite.clone(),
                texture: texture.clone(),
                transform: Transform::from_translation(global_transform.translation().with_z(-5.0))
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::PI * 0.5)),
                ..Default::default()
            },
            crate::room::RoomObject,
        ));
    }
}

fn alert_visual(mut commands: Commands, mut events: EventReader<EnemyAlertEvent>) {
    const OFFSET: Vec3 = bevy_math::vec3(0.0, 16.0, 0.0);
    const VELOCITY: Vec2 = bevy_math::vec2(0.0, 8.0);
    for EnemyAlertEvent { enemy, ty } in events.read() {
        match ty {
            EnemyAlertEventType::TooFar => {
                // do nothing
                continue;
            }
            EnemyAlertEventType::Alert => {
                // go on...
            }
        }

        let floating_text = commands
            .spawn((
                SpatialBundle {
                    transform: Transform::from_translation(OFFSET),
                    ..Default::default()
                },
                crate::text::TextMarker {
                    color: Some(bevy::color::palettes::basic::YELLOW.into()),
                    fancy: false,
                    font_size: 18.0,
                    text: "!".to_string(),
                    ..Default::default()
                },
                crate::text::FloatingText {
                    timer: Timer::from_seconds(0.5, TimerMode::Once),
                    velocity: VELOCITY,
                    ..Default::default()
                },
            ))
            .id();
        commands.entity(*enemy).add_child(floating_text);
    }
}

fn alert_enemies(
    mut enemy_query: Query<(Entity, &mut EnemyState, &EnemyStats, &GlobalTransform), With<Enemy>>,
    player_query: Query<&GlobalTransform, With<crate::player::Player>>,
    mut writer: EventWriter<EnemyAlertEvent>,
) {
    let Some(player_pos) = player_query
        .iter()
        .next()
        .map(|t| t.translation().truncate())
    else {
        warn!("no player entity");
        return;
    };
    for (enemy, mut enemy_state, enemy_stats, enemy_transform) in enemy_query.iter_mut() {
        let enemy_pos = enemy_transform.translation().truncate();

        let sqr_dist = player_pos.distance_squared(enemy_pos);

        match *enemy_state {
            EnemyState::Wander
                if sqr_dist <= enemy_stats.alert_radius * enemy_stats.alert_radius =>
            {
                *enemy_state = EnemyState::Chase;
                writer.send(EnemyAlertEvent {
                    enemy,
                    ty: EnemyAlertEventType::Alert,
                });
            }
            EnemyState::Chase if sqr_dist > enemy_stats.chase_radius * enemy_stats.chase_radius => {
                *enemy_state = EnemyState::Wander;
                writer.send(EnemyAlertEvent {
                    enemy,
                    ty: EnemyAlertEventType::TooFar,
                });
            }
            _ => {
                continue;
            }
        }
    }
}

fn move_enemies(
    mut query: Query<
        (
            &EnemyState,
            &EnemyStats,
            &GlobalTransform,
            &mut crate::character_controller::CharacterController,
            &mut WanderState,
        ),
        With<Enemy>,
    >,
    player_query: Query<&GlobalTransform, With<crate::player::Player>>,
    time: Res<Time>,
    mut rng: ResMut<crate::rand::GlobalRng>,
) {
    const CLOSE_ENOUGH: f32 = 16.0;
    let player_pos = player_query
        .iter()
        .next()
        .map(|t| t.translation().truncate());

    for (enemy_state, stats, transform, mut controller, mut wander_state) in query.iter_mut() {
        let enemy_pos = transform.translation().truncate();
        match enemy_state {
            EnemyState::Wander => {
                wander_state.timer.tick(time.delta());
                if wander_state.timer.finished() {
                    let new_target = Circle::new(75.0).sample_interior(rng.as_mut());
                    wander_state.target = Some(new_target + enemy_pos);
                    wander_state.reset(rng.as_mut());
                }

                if let Some(target) = wander_state.target {
                    if target.distance(enemy_pos) < CLOSE_ENOUGH {
                        controller.desired_direction = Vec2::ZERO;
                        continue;
                    }
                    controller.desired_direction = (target - enemy_pos).clamp_length_max(1.0) * 0.5;
                } else {
                    controller.desired_direction = Vec2::ZERO;
                }
            }
            EnemyState::Chase => {
                let Some(player_pos) = player_pos else {
                    // no player position, so no need to move
                    controller.desired_direction = Vec2::ZERO;
                    continue;
                };

                let actual_distance = player_pos.distance(enemy_pos);
                let desired_distance = stats.desired_distance;

                // move toward the player if the actual distance is greater than the desired distance,
                // and away if the actual distance is less than the desired distance
                let dir = (player_pos - enemy_pos).clamp_length_max(1.0)
                    * f32::signum(actual_distance - desired_distance);
                if (actual_distance - desired_distance).abs() < CLOSE_ENOUGH {
                    controller.desired_direction = Vec2::ZERO;
                    continue;
                }

                controller.desired_direction = dir;
            }
        }
    }
}
