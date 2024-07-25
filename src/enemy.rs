use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Default)]
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemyAlertEvent>().add_systems(
            Update,
            (move_enemies, alert_enemies).run_if(in_state(crate::states::GameState::InGame)),
        );
    }
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

fn alert_enemies(
    mut enemy_query: Query<
        (
            Entity,
            &mut EnemyState,
            &crate::assets::EnemyStats,
            &GlobalTransform,
        ),
        With<Enemy>,
    >,
    player_query: Query<&GlobalTransform, With<crate::player::Player>>,
    mut writer: EventWriter<EnemyAlertEvent>,
    mut gizmos: Gizmos,
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

        gizmos.circle_2d(
            enemy_pos,
            enemy_stats.chase_radius,
            bevy::color::palettes::basic::GREEN,
        );
        gizmos.circle_2d(
            enemy_pos,
            enemy_stats.alert_radius,
            bevy::color::palettes::basic::RED,
        );

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
            &crate::assets::EnemyStats,
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

pub fn spawn_melee_enemies(
    In(to_spawn): In<Vec<(Vec2, crate::assets::EnemyStats)>>,
    mut commands: Commands,
) {
    for (pos, stats) in to_spawn {
        commands.spawn((
            // TODO: make movement speed, etc. configurable
            crate::character_controller::CharacterController {
                acceleration: 10.0,
                max_speed: 64.0,
                ..Default::default()
            },
            Enemy,
            EnemyState::default(),
            RigidBody::Dynamic,
            // TODO: make size configurable?
            Collider::ball(16.0),
            ColliderMassProperties::Density(0.0),
            AdditionalMassProperties::MassProperties(MassProperties {
                mass: 1.0,
                ..Default::default()
            }),
            Velocity::default(),
            ExternalImpulse::default(),
            TransformBundle::from_transform(Transform::from_translation(pos.extend(0.0))),
            ActiveEvents::COLLISION_EVENTS,
            stats,
        ));
    }
}
