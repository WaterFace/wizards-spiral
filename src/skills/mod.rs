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
            .add_systems(
                Update,
                    send_xp_events,
                    process_xp_events,
                )
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
fn process_xp_events(
    mut player_skills: ResMut<PlayerSkills>,
    mut events: EventReader<SkillXpEvent>,
) {
    for SkillXpEvent { skill, xp } in events.read() {
        player_skills.add_xp(*skill, *xp);
    }
}

fn send_xp_events(
    mut writer: EventWriter<SkillXpEvent>,
    mut damage_events: EventReader<crate::damage::DamageEvent>,
    mut melee_attack_events: EventReader<crate::damage::MeleeAttackEvent>,
) {
    // Damage events / Armor skill
    for ev in damage_events.read() {
        match ev {
            crate::damage::DamageEvent::Player { .. } => {
                writer.send(SkillXpEvent {
                    skill: Skill::Armor,
                    xp: 1.0,
                });
            }
            _ => {}
        }
    }

    // Melee attack events / Sword skill
    for crate::damage::MeleeAttackEvent { .. } in melee_attack_events.read() {
        writer.send(SkillXpEvent {
            skill: Skill::Sword,
            xp: 1.0,
        });
    }
}
