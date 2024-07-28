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
            .add_event::<SkillFirstUnlockedEvent>()
            .add_event::<HealEvent>()
            .add_systems(
                Update,
                (
                    send_levelup_events,
                    process_unlock_events,
                    send_xp_events,
                    process_xp_events,
                    speed_xp,
                    healing,
                    update_player_speed,
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

#[derive(Debug, Event, Clone)]
pub struct SkillFirstUnlockedEvent {
    pub skill: Skill,
}

#[derive(Debug, Default, Event, Clone)]
pub struct HealEvent;

#[derive(Debug, Clone, Resource)]
pub struct HealTimer(Timer);

impl HealTimer {
    pub fn new() -> Self {
        HealTimer(Timer::from_seconds(3.0, TimerMode::Repeating))
    }

    pub fn tick(&mut self, delta: std::time::Duration) {
        self.0.tick(delta);
    }

    pub fn reset(&mut self) {
        self.0.reset();
    }
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
    mut first_unlock_event: EventWriter<SkillFirstUnlockedEvent>,
) {
    // do it this way so we get all such events, regardless of when this runs vs when they're sent
    for SkillUnlockedEvent { skill } in events.drain() {
        if !player_skills.get_unlocked(skill) {
            info!("unlocked skill: {}", skill);
            first_unlock_event.send(SkillFirstUnlockedEvent { skill });
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

#[derive(Debug, Resource)]
pub struct PlayerSpeedTimer(Timer);

impl PlayerSpeedTimer {
    pub fn new() -> Self {
        PlayerSpeedTimer(Timer::from_seconds(1.0, TimerMode::Repeating))
    }

    pub fn reset(&mut self) {
        self.0.reset()
    }
}

fn healing(
    mut player_health: ResMut<crate::player::PlayerHealth>,
    player_skills: Res<PlayerSkills>,
    mut heal_timer: ResMut<HealTimer>,
    mut writer: EventWriter<HealEvent>,
    time: Res<Time>,
) {
    heal_timer.tick(time.delta());

    if !player_health.dead && player_skills.get_unlocked(Skill::Healing) {
        for _ in 0..heal_timer.0.times_finished_this_tick() {
            // we can only heal health that's actually missing
            let healed = f32::min(
                player_skills.healing() * player_health.maximum,
                player_health.maximum - player_health.current,
            );

            if healed > 0.0 {
                player_health.current += healed;
                writer.send(HealEvent);
            }
        }
    }
}

fn update_player_speed(
    mut query: Query<
        &mut crate::character_controller::CharacterController,
        With<crate::player::Player>,
    >,
    player_skills: Res<PlayerSkills>,
    mut levelups: EventReader<LevelUpEvent>,
) {
    if levelups
        .read()
        .any(|LevelUpEvent { skill, .. }| *skill == Skill::Speed)
    {
        if let Ok(mut character_controller) = query.get_single_mut() {
            character_controller.max_speed = player_skills.get_total_speed();
        }
    }
}

fn speed_xp(
    query: Query<&bevy_rapier2d::prelude::Velocity, With<crate::player::Player>>,
    mut writer: EventWriter<SkillXpEvent>,
    mut speed_timer: ResMut<PlayerSpeedTimer>,
    time: Res<Time>,
) {
    let Ok(velocity) = query.get_single() else {
        return;
    };

    if velocity.linvel.length_squared() > 0.0 {
        speed_timer.0.tick(time.delta());
    }

    if speed_timer.0.times_finished_this_tick() > 0 {
        writer.send(SkillXpEvent {
            skill: Skill::Speed,
            xp: speed_timer.0.times_finished_this_tick() as f32,
        });
    }
}

fn send_xp_events(
    mut writer: EventWriter<SkillXpEvent>,
    mut damage_events: EventReader<crate::damage::DamageEvent>,
    mut melee_attack_events: EventReader<crate::damage::MeleeAttackEvent>,
    mut damage_blocked_events: EventReader<crate::damage::DamageBlockedEvent>,
    mut projectile_reflected_event: EventReader<crate::projectiles::ProjectileReflectEvent>,
    mut heal_events: EventReader<HealEvent>,
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

    // Melee attack events / Sword skill AND Pants skill
    for crate::damage::MeleeAttackEvent { .. } in melee_attack_events.read() {
        writer.send(SkillXpEvent {
            skill: Skill::Sword,
            xp: 1.0,
        });

        writer.send(SkillXpEvent {
            skill: Skill::Pants,
            xp: 1.0,
        });
    }

    // Attacks blocked / Shield skill
    for crate::damage::DamageBlockedEvent {} in damage_blocked_events.read() {
        writer.send(SkillXpEvent {
            skill: Skill::Shield,
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

    // heals / Healing skill
    for HealEvent in heal_events.read() {
        writer.send(SkillXpEvent {
            skill: Skill::Healing,
            xp: 1.0,
        });
    }
}
