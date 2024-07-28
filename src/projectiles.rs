use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

#[derive(Debug, Default)]
pub struct ProjectilesPlugin;

impl Plugin for ProjectilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ProjectileHitEvent>()
            .add_event::<ProjectileReflectEvent>()
            .add_systems(
                Update,
                (
                    add_projectile_launcher_state,
                    launch_projectiles,
                    update_projectiles,
                    detect_projectile_hits,
                    handle_projectile_hits,
                    spawn_reflected_projectiles,
                )
                    .run_if(in_state(crate::states::GameState::InGame)),
            );
    }
}

#[derive(Debug, Clone, Component)]
pub struct Projectile {
    pub target: Entity,
    pub source: Entity,
    pub speed: f32,
    pub damage: f32,
    pub homing: bool,
    pub timer: Timer,
}

#[derive(Debug, Clone, Event)]
pub struct ProjectileHitEvent {
    pub projectile: Projectile,
    pub target: Entity,
}

#[derive(Debug, Clone, Event)]
pub struct ProjectileReflectEvent {
    pub projectile: Projectile,
}

#[derive(Debug, Component)]
struct ProjectileLauncherState {
    timer: Timer,
    delay: f32,
    projectile_speed: f32,
    projectile_damage: f32,
    projectile_lifetime: f32,
    homing: bool,
}

fn spawn_reflected_projectiles(
    mut commands: Commands,
    player_query: Query<&GlobalTransform, With<crate::player::Player>>,
    enemy_query: Query<&GlobalTransform>,
    player_skills: Res<crate::skills::PlayerSkills>,
    mut events: EventReader<ProjectileReflectEvent>,
    current_room: Res<crate::room::CurrentRoom>,
    mut rng: ResMut<crate::rand::GlobalRng>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        error!("spawn_reflected_projectiles: couldn't get single player");
        return;
    };
    let player_pos = player_transform.translation().truncate();
    for ProjectileReflectEvent { projectile } in events.read() {
        let dir = {
            if let Ok(enemy_transform) = enemy_query.get(projectile.source) {
                let enemy_pos = enemy_transform.translation().truncate();
                (enemy_pos - player_pos).normalize_or(Vec2::X)
            } else {
                // the projectile's source is already gone, just fire in a random direction
                Dir2::from_rng(rng.as_mut()).into()
            }
        };
        let initial_angle = Vec2::X.angle_between(dir);
        let mut new_timer = projectile.timer.clone();
        new_timer.reset();
        commands.spawn((
            SpriteBundle {
                texture: current_room.assets.projectile.clone(),
                transform: Transform::from_translation(player_pos.extend(-1.0))
                    .with_rotation(Quat::from_rotation_z(initial_angle)),
                ..Default::default()
            },
            Projectile {
                source: projectile.target,
                target: projectile.source,
                damage: player_skills.attack_damage(),
                timer: new_timer,
                ..projectile.clone()
            },
            RigidBody::KinematicVelocityBased,
            Collider::ball(8.0),
            Sensor,
            CollisionGroups::new(
                crate::physics::COLLISION_GROUP_REFLECTED_PROJECTILE,
                crate::physics::COLLISION_GROUP_ENEMY,
            ),
            Velocity::linear(dir * projectile.speed),
            crate::room::RoomObject,
            Name::new("Reflected Projectile"),
        ));
    }
}

fn handle_projectile_hits(
    mut events: EventReader<ProjectileHitEvent>,
    player_query: Query<Entity, With<crate::player::Player>>,
    player_skills: Res<crate::skills::PlayerSkills>,
    // enemy_query: Query<Entity, With<crate::enemy::Enemy>>,
    mut damage_events: EventWriter<crate::damage::DamageEvent>,
    mut reflect_events: EventWriter<ProjectileReflectEvent>,
    mut rng: ResMut<crate::rand::GlobalRng>,
) {
    for ProjectileHitEvent { projectile, target } in events.read() {
        if player_query.contains(*target) {
            if rng.as_mut().gen_bool(player_skills.reflect_chance() as f64) {
                reflect_events.send(ProjectileReflectEvent {
                    projectile: projectile.clone(),
                });
                continue;
            }
            damage_events.send(crate::damage::DamageEvent::Player {
                damage: projectile.damage,
            });
            continue;
        }

        damage_events.send(crate::damage::DamageEvent::Enemy {
            entity: *target,
            damage: projectile.damage,
        });
    }
}

fn detect_projectile_hits(
    mut commands: Commands,
    mut collisions: EventReader<CollisionEvent>,
    target_query: Query<Entity, Or<(With<crate::player::Player>, With<crate::enemy::Enemy>)>>,
    projectile_query: Query<(Entity, &Projectile)>,
    mut writer: EventWriter<ProjectileHitEvent>,
) {
    for ev in collisions.read() {
        let CollisionEvent::Started(e1, e2, _flags) = ev else {
            // we only care about the `Started` events here
            continue;
        };

        let Ok(target) = target_query.get(*e1).or(target_query.get(*e2)) else {
            continue;
        };

        let Ok((projectile_entity, projectile)) =
            projectile_query.get(*e1).or(projectile_query.get(*e2))
        else {
            continue;
        };

        commands.entity(projectile_entity).despawn_recursive();
        writer.send(ProjectileHitEvent {
            projectile: projectile.clone(),
            target,
        });
    }
}

fn update_projectiles(
    mut commands: Commands,
    mut projectile_query: Query<(
        Entity,
        &GlobalTransform,
        &mut Transform,
        &mut Velocity,
        &mut Projectile,
    )>,
    target_query: Query<
        &GlobalTransform,
        Or<(With<crate::player::Player>, With<crate::enemy::Enemy>)>,
    >,
    time: Res<Time>,
) {
    for (entity, global_transform, mut local_transform, mut velocity, mut projectile) in
        projectile_query.iter_mut()
    {
        projectile.timer.tick(time.delta());
        if projectile.timer.finished() {
            commands.entity(entity).despawn_recursive();
            continue;
        }
        if !projectile.homing {
            // nothing to do
            continue;
        }
        let Ok(target_transform) = target_query.get(projectile.target) else {
            // it's a homing projectile but its target is gone, just go straight
            continue;
        };

        let target_pos = target_transform.translation().truncate();
        let current_pos = global_transform.translation().truncate();

        let dir = (target_pos - current_pos).normalize_or(Vec2::X);
        let angle = Vec2::X.angle_between(dir);
        local_transform.rotation = Quat::from_rotation_z(angle);
        velocity.linvel = dir * projectile.speed;
    }
}

fn launch_projectiles(
    mut commands: Commands,
    player_query: Query<(Entity, &GlobalTransform), With<crate::player::Player>>,
    mut enemy_query: Query<(
        Entity,
        &GlobalTransform,
        &crate::enemy::EnemyState,
        &mut ProjectileLauncherState,
    )>,
    current_room: Res<crate::room::CurrentRoom>,
    time: Res<Time>,
) {
    let Ok((player_entity, player_global_transform)) = player_query.get_single() else {
        warn!("launch_projectiles: couldn't get single player");
        return;
    };
    let player_pos = player_global_transform.translation().truncate();

    for (enemy_entity, global_transform, enemy_state, mut projectile_launcher_state) in
        enemy_query.iter_mut()
    {
        match enemy_state {
            crate::enemy::EnemyState::Wander => {
                // reset the delay on the launcher, that's it
                projectile_launcher_state.timer.reset();
                let delay = projectile_launcher_state.delay;
                projectile_launcher_state
                    .timer
                    .set_duration(std::time::Duration::from_secs_f32(delay));
                continue;
            }
            &crate::enemy::EnemyState::Chase => {
                // go on...
            }
        }

        projectile_launcher_state.timer.tick(time.delta());

        if !projectile_launcher_state.timer.finished() {
            continue;
        }

        projectile_launcher_state.timer.reset();

        let enemy_pos = global_transform.translation().truncate();
        let dir = (player_pos - enemy_pos).normalize_or(Vec2::X);
        let initial_angle = Vec2::X.angle_between(dir);
        commands.spawn((
            SpriteBundle {
                texture: current_room.assets.projectile.clone(),
                transform: Transform::from_translation(enemy_pos.extend(-1.0))
                    .with_rotation(Quat::from_rotation_z(initial_angle)),
                ..Default::default()
            },
            Projectile {
                source: enemy_entity,
                target: player_entity,
                speed: projectile_launcher_state.projectile_speed,
                damage: projectile_launcher_state.projectile_damage,
                homing: projectile_launcher_state.homing,
                timer: Timer::from_seconds(
                    projectile_launcher_state.projectile_lifetime,
                    TimerMode::Once,
                ),
            },
            RigidBody::KinematicVelocityBased,
            Collider::ball(8.0),
            Sensor,
            CollisionGroups::new(
                crate::physics::COLLISION_GROUP_PROJECTILE,
                crate::physics::COLLISION_GROUP_PLAYER,
            ),
            Velocity::linear(dir * projectile_launcher_state.projectile_speed),
            crate::room::RoomObject,
            Name::new("Projectile"),
        ));
    }
}

fn add_projectile_launcher_state(
    mut commands: Commands,
    query: Query<(Entity, &crate::enemy::EnemyStats), Added<crate::enemy::EnemyStats>>,
) {
    for (entity, enemy_stats) in query.iter() {
        let crate::enemy::EnemyType::Ranged {
            projectile_damage,
            projectile_speed,
            projectile_lifetime,
            homing,
            delay,
            ..
        } = enemy_stats.enemy_type
        else {
            continue;
        };

        let projectile_launcher_state = ProjectileLauncherState {
            delay,
            projectile_damage,
            projectile_speed,
            projectile_lifetime,
            homing,
            timer: Timer::from_seconds(delay, TimerMode::Once),
        };
        commands.entity(entity).insert(projectile_launcher_state);
    }
}
