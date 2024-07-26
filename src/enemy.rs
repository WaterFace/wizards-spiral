use bevy::prelude::*;

#[derive(Debug, Default)]
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemyAlertEvent>().add_systems(
            Update,
            (move_enemies, alert_enemies, alert_visual)
                .run_if(in_state(crate::states::GameState::InGame)),
        );
    }
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

#[derive(Debug, Default, Component)]
pub enum EnemyState {
    #[default]
    Wander,
    Chase,
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

fn alert_visual(mut commands: Commands, mut events: EventReader<EnemyAlertEvent>) {
    const OFFSET: Vec3 = bevy_math::vec3(0.0, 16.0, 0.0);
    const VELOCITY: Vec2 = bevy_math::vec2(0.0, 1.0);
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
        ),
        With<Enemy>,
    >,
    player_query: Query<&GlobalTransform, With<crate::player::Player>>,
) {
    let player_pos = player_query
        .iter()
        .next()
        .map(|t| t.translation().truncate());

    for (enemy_state, stats, transform, mut controller) in query.iter_mut() {
        match enemy_state {
            EnemyState::Wander => {
                // TODO: implement wandering
                controller.desired_direction = Vec2::ZERO;
            }
            EnemyState::Chase => {
                let enemy_pos = transform.translation().truncate();
                let Some(player_pos) = player_pos else {
                    // no player position, so no need to move
                    controller.desired_direction = Vec2::ZERO;
                    continue;
                };

                let actual_distance_sqr = player_pos.distance_squared(enemy_pos);
                let desired_distance_sqr = stats.desired_distance * stats.desired_distance;

                // move toward the player if the actual distance is greater than the desired distance,
                // and away if the actual distance is less than the desired distance
                let dir = (player_pos - enemy_pos).clamp_length_max(1.0)
                    * f32::signum(actual_distance_sqr - desired_distance_sqr);

                controller.desired_direction = dir;
            }
        }
    }
}
