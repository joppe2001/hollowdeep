//! RON data loader
//!
//! Loads game data from external RON files, with fallback to hardcoded defaults.

use std::path::Path;
use std::fs;

use crate::progression::{Skill, SkillRarity};
use super::items::{ItemTemplates, default_item_templates};
use super::enemies::{EnemyTemplates, default_enemy_templates};
use super::synergies::{SynergyDefs, default_synergy_defs};

/// Manages all external game data
#[derive(Debug, Clone)]
pub struct DataManager {
    /// Item templates
    pub items: ItemTemplates,
    /// Enemy templates
    pub enemies: EnemyTemplates,
    /// Synergy definitions
    pub synergies: SynergyDefs,
    /// Skill definitions
    pub skills: SkillCollection,
}

/// Collection of skill definitions
#[derive(Debug, Clone, Default)]
pub struct SkillCollection {
    pub skills: Vec<Skill>,
}

impl SkillCollection {
    /// Find a skill by ID
    pub fn find(&self, id: u32) -> Option<&Skill> {
        self.skills.iter().find(|s| s.id == id)
    }

    /// Get all skills by rarity
    pub fn by_rarity(&self, rarity: SkillRarity) -> Vec<&Skill> {
        self.skills.iter().filter(|s| s.rarity == rarity).collect()
    }

    /// Get starting skills (Common skills with low IDs)
    pub fn starting_skills(&self) -> Vec<Skill> {
        self.skills.iter()
            .filter(|s| s.id <= 2)
            .cloned()
            .collect()
    }
}

impl DataManager {
    /// Create a new DataManager, loading from files or using defaults
    pub fn new() -> Self {
        Self::load_from_assets().unwrap_or_else(|e| {
            eprintln!("Warning: Failed to load data files: {}. Using defaults.", e);
            Self::default()
        })
    }

    /// Load data from assets/data/ directory
    pub fn load_from_assets() -> Result<Self, String> {
        let base_path = Path::new("assets/data");

        // Try to load each file, fall back to defaults if missing
        let items = Self::load_items(base_path);
        let enemies = Self::load_enemies(base_path);
        let synergies = Self::load_synergies(base_path);
        let skills = Self::load_skills(base_path);

        Ok(Self {
            items,
            enemies,
            synergies,
            skills,
        })
    }

    /// Load item templates from RON file
    fn load_items(base_path: &Path) -> ItemTemplates {
        let path = base_path.join("items.ron");
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match ron::from_str(&content) {
                        Ok(templates) => return templates,
                        Err(e) => eprintln!("Warning: Failed to parse items.ron: {}", e),
                    }
                }
                Err(e) => eprintln!("Warning: Failed to read items.ron: {}", e),
            }
        }
        default_item_templates()
    }

    /// Load enemy templates from RON file
    fn load_enemies(base_path: &Path) -> EnemyTemplates {
        let path = base_path.join("enemies.ron");
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match ron::from_str(&content) {
                        Ok(templates) => return templates,
                        Err(e) => eprintln!("Warning: Failed to parse enemies.ron: {}", e),
                    }
                }
                Err(e) => eprintln!("Warning: Failed to read enemies.ron: {}", e),
            }
        }
        default_enemy_templates()
    }

    /// Load synergy definitions from RON file
    fn load_synergies(base_path: &Path) -> SynergyDefs {
        let path = base_path.join("synergies.ron");
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match ron::from_str(&content) {
                        Ok(defs) => return defs,
                        Err(e) => eprintln!("Warning: Failed to parse synergies.ron: {}", e),
                    }
                }
                Err(e) => eprintln!("Warning: Failed to read synergies.ron: {}", e),
            }
        }
        default_synergy_defs()
    }

    /// Load skills from RON file (skills are already serializable)
    fn load_skills(base_path: &Path) -> SkillCollection {
        let path = base_path.join("skills.ron");
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => {
                    match ron::from_str::<Vec<Skill>>(&content) {
                        Ok(skills) => return SkillCollection { skills },
                        Err(e) => eprintln!("Warning: Failed to parse skills.ron: {}", e),
                    }
                }
                Err(e) => eprintln!("Warning: Failed to read skills.ron: {}", e),
            }
        }
        default_skills()
    }

    /// Get item templates
    pub fn item_templates(&self) -> &ItemTemplates {
        &self.items
    }

    /// Get enemy templates
    pub fn enemy_templates(&self) -> &EnemyTemplates {
        &self.enemies
    }

    /// Get synergy definitions
    pub fn synergy_defs(&self) -> &SynergyDefs {
        &self.synergies
    }

    /// Get skill collection
    pub fn skill_collection(&self) -> &SkillCollection {
        &self.skills
    }
}

impl Default for DataManager {
    fn default() -> Self {
        Self {
            items: default_item_templates(),
            enemies: default_enemy_templates(),
            synergies: default_synergy_defs(),
            skills: default_skills(),
        }
    }
}

/// Create default skill collection (from existing skill functions)
pub fn default_skills() -> SkillCollection {
    use crate::progression::skills::*;

    SkillCollection {
        skills: vec![
            // Starting skills
            skill_power_strike(),
            skill_first_aid(),

            // Common
            skill_quick_strike(),
            skill_bandage(),
            skill_bash(),

            // Uncommon
            skill_envenom(),
            skill_iron_skin(),
            skill_burning_strike(),
            skill_battle_cry(),
            skill_recuperate(),

            // Rare
            skill_whirlwind(),
            skill_shadow_step(),
            skill_frost_nova(),
            skill_life_drain(),
            skill_executioner(),

            // Epic
            skill_berserker_rage(),
            skill_chain_lightning(),
            skill_shield_wall(),
            skill_assassinate(),

            // Legendary
            skill_meteor_strike(),
            skill_divine_intervention(),
            skill_deaths_embrace(),
        ],
    }
}

/// Export all default data to RON files for easy editing
pub fn export_default_data() -> Result<(), String> {
    let base_path = Path::new("assets/data");

    // Create directory if it doesn't exist
    if !base_path.exists() {
        fs::create_dir_all(base_path)
            .map_err(|e| format!("Failed to create assets/data directory: {}", e))?;
    }

    // Export items
    let items = default_item_templates();
    let items_ron = ron::ser::to_string_pretty(&items, ron::ser::PrettyConfig::default())
        .map_err(|e| format!("Failed to serialize items: {}", e))?;
    fs::write(base_path.join("items.ron"), items_ron)
        .map_err(|e| format!("Failed to write items.ron: {}", e))?;

    // Export enemies
    let enemies = default_enemy_templates();
    let enemies_ron = ron::ser::to_string_pretty(&enemies, ron::ser::PrettyConfig::default())
        .map_err(|e| format!("Failed to serialize enemies: {}", e))?;
    fs::write(base_path.join("enemies.ron"), enemies_ron)
        .map_err(|e| format!("Failed to write enemies.ron: {}", e))?;

    // Export synergies
    let synergies = default_synergy_defs();
    let synergies_ron = ron::ser::to_string_pretty(&synergies, ron::ser::PrettyConfig::default())
        .map_err(|e| format!("Failed to serialize synergies: {}", e))?;
    fs::write(base_path.join("synergies.ron"), synergies_ron)
        .map_err(|e| format!("Failed to write synergies.ron: {}", e))?;

    // Export skills
    let skills = default_skills();
    let skills_ron = ron::ser::to_string_pretty(&skills.skills, ron::ser::PrettyConfig::default())
        .map_err(|e| format!("Failed to serialize skills: {}", e))?;
    fs::write(base_path.join("skills.ron"), skills_ron)
        .map_err(|e| format!("Failed to write skills.ron: {}", e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_default_data() {
        // Export default data to RON files
        let result = export_default_data();
        assert!(result.is_ok(), "Failed to export default data: {:?}", result.err());

        // Verify files were created
        let base_path = Path::new("assets/data");
        assert!(base_path.join("items.ron").exists(), "items.ron not created");
        assert!(base_path.join("enemies.ron").exists(), "enemies.ron not created");
        assert!(base_path.join("synergies.ron").exists(), "synergies.ron not created");
        assert!(base_path.join("skills.ron").exists(), "skills.ron not created");
    }

    #[test]
    fn test_load_default_data() {
        // First export the data
        let _ = export_default_data();

        // Then load it back
        let manager = DataManager::new();

        // Verify data was loaded
        assert!(!manager.items.templates.is_empty(), "No item templates loaded");
        assert!(!manager.enemies.templates.is_empty(), "No enemy templates loaded");
        assert!(!manager.synergies.synergies.is_empty(), "No synergy definitions loaded");
        assert!(!manager.skills.skills.is_empty(), "No skills loaded");
    }
}
