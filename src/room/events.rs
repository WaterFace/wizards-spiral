use bevy::prelude::*;

#[derive(Debug, Event, Clone)]
pub struct ChangeRoom {
    pub next_room_name: String,
}

pub fn start_game(mut change_room: EventWriter<ChangeRoom>) {
    change_room.send(ChangeRoom {
        next_room_name: "Lovely Cottage".to_string(),
    });
}

pub fn handle_change_room(
    mut commands: Commands,
    mut reader: EventReader<ChangeRoom>,
    rooms: Res<super::Rooms>,
    enemy_stats: Res<Assets<crate::enemy::EnemyStats>>,
    mut next_state: ResMut<NextState<crate::states::GameState>>,
) {
    // Only take the first event per frame, dropping the rest
    let Some(ChangeRoom { next_room_name }) = reader.read().next() else {
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

    commands.insert_resource(crate::room::CurrentRoom {
        info: info.clone(),
        assets: assets.clone(),
        melee_enemy_stats: melee_enemy_stats.clone(),
        ranged_enemy_stats: ranged_enemy_stats.clone(),
    });

    next_state.set(crate::states::GameState::RoomTransition);
}
