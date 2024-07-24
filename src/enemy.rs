use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Debug, Default)]
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EnemyAlertEvent>().add_systems(
            Update,
            (
                move_enemies,
                collisions_to_alert_events,
                process_alert_events,
            )
                .run_if(in_state(crate::states::GameState::InGame)),
        );
    }
}

#[derive(Debug, Component)]
pub struct EnemyStats {
    pub enemy_type: EnemyType,
    pub health: f32,
    pub alert_radius: f32,
    pub chase_radius: f32,
}

#[derive(Debug, Default, Component, Clone, Copy)]
pub enum EnemyType {
    #[default]
    Melee,
    Ranged,
}

#[derive(Debug, Default, Component, Clone, Copy)]
pub struct Enemy;

#[derive(Debug, Default, Component)]
pub enum EnemyState {
    #[default]
    Wander,
    Chase,
}

#[derive(Debug, Component)]
struct AlertSensor;

#[derive(Debug, Component)]
struct AttachedTo(Entity);

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
        match (enemy_state, stats.enemy_type) {
            (EnemyState::Wander, _) => {
                // TODO: implement wandering
                controller.desired_direction = Vec2::ZERO;
            }
            (EnemyState::Chase, EnemyType::Melee) => {
                let enemy_pos = transform.translation().truncate();
                let Some(player_pos) = player_pos else {
                    // no player position, so no need to move
                    controller.desired_direction = Vec2::ZERO;
                    continue;
                };
                let dir = (player_pos - enemy_pos).normalize_or_zero();

                controller.desired_direction = dir;
            }
            (EnemyState::Chase, EnemyType::Ranged) => {
                // TODO: implement ranged enemies

                // They should try to stay a (configurable) distance from the player
                controller.desired_direction = Vec2::ZERO;
            }
        }
    }
}

pub fn spawn_melee_enemies(In(to_spawn): In<Vec<(Vec2, EnemyStats)>>, mut commands: Commands) {
    for (pos, stats) in to_spawn {
        let enemy_id = commands.spawn_empty().id();

        let alert_sensor = create_alert_sensor(&stats);
        let alert_sensor_id = commands.spawn((alert_sensor, AttachedTo(enemy_id))).id();

        commands
            .entity(enemy_id)
            .insert((
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
                TransformBundle::from_transform(Transform::from_translation(pos.extend(0.0))),
                ActiveEvents::COLLISION_EVENTS,
                stats,
            ))
            .add_child(alert_sensor_id);
    }
}

fn process_alert_events(
    mut alert_events: EventReader<EnemyAlertEvent>,
    mut enemy_query: Query<&mut EnemyState, With<Enemy>>,
) {
    for ev in alert_events.read() {
        let EnemyAlertEvent { enemy, ty } = ev;
        let Ok(mut enemy_state) = enemy_query.get_mut(*enemy) else {
            warn!("Recieved an EnemyAlertEvent regarding an entity ({enemy:?}) without `Enemy` and `EnemyState` components");
            continue;
        };
        match ty {
            EnemyAlertEventType::Alert => *enemy_state = EnemyState::Chase,
            EnemyAlertEventType::TooFar => *enemy_state = EnemyState::Wander,
        }
    }
}

fn collisions_to_alert_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut alert_writer: EventWriter<EnemyAlertEvent>,
    alert_sensor_query: Query<&AttachedTo, With<AlertSensor>>,
    player_query: Query<Entity, With<crate::player::Player>>,
) {
    for ev in collision_events.read() {
        // we only care about the `Started` events at this point
        let CollisionEvent::Started(e1, e2, _flags) = ev else {
            continue;
        };

        let Ok(alert_sensor_attached_to) =
            alert_sensor_query.get(*e1).or(alert_sensor_query.get(*e2))
        else {
            // Neither entity was an alert sensor
            continue;
        };

        let Ok(_player_entity) = player_query.get(*e1).or(player_query.get(*e2)) else {
            // Neither entity was the player
            continue;
        };

        info!(
            "Player({:?}) entered Enemy({:?})'s alert sensor",
            _player_entity, alert_sensor_attached_to.0
        );
        alert_writer.send(EnemyAlertEvent {
            enemy: alert_sensor_attached_to.0,
            ty: EnemyAlertEventType::Alert,
        });
    }
}

fn create_alert_sensor(stats: &EnemyStats) -> impl Bundle {
    (
        Collider::ball(stats.alert_radius),
        ColliderMassProperties::Density(0.0),
        Sensor,
        AlertSensor,
    )
}
