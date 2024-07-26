use std::ops::{Index, IndexMut};

use bevy::prelude::*;

use super::LevelUpEvent;

#[derive(Debug, Clone, Copy)]
pub enum Skill {
    Armor,
    Sword,
    Shield,
    Pants,
    Mirror,
    Healing,
    Speed,
}

impl Skill {
    pub fn iter() -> impl Iterator<Item = Skill> {
        [
            Skill::Armor,
            Skill::Sword,
            Skill::Shield,
            Skill::Pants,
            Skill::Mirror,
            Skill::Healing,
            Skill::Speed,
        ]
        .iter()
        .copied()
    }
}

#[derive(Debug, Default, Resource)]
pub struct PlayerSkills {
    /// Holds the skills the player has accumulated in previous cycles;
    /// these are the values that are saved if the game exits now
    stored_levels: PlayerSkillsImpl<u64>,
    /// holds the skill levels that the player has gained this cycle
    delta_levels: PlayerSkillsImpl<u64>,

    /// xp leftover from previous cycles
    stored_xp: PlayerSkillsImpl<f32>,

    /// xp from this cycle
    delta_xp: PlayerSkillsImpl<f32>,

    /// stores the number of level-ups since they were last cleared
    levelups: PlayerSkillsImpl<u64>,

    /// whether or not each skill is unlocked
    unlocked: PlayerSkillsImpl<bool>,
}

#[derive(Debug, Default)]
struct PlayerSkillsImpl<T> {
    armor: T,
    sword: T,
    shield: T,
    pants: T,
    mirror: T,
    healing: T,
    speed: T,
}

impl<T> Index<Skill> for PlayerSkillsImpl<T> {
    type Output = T;
    fn index(&self, index: Skill) -> &Self::Output {
        match index {
            Skill::Armor => &self.armor,
            Skill::Sword => &self.sword,
            Skill::Shield => &self.shield,
            Skill::Pants => &self.pants,
            Skill::Mirror => &self.mirror,
            Skill::Healing => &self.healing,
            Skill::Speed => &self.speed,
        }
    }
}

impl<T> IndexMut<Skill> for PlayerSkillsImpl<T> {
    fn index_mut(&mut self, index: Skill) -> &mut Self::Output {
        match index {
            Skill::Armor => &mut self.armor,
            Skill::Sword => &mut self.sword,
            Skill::Shield => &mut self.shield,
            Skill::Pants => &mut self.pants,
            Skill::Mirror => &mut self.mirror,
            Skill::Healing => &mut self.healing,
            Skill::Speed => &mut self.speed,
        }
    }
}

impl PlayerSkills {
    /// Get the total level of a skill
    pub fn get(&self, skill: Skill) -> u64 {
        self.stored_levels[skill] + self.delta_levels[skill]
    }

    /// same as `PlayerSkills::get` but returns an `f32`
    pub fn get_f32(&self, skill: Skill) -> f32 {
        self.get(skill) as f32
    }

    /// Get the level of a skill as of the beginning of the current cycle; i.e. the highest level previously achieved
    pub fn get_highest(&self, skill: Skill) -> u64 {
        self.stored_levels[skill]
    }

    /// same as `PlayerSkills::get_highest` but returns an `f32`
    pub fn get_highest_f32(&self, skill: Skill) -> f32 {
        self.get_highest(skill) as f32
    }

    pub fn get_xp(&self, skill: Skill) -> f32 {
        self.stored_xp[skill] + self.delta_xp[skill]
    }

    /// Add the given base amount of xp to the given skill, processing level-ups as necessary
    pub fn add_xp(&mut self, skill: Skill, xp: f32) {
        // no xp until a skill is unlocked
        if !self.unlocked[skill] {
            return;
        }

        // TODO: maybe give an xp bonus based on highest previously achieved?
        // let xp = xp * (1.0 + (self.get_highest_f32(skill) / 100.0).sqrt());
        self.delta_xp[skill] += xp;
        loop {
            let current_xp = self.get_xp(skill);
            let xp_needed = self.xp_needed(skill);
            if current_xp.is_infinite() || current_xp.is_nan() {
                error!("PlayerSkills::add_xp: infinite or nan xp");
                return;
            }
            if self.subtract_xp(skill, xp_needed) {
                // successfully leveled up
                self.levelups[skill] += 1;
            } else {
                return;
            }
        }
    }

    /// returns the total amount of xp required to gain one level in the given skill.
    /// does not include any xp currently stored
    pub fn xp_needed(&self, skill: Skill) -> f32 {
        self.get_f32(skill) / 2.0
    }

    /// if the total amount of xp for this skill is greater than `xp`,
    /// then this function subtracts that amount first from the delta_xp
    /// field, then the `stored_xp` field if necessary.
    ///
    /// returns true if there was enough xp and it performed the subtraction,
    /// and false if there was not enough and it did not perform the subtraction
    fn subtract_xp(&mut self, skill: Skill, mut xp: f32) -> bool {
        if self.get_xp(skill) < xp {
            return false;
        }

        if self.delta_xp[skill] >= xp {
            self.delta_xp[skill] -= xp;
            return true;
        } else {
            xp -= self.delta_xp[skill];
            self.delta_xp[skill] = 0.0;
        }
        self.stored_xp[skill] -= xp;
        debug_assert!(self.stored_xp[skill] >= 0.0 && self.delta_xp[skill] >= 0.0);
        return true;
    }

    pub fn unlock_skill(&mut self, skill: Skill) {
        self.unlocked[skill] = true;
    }

    pub fn end_cycle(&mut self) {
        for skill in Skill::iter() {
            self.stored_levels[skill] += self.delta_levels[skill];
            self.delta_levels[skill] = 0;

            self.stored_xp[skill] += self.delta_xp[skill];
            self.delta_xp[skill] = 0.0;
        }
    }

    pub fn drain_levelups(&mut self, buf: &mut Vec<LevelUpEvent>) {
        for skill in Skill::iter() {
            buf.push(LevelUpEvent {
                num_levels: self.levelups[skill],
                skill,
            });
            self.levelups[skill] = 0;
        }
    }

    /// the amount of damage the player will deal
    pub fn attack_damage(&self) -> f32 {
        10.0 + (1.0 / 60.0) * (self.get_f32(Skill::Sword)).powi(2)
    }

    /// the fraction of damage the player will take. returns a value between 0 and 1
    pub fn damage_taken(&self) -> f32 {
        1.0 / (1.0 + self.get_f32(Skill::Armor) / 100.0)
    }

    /// the chance an incoming attack will be blocked. returns a value between 0 and 1
    pub fn block_chance(&self) -> f32 {
        0.65 - 500.0 / (self.get_f32(Skill::Shield).powi(2) + 1000.0)
    }

    /// the chance the player will reflect a projectile attack
    pub fn reflect_chance(&self) -> f32 {
        0.75 - 700.0 / (self.get_f32(Skill::Mirror).powi(2) + 1000.0)
    }

    /// mass divides the magnitude of incoming knockback, and multiplies the outgoing magnitude
    pub fn mass(&self) -> f32 {
        1.0 + (self.get_f32(Skill::Pants) / 10.0).sqrt()
    }

    /// returns the fraction of health healed every 5 seconds
    pub fn healing(&self) -> f32 {
        0.5 - 98.0 / (self.get_f32(Skill::Healing).powi(2) + 200.0)
    }

    /// returns the player's speed multiplier
    pub fn speed(&self) -> f32 {
        1.0 + f32::log2(self.get_f32(Skill::Speed) + 1.0) / f32::log2(25.0)
    }
}
