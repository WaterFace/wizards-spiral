use bevy::prelude::*;
use bevy_math::vec2;
use bevy_rapier2d::prelude::*;
use rand::{prelude::Distribution, Rng};

pub fn spawn_enemies(
    mut commands: Commands,
    room_state: Res<super::PersistentRoomState>,
    query: Query<(&super::Spawner, &Transform)>,
    current_room: Res<super::CurrentRoom>,
    mut next_state: ResMut<NextState<crate::states::GameState>>,
) {
    for (spawner, transform) in query.iter() {
        if !spawner.active {
            continue;
        }

        if let Some(room_state) = room_state.rooms.get(&current_room.info.name) {
            let Some(spawner) = room_state.spawners.get(spawner.index) else {
                error!("spawn_enemies: {:?} present in PersistentRoomState map, but list doesn't contain index {:?}", current_room.info.name, spawner.index);
                continue;
            };

            if !spawner.active {
                // This spawner's enemy has already been killed this cycle
                continue;
            }
        }

        info!("spawning enemy at {:?}", transform.translation);

        let (texture, stats) = match spawner.ty {
            super::SpawnerType::Melee => (
                current_room.assets.melee_enemy_texture.clone(),
                current_room.melee_enemy_stats.clone(),
            ),
            super::SpawnerType::Ranged => (
                current_room.assets.ranged_enemy_texture.clone(),
                current_room.ranged_enemy_stats.clone(),
            ),
        };

        commands
            .spawn(SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(vec2(32.0, 32.0)),
                    ..Default::default()
                },
                texture,
                ..Default::default()
            })
            .insert((
                // TODO: make movement speed, etc. configurable
                crate::character_controller::CharacterController {
                    acceleration: 10.0,
                    max_speed: 64.0,
                    ..Default::default()
                },
                crate::enemy::Enemy,
                crate::enemy::EnemyState::default(),
                stats.clone(),
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
                ActiveEvents::COLLISION_EVENTS,
                transform.clone(),
                super::SpawnerIndex(spawner.index),
                // So the enemy will be despawned when we change room
                crate::room::RoomObject,
            ));
    }

    next_state.set(crate::states::GameState::InGame);
}

pub fn destroy_room(mut commands: Commands, query: Query<Entity, With<super::RoomObject>>) {
    // TODO: keep track of dead enemies and flag their spawners as inactive in the `PersistentRoomState` cache
    let mut count = 0;
    for e in query.iter() {
        commands.entity(e).despawn_recursive();
        count += 1;
    }
    info!("destroy_room: despawned {count} entities");
}

pub fn spawn_room(
    mut commands: Commands,
    current_room: Res<super::CurrentRoom>,
    mut room_state: ResMut<super::PersistentRoomState>,
    mut rng: ResMut<crate::rand::GlobalRng>,
    mut working: Local<Vec<Vec2>>,
) {
    // Floor
    commands.spawn((
        super::FloorBundle {
            texture: current_room.assets.background_texture.clone(),
            sprite: Sprite {
                custom_size: Some(current_room.info.rect.size()),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, -10.0),
            ..Default::default()
        },
        ImageScaleMode::Tiled {
            tile_x: true,
            tile_y: true,
            stretch_value: 1.0,
        },
    ));

    // Walls
    const WALL_THICKNESS: f32 = 100.0;

    let room_rect = current_room.info.rect;
    let top_left = room_rect.min;
    let top_right = vec2(room_rect.max.x, room_rect.min.y);
    let bottom_left = vec2(room_rect.min.x, room_rect.max.y);
    let bottom_right = room_rect.max;

    let wall_rects = [
        // Left wall
        Rect::from_corners(
            top_left - vec2(WALL_THICKNESS, WALL_THICKNESS),
            bottom_left + vec2(0.0, WALL_THICKNESS),
        ),
        // Top wall
        Rect::from_corners(
            bottom_left - vec2(WALL_THICKNESS, 0.0),
            bottom_right + vec2(WALL_THICKNESS, WALL_THICKNESS),
        ),
        // Right wall
        Rect::from_corners(
            top_right - vec2(0.0, WALL_THICKNESS),
            bottom_right + vec2(WALL_THICKNESS, WALL_THICKNESS),
        ),
        // Bottom wall
        Rect::from_corners(
            top_left - vec2(WALL_THICKNESS, WALL_THICKNESS),
            top_right + vec2(WALL_THICKNESS, 0.0),
        ),
    ];
    for (i, rect) in wall_rects.into_iter().enumerate() {
        let wall = match i {
            0 => crate::room::Wall(crate::room::CardinalDirection::West),
            1 => crate::room::Wall(crate::room::CardinalDirection::North),
            2 => crate::room::Wall(crate::room::CardinalDirection::East),
            3 => crate::room::Wall(crate::room::CardinalDirection::South),
            _ => unreachable!(),
        };
        commands.spawn((
            TransformBundle {
                local: Transform::from_translation(rect.center().extend(0.0)),
                ..Default::default()
            },
            Collider::cuboid(rect.half_size().x, rect.half_size().y),
            wall,
            crate::room::RoomObject,
        ));
    }

    if let Some(room_state) = room_state.rooms.get(&current_room.info.name) {
        info!(
            "{} has been previously visited this cycle, spawning according to cached data",
            current_room.info.name
        );
        // room state found, spawn things according to the cached data
        for (index, spawner_state) in room_state.spawners.iter().enumerate() {
            commands.spawn(super::SpawnerBundle {
                transform: Transform::from_translation(spawner_state.position.extend(0.0)),
                spawner: super::Spawner {
                    index,
                    ty: spawner_state.ty,
                    active: spawner_state.active,
                },
                ..Default::default()
            });
        }
        for obstacle_state in room_state.obstacles.iter() {
            commands.spawn(super::ObstacleBundle {
                texture: current_room.assets.obstacle_texture.clone(),
                sprite: Sprite {
                    custom_size: Some(vec2(32.0, 64.0)),
                    ..Default::default()
                },
                transform: Transform::from_translation(obstacle_state.position.extend(0.0)),
                collider: Collider::capsule_y(12.0, 12.0),
                ..Default::default()
            });
        }
    } else {
        info!(
            "{} hasn't been visited this cycle, spawning new entities",
            current_room.info.name
        );
        // room state not found, spawn things freshly and cache the data
        let spawning_rectangle = {
            let room_rect = current_room.info.rect;
            // don't spawn things on the outer 10% of the room
            Rectangle {
                half_size: room_rect.half_size() * 0.95,
            }
        };

        let mut this_room_state = super::RoomState::default();

        // spawners:
        working.clear();
        working.extend(
            spawning_rectangle
                .interior_dist()
                .sample_iter(rng.as_deref_mut().as_mut())
                .take(current_room.info.num_melee_enemies + current_room.info.num_ranged_enemies),
        );
        for (index, pos) in working.drain(..).enumerate() {
            let ty = if index < current_room.info.num_melee_enemies {
                super::SpawnerType::Melee
            } else {
                super::SpawnerType::Ranged
            };

            info!("Placing spawner {index} at {pos:?}");

            commands.spawn(super::SpawnerBundle {
                transform: Transform::from_translation(pos.extend(0.0)),
                spawner: super::Spawner {
                    index,
                    ty,
                    active: true,
                },
                ..Default::default()
            });
            this_room_state.spawners.push(super::SpawnerState {
                active: true,
                position: pos,
                ty,
            });
        }

        // obstacles:
        working.clear();
        working.extend(
            spawning_rectangle
                .interior_dist()
                .sample_iter(rng.as_deref_mut().as_mut())
                .take(current_room.info.num_obstacles),
        );
        for pos in working.drain(..) {
            commands.spawn(super::ObstacleBundle {
                texture: current_room.assets.obstacle_texture.clone(),
                sprite: Sprite {
                    custom_size: Some(vec2(32.0, 64.0)),
                    ..Default::default()
                },
                transform: Transform::from_translation(pos.extend(0.0)),
                collider: Collider::capsule_y(12.0, 12.0),
                ..Default::default()
            });

            this_room_state
                .obstacles
                .push(super::ObstacleState { position: pos });
        }

        let None = room_state
            .rooms
            .insert(current_room.info.name.clone(), this_room_state)
        else {
            panic!(
                "room_state map already has entry for {}, but we checked that it didn't!",
                current_room.info.name
            );
        };
    }
}
