use bevy::prelude::*;

#[derive(Debug, Default)]
pub struct CyclePlugin;

impl Plugin for CyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(crate::states::GameState::RestartCycle),
            (
                reset_init_global_state,
                crate::save_data::save_data,
                start_cycle,
            )
                .chain(),
        );
    }
}

#[derive(Debug, Default, Resource)]
pub struct CycleCounter {
    pub count: u64,
}

fn start_cycle(mut change_room: EventWriter<crate::room::ChangeRoom>) {
    change_room.send(crate::room::ChangeRoom {
        next_room_name: "Lovely Cottage".into(),
        ..Default::default()
    });
}

/// reset or initialize various global state
fn reset_init_global_state(
    mut commands: Commands,
    cycle_counter: Option<Res<CycleCounter>>,
    player_skills: Option<ResMut<crate::skills::PlayerSkills>>,
    persistent_room_state: Option<ResMut<crate::room::PersistentRoomState>>,
) {
    // initialize the cycle counter if necessary
    if cycle_counter.is_none() {
        commands.init_resource::<CycleCounter>();
    }

    // merge the previous cycle's progress into the persistent storage
    if let Some(mut player_skills) = player_skills {
        player_skills.end_cycle();
    } else {
        commands.init_resource::<crate::skills::PlayerSkills>();
    }

    // remove the cached spawn data so rooms will spawn freshly
    if let Some(mut persistent_room_state) = persistent_room_state {
        persistent_room_state.rooms.clear();
    } else {
        commands.init_resource::<crate::room::PersistentRoomState>();
    }

    // reset/initialize player health
    commands.init_resource::<crate::player::PlayerHealth>();
}
