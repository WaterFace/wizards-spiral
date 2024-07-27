use bevy::prelude::*;
use bevy_pkv::{GetError, PkvStore};

#[derive(Debug, Default)]
pub struct SaveDataPlugin;

impl Plugin for SaveDataPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PkvStore::new("Water Face", "Wizard's Spiral"))
            .add_systems(
                OnTransition {
                    entered: crate::states::AppState::InMenu,
                    exited: crate::states::AppState::RoomLoading,
                },
                load_data,
            );
    }
}

fn load_data(mut commands: Commands, pkv_store: Res<PkvStore>) {
    let save_string = match pkv_store.get::<String>("save_data") {
        Ok(string) => {
            info!("save_data: successfully loaded save string");
            string
        }
        Err(GetError::NotFound) => {
            info!("save_data: no save data found");
            return;
        }
        Err(e) => {
            warn!("save_data: failed to load save string: {e}");
            return;
        }
    };

    let save_data = match ron::de::from_str::<SaveData>(&save_string) {
        Ok(data) => {
            info!("save_data: successfully parsed save data");
            data
        }
        Err(e) => {
            error!("save_data: failed to parse save data: {e}");
            return;
        }
    };

    let (player_skills, cycle_counter) = save_data.to_resources();

    commands.insert_resource(player_skills);
    commands.insert_resource(cycle_counter);
}

pub fn save_data(
    mut pkv_store: ResMut<PkvStore>,
    player_skills: Res<crate::skills::PlayerSkills>,
    cycle_counter: Res<crate::cycles::CycleCounter>,
) {
    let save_data = SaveData::from_resources(&player_skills, &cycle_counter);

    let ron_string = match ron::ser::to_string(&save_data) {
        Ok(string) => {
            info!("save_data: successfully encoded save data");
            string
        }
        Err(e) => {
            panic!("Failed to encode save file as RON: {e}!")
        }
    };

    match pkv_store.set_string("save_data", &ron_string) {
        Ok(()) => {
            info!("save_data: successfully saved save data");
        }
        Err(e) => {
            error!("save_data: failed to save save data: {e}");
            // TODO: panic?
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SaveData {
    pub cycles: u64,

    pub armor_level: u64,
    pub armor_xp: f32,

    pub sword_level: u64,
    pub sword_xp: f32,

    pub shield_level: u64,
    pub shield_xp: f32,

    pub pants_level: u64,
    pub pants_xp: f32,

    pub mirror_level: u64,
    pub mirror_xp: f32,

    pub healing_level: u64,
    pub healing_xp: f32,

    pub speed_level: u64,
    pub speed_xp: f32,
}

impl SaveData {
    /// extracts the relevant data from the game state.
    ///
    /// remember to call PlayerSkills::end_cycle before this
    pub fn from_resources(
        player_skills: &crate::skills::PlayerSkills,
        cycle_counter: &crate::cycles::CycleCounter,
    ) -> Self {
        Self {
            cycles: cycle_counter.count,

            armor_level: player_skills.get(crate::skills::Skill::Armor),
            armor_xp: player_skills.get_xp(crate::skills::Skill::Armor),

            sword_level: player_skills.get(crate::skills::Skill::Sword),
            sword_xp: player_skills.get_xp(crate::skills::Skill::Sword),

            shield_level: player_skills.get(crate::skills::Skill::Shield),
            shield_xp: player_skills.get_xp(crate::skills::Skill::Shield),

            pants_level: player_skills.get(crate::skills::Skill::Pants),
            pants_xp: player_skills.get_xp(crate::skills::Skill::Pants),

            mirror_level: player_skills.get(crate::skills::Skill::Mirror),
            mirror_xp: player_skills.get_xp(crate::skills::Skill::Mirror),

            healing_level: player_skills.get(crate::skills::Skill::Healing),
            healing_xp: player_skills.get_xp(crate::skills::Skill::Healing),

            speed_level: player_skills.get(crate::skills::Skill::Speed),
            speed_xp: player_skills.get_xp(crate::skills::Skill::Speed),
        }
    }

    pub fn to_resources(&self) -> (crate::skills::PlayerSkills, crate::cycles::CycleCounter) {
        (
            crate::skills::PlayerSkills::from_save_data(self),
            crate::cycles::CycleCounter { count: self.cycles },
        )
    }
}
