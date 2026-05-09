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

    /// Возвращает количество нейтральных армий, оставшихся на карте.
    #[must_use]
    pub fn count_neutral_armies(&self) -> usize {
        use crate::core::map::MapObject;
        self.map
            .tiles
            .iter()
            .filter(|t| matches!(t.object, Some(MapObject::NeutralArmy(_))))
            .count()
    }
}

// ---------------------------------------------------------------------------
// Тесты
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::core::hero::{Army, UnitStack, UnitType};
    use crate::core::map::{AdventureMap, MapObject};
    use crate::core::player::{Player, PlayerId};
    use crate::core::resources::ResourceBag;

    use super::*;

    fn minimal_state(map: AdventureMap) -> GameState {
        GameState {
            map,
            players: vec![Player::new(PlayerId(0))],
            heroes: vec![],
            towns: vec![],
            current_day: 1,
            active_player_id: PlayerId(0),
        }
    }

    fn neutral_army() -> MapObject {
        let unit = UnitType {
            name: "Гоблин".to_string(),
            damage_per_unit: 2,
            hp: 5,
            cost: ResourceBag::gold(0),
        };
        MapObject::NeutralArmy(Army(vec![UnitStack::new(unit, 3)]))
    }

    #[test]
    fn victory_when_all_neutrals_dead() {
        let map = AdventureMap::new(5, 5);
        let gs = minimal_state(map);
        assert_eq!(gs.count_neutral_armies(), 0);
    }

    #[test]
    fn neutrals_counted_correctly() {
        let mut map = AdventureMap::new(5, 5);
        map.get_mut(crate::core::map::Position::new(1, 0))
            .unwrap()
            .object = Some(neutral_army());
        map.get_mut(crate::core::map::Position::new(2, 0))
            .unwrap()
            .object = Some(neutral_army());
        map.get_mut(crate::core::map::Position::new(3, 0))
            .unwrap()
            .object = Some(neutral_army());

        let mut gs = minimal_state(map);
        assert_eq!(gs.count_neutral_armies(), 3);

        gs.map
            .get_mut(crate::core::map::Position::new(1, 0))
            .unwrap()
            .object = None;
        assert_eq!(gs.count_neutral_armies(), 2);
    }
}
