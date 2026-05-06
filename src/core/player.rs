#![allow(dead_code)]

use crate::core::hero::{Army, HeroId, UnitType};
use crate::core::map::Position;
use crate::core::resources::ResourceBag;

// ---------------------------------------------------------------------------
// IDs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TownId(pub u32);

// ---------------------------------------------------------------------------
// Town
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Town {
    pub id: TownId,
    pub position: Position,
    pub garrison: Army,
    /// Список доступных для найма существ: (тип, доступное количество)
    pub available_recruits: Vec<(UnitType, u32)>,
    /// Доход, начисляемый активному игроку в начале каждого дня
    pub income: ResourceBag,
    /// Суточный прирост для каждого слота `available_recruits`
    pub daily_growth: Vec<u32>,
}

impl Town {
    #[must_use]
    pub fn new(id: TownId, position: Position, income: ResourceBag) -> Self {
        Self {
            id,
            position,
            garrison: Army::new(),
            available_recruits: Vec::new(),
            income,
            daily_growth: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Player
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Player {
    pub id: PlayerId,
    pub resources: ResourceBag,
    pub hero_ids: Vec<HeroId>,
}

impl Player {
    #[must_use]
    pub fn new(id: PlayerId) -> Self {
        Self {
            id,
            resources: ResourceBag::default(),
            hero_ids: Vec::new(),
        }
    }
}
