#![allow(dead_code)]

use crate::core::hero::{Hero, HeroId};
use crate::core::map::AdventureMap;
use crate::core::player::{Player, PlayerId, Town, TownId};

#[derive(Debug, Clone)]
pub struct GameState {
    pub map: AdventureMap,
    pub players: Vec<Player>,
    pub heroes: Vec<Hero>,
    pub towns: Vec<Town>,
    pub current_day: u32,
    pub active_player_id: PlayerId,
}

impl GameState {
    pub fn get_hero(&self, id: HeroId) -> Option<&Hero> {
        self.heroes.iter().find(|h| h.id == id)
    }

    pub fn get_hero_mut(&mut self, id: HeroId) -> Option<&mut Hero> {
        self.heroes.iter_mut().find(|h| h.id == id)
    }

    pub fn get_player(&self, id: PlayerId) -> Option<&Player> {
        self.players.iter().find(|p| p.id == id)
    }

    pub fn get_player_mut(&mut self, id: PlayerId) -> Option<&mut Player> {
        self.players.iter_mut().find(|p| p.id == id)
    }

    pub fn get_town_mut(&mut self, id: TownId) -> Option<&mut Town> {
        self.towns.iter_mut().find(|t| t.id == id)
    }

    /// Активный игрок всегда существует — паника была бы ошибкой инициализации.
    pub fn active_player_mut(&mut self) -> &mut Player {
        let id = self.active_player_id;
        self.players
            .iter_mut()
            .find(|p| p.id == id)
            .expect("active player must always exist")
    }
}
