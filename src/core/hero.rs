#![allow(dead_code)]

use crate::core::map::AdventureMap;
use crate::core::map::Position;
use crate::core::resources::ResourceBag;

pub const MAX_ARMY_SLOTS: usize = 7;

// ---------------------------------------------------------------------------
// UnitType
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitType {
    pub name: String,
    pub damage_per_unit: u32,
    pub hp: u32,
    pub cost: ResourceBag,
}

// ---------------------------------------------------------------------------
// UnitStack
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct UnitStack {
    pub unit_type: UnitType,
    pub count: u32,
    pub hp_remaining: u32,
}

impl UnitStack {
    #[must_use]
    pub fn new(unit_type: UnitType, count: u32) -> Self {
        let hp_remaining = unit_type.hp;
        Self {
            unit_type,
            count,
            hp_remaining,
        }
    }

    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.count > 0
    }
}

// ---------------------------------------------------------------------------
// Army errors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArmyFull;

// ---------------------------------------------------------------------------
// Army
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct Army(pub Vec<UnitStack>);

impl Army {
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add_stack(&mut self, stack: UnitStack) -> Result<(), ArmyFull> {
        if let Some(existing) = self
            .0
            .iter_mut()
            .find(|s| s.unit_type.name == stack.unit_type.name)
        {
            existing.count += stack.count;
            return Ok(());
        }
        if self.0.len() >= MAX_ARMY_SLOTS {
            return Err(ArmyFull);
        }
        self.0.push(stack);
        Ok(())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|s| !s.is_alive())
    }
}

// ---------------------------------------------------------------------------
// Hero errors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoMovementPoints;

// ---------------------------------------------------------------------------
// HeroId / Hero
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeroId(pub u32);

#[derive(Debug, Clone)]
pub struct Hero {
    pub id: HeroId,
    pub name: String,
    pub position: Position,
    pub army: Army,
    pub movement_points: u32,
    pub movement_points_max: u32,
    pub sight_range: u32,
    pub attack: u32,
    pub defense: u32,
}

impl Hero {
    /// Возвращает `true`, если `pos` смежна с текущей позицией героя
    /// (4 направления) И клетка проходима.
    #[must_use]
    pub fn can_move_to(&self, pos: Position, map: &AdventureMap) -> bool {
        let dx = (pos.x - self.position.x).abs();
        let dy = (pos.y - self.position.y).abs();
        let adjacent = (dx == 1 && dy == 0) || (dx == 0 && dy == 1);
        adjacent && map.is_passable(pos)
    }

    pub fn spend_movement(&mut self, cost: u32) -> Result<(), NoMovementPoints> {
        if self.movement_points < cost {
            return Err(NoMovementPoints);
        }
        self.movement_points -= cost;
        Ok(())
    }

    pub fn restore_movement(&mut self) {
        self.movement_points = self.movement_points_max;
    }
}
