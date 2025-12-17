//! Progression systems

pub mod xp;
pub mod skills;
pub mod unlocks;
pub mod difficulty;

pub use difficulty::{Difficulty, FloorScaling, floor_hp_scale, floor_xp_scale, floor_stat_scale};
pub use skills::{Skill, SkillId, SkillCost, TargetType, SkillEffect, EquippedSkills, SkillRarity};
pub use skills::{skill_power_strike, skill_first_aid, starting_skills, learnable_skills, generate_shrine_skills};
