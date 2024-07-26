use bevy::prelude::*;

mod player_skills;
pub use player_skills::{PlayerSkills, Skill};

#[derive(Debug, Default)]
pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LevelUpEvent>()
            .add_event::<SkillUnlockedEvent>()
            .add_event::<SkillXpEvent>()
            //  TEMPORARY!!!
            .init_resource::<PlayerSkills>()
            .add_systems(
                Update,
                (send_levelup_events, process_unlock_events)
                    .run_if(in_state(crate::states::GameState::InGame)),
            );
    }
}

#[derive(Debug, Event, Clone)]
pub struct SkillXpEvent {
    pub skill: Skill,
    pub xp: f32,
}

#[derive(Debug, Event, Clone)]
pub struct LevelUpEvent {
    pub num_levels: u64,
    pub skill: Skill,
}

#[derive(Debug, Event, Clone)]
pub struct SkillUnlockedEvent {
    pub skill: Skill,
}

fn send_levelup_events(
    mut player_skills: ResMut<PlayerSkills>,
    mut writer: EventWriter<LevelUpEvent>,
    mut buffer: Local<Vec<LevelUpEvent>>,
) {
    player_skills.drain_levelups(&mut buffer);
    writer.send_batch(buffer.drain(..));
}

fn process_unlock_events(
    mut player_skills: ResMut<PlayerSkills>,
    mut reader: EventReader<SkillUnlockedEvent>,
) {
    for SkillUnlockedEvent { skill } in reader.read() {
        player_skills.unlock_skill(*skill);
    }
}

fn send_xp_events(mut writer: EventWriter<SkillXpEvent>) {}
