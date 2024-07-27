use bevy::prelude::*;

mod player_skills;
pub use player_skills::{PlayerSkills, Skill};

#[derive(Debug, Default)]
pub struct SkillsPlugin;

impl Plugin for SkillsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LevelUpEvent>()
            // setup unlocked events to be manually cleared so they don't get lost
            .init_resource::<Events<SkillUnlockedEvent>>()
            .add_event::<SkillXpEvent>()
            .add_systems(
                Update,
                (
                    send_levelup_events,
                    process_unlock_events,
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
    mut events: ResMut<Events<SkillUnlockedEvent>>,
) {
    // do it this way so we get all such events, regardless of when this runs vs when they're sent
    for SkillUnlockedEvent { skill } in events.drain() {
        if !player_skills.get_unlocked(skill) {
            info!("unlocked skill: {}", skill);
        }
        player_skills.unlock_skill(skill);
    }
}

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
    mut projectile_reflected_event: EventReader<crate::projectiles::ProjectileReflectEvent>,
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
    // Projectile reflected / Mirror skill
    for crate::projectiles::ProjectileReflectEvent { .. } in projectile_reflected_event.read() {
        writer.send(SkillXpEvent {
            skill: Skill::Mirror,
            xp: 1.0,
        });
    }
}
