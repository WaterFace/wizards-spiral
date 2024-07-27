use bevy::prelude::*;
use bevy_math::vec2;

#[derive(Debug, Default, Event, Clone)]
pub struct ChangeRoom {
    /// Which room to go to
    pub next_room_name: String,
    /// Where we're entering from, from the perspective of the new room
    pub coming_from: Option<super::CardinalDirection>,
}

pub fn handle_change_room(
    mut commands: Commands,
    mut reader: EventReader<ChangeRoom>,
    rooms: Res<super::Rooms>,
    enemy_stats: Res<Assets<crate::enemy::EnemyStats>>,
    boss_stats: Res<Assets<crate::enemy::BossStats>>,
    mut next_state: ResMut<NextState<crate::states::GameState>>,
) {
    // Only take the first event per frame, dropping the rest
    let Some(ChangeRoom {
        next_room_name,
        coming_from,
    }) = reader.read().next()
    else {
        return;
    };

    info!("Preparing to change rooms to {}", next_room_name);

    let Some((info, assets)) = rooms.map.get(next_room_name) else {
        error!("Room `{}` not loaded!", next_room_name);
        panic!();
    };

    let Some(melee_enemy_stats) = enemy_stats.get(&assets.melee_enemy_stats) else {
        error!(
            "No EnemyStats found with handle {:?}",
            assets.melee_enemy_stats
        );
        panic!();
    };
    let Some(ranged_enemy_stats) = enemy_stats.get(&assets.ranged_enemy_stats) else {
        error!(
            "No EnemyStats found with handle {:?}",
            assets.ranged_enemy_stats
        );
        panic!();
    };

    let boss_stats = match &assets.boss_stats {
        None => {
            // no boss in this room
            None
        }
        Some(handle) => {
            let Some(boss_stats) = boss_stats.get(handle) else {
                error!("No BossStats found with handle {:?}", handle);
                panic!();
            };
            Some(boss_stats.clone())
        }
    };

    commands.insert_resource(crate::room::CurrentRoom {
        info: info.clone(),
        assets: assets.clone(),
        boss_stats,
        melee_enemy_stats: melee_enemy_stats.clone(),
        ranged_enemy_stats: ranged_enemy_stats.clone(),
    });

    const DISTANCE_FROM_EXIT: f32 = 50.0;
    let pos;
    let room_rect = info.rect;
    let half_size = room_rect.half_size();
    match coming_from {
        None => {
            pos = room_rect.center();
        }
        Some(crate::room::CardinalDirection::North) => {
            pos = room_rect.center() + vec2(0.0, half_size.y - DISTANCE_FROM_EXIT);
        }
        Some(crate::room::CardinalDirection::South) => {
            pos = room_rect.center() - vec2(0.0, half_size.y - DISTANCE_FROM_EXIT);
        }

        Some(crate::room::CardinalDirection::East) => {
            pos = room_rect.center() + vec2(half_size.x - DISTANCE_FROM_EXIT, 0.0);
        }
        Some(crate::room::CardinalDirection::West) => {
            pos = room_rect.center() - vec2(half_size.x - DISTANCE_FROM_EXIT, 0.0);
        }
    }
    commands.insert_resource(crate::player::PlayerSpawnPosition { pos });

    next_state.set(crate::states::GameState::RoomTransition);
}
