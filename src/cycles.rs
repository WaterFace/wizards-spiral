use bevy::prelude::*;

#[derive(Debug, Default)]
pub struct CyclePlugin;

impl Plugin for CyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnTransition {
                exited: crate::states::GameState::MainMenu,
                entered: crate::states::GameState::RestartCycle,
            },
            start_game,
        )
        .add_systems(
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

fn start_game(
    mut commands: Commands,
    save_data: Option<Res<crate::save_data::SaveData>>,
    new_game: Option<Res<crate::menus::NewGame>>,
) {
    let Some(save_data) = save_data else {
        info!("start_game: No save data present");
        return;
    };

    if let Some(_new_game) = new_game {
        info!("start_game: ignoring existing save data. starting new game");
        return;
    }

    let (player_skills, cycle_counter) = save_data.to_resources();
    commands.insert_resource(player_skills);
    commands.insert_resource(cycle_counter);
}

fn start_cycle(
    mut change_room: EventWriter<crate::room::ChangeRoom>,
    mut unlock_skill: EventWriter<crate::skills::SkillUnlockedEvent>,
) {
    unlock_skill.send(crate::skills::SkillUnlockedEvent {
        skill: crate::skills::Skill::Armor,
    });
    change_room.send(crate::room::ChangeRoom {
        next_room_name: "Lovely Cottage".into(),
        ..Default::default()
    });
}

/// reset or initialize various global state
fn reset_init_global_state(
    mut commands: Commands,
    cycle_counter: Option<ResMut<CycleCounter>>,
    player_skills: Option<ResMut<crate::skills::PlayerSkills>>,
    persistent_room_state: Option<ResMut<crate::room::PersistentRoomState>>,
) {
    // initialize the cycle counter if necessary
    if cycle_counter.is_none() {
        commands.insert_resource(CycleCounter::default());
    }

    // merge the previous cycle's progress into the persistent storage
    if let Some(mut player_skills) = player_skills {
        player_skills.end_cycle();
    } else {
        commands.insert_resource(crate::skills::PlayerSkills::default());
    }

    // remove the cached spawn data so rooms will spawn freshly
    if let Some(mut persistent_room_state) = persistent_room_state {
        persistent_room_state.rooms.clear();
    } else {
        commands.insert_resource(crate::room::PersistentRoomState::default());
    }

    // reset/initialize player health
    commands.insert_resource(crate::player::PlayerHealth::default());
}
