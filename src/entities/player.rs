//! Player entity creation

use hecs::World;
use crate::ecs::{
    Position, Renderable, Name, Player, Stats, Health, Mana, Stamina,
    Experience, FieldOfView, FactionComponent, Faction,
    InventoryComponent, EquipmentComponent, StatPoints, SkillsComponent,
    StatusEffects,
};
use crate::items::{Inventory, Equipment, item::templates};
use crate::items::loot::next_item_id;
use crate::progression::{EquippedSkills, skill_power_strike, skill_first_aid};

/// Spawn the player entity
pub fn spawn_player(world: &mut World, pos: Position) -> hecs::Entity {
    let stats = Stats::player_base();

    // Create inventory with starting items
    let mut inventory = Inventory::new();
    inventory.add_item(templates::health_potion(next_item_id()));
    inventory.add_item(templates::health_potion(next_item_id()));
    inventory.add_item(templates::mana_potion(next_item_id()));

    // Create equipment with starting weapon
    let mut equipment = Equipment::new();
    equipment.equip(templates::rusty_dagger(next_item_id()));

    // Create skills with starting abilities
    let mut skills = EquippedSkills::new();
    // Learn starting skills first, then equip them
    let power_strike = skill_power_strike();
    let first_aid = skill_first_aid();
    skills.learn(power_strike.clone());
    skills.learn(first_aid.clone());
    skills.equip(0, power_strike); // Slot 1: Power Strike
    skills.equip(1, first_aid);    // Slot 2: First Aid

    // Note: hecs has a tuple limit, so we spawn with initial components
    // then add more separately
    let entity = world.spawn((
        Player,
        Name::new("Hero"),
        pos,
        Renderable::new('@', (255, 255, 200)).with_order(100),
        stats,
        Health::new(100),
        Mana::new(50),
        Stamina::new(50),
        Experience::new(),
        StatPoints(0),
        FieldOfView::new(8),
        FactionComponent(Faction::Player),
        InventoryComponent { inventory },
        EquipmentComponent { equipment },
    ));

    // Add remaining components
    let _ = world.insert(entity, (
        SkillsComponent { skills },
        StatusEffects::default(),
    ));

    entity
}
