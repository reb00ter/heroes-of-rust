#![allow(dead_code)]

use bevy::log::info;

use crate::core::hero::HeroId;
use crate::core::hero::{ArmyFull, UnitStack};
use crate::core::map::{MapObject, Position};
use crate::core::player::{PlayerId, TownId};
use crate::core::resources::ResourceBag;
use crate::core::state::GameState;

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub enum GameCommand {
    MoveHero {
        hero_id: HeroId,
        target: Position,
    },
    CollectResource {
        hero_id: HeroId,
    },
    RecruitUnits {
        town_id: TownId,
        unit_type_idx: usize,
        count: u32,
    },
    EndTurn,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
    HeroMoved {
        hero_id: HeroId,
        from: Position,
        to: Position,
    },
    ResourceCollected {
        hero_id: HeroId,
        amount: ResourceBag,
    },
    BattleStarted {
        attacker: HeroId,
        defender_pos: Position,
    },
    TurnEnded {
        day: u32,
    },
    DayIncome {
        player_id: PlayerId,
        amount: ResourceBag,
    },
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandError {
    HeroNotFound,
    NotYourHero,
    NotAdjacent,
    TileNotPassable,
    NoMovementPoints,
    NothingToCollect,
    TownNotFound,
    NotInTown,
    NotEnoughRecruitsAvailable,
    InsufficientFunds,
    ArmyFull,
}

impl From<ArmyFull> for CommandError {
    fn from(_: ArmyFull) -> Self {
        CommandError::ArmyFull
    }
}

// ---------------------------------------------------------------------------
// apply
// ---------------------------------------------------------------------------

impl GameState {
    pub fn apply(&mut self, command: GameCommand) -> Result<Vec<GameEvent>, CommandError> {
        match command {
            GameCommand::MoveHero { hero_id, target } => self.cmd_move_hero(hero_id, target),
            GameCommand::CollectResource { hero_id } => self.cmd_collect_resource(hero_id),
            GameCommand::RecruitUnits {
                town_id,
                unit_type_idx,
                count,
            } => self.cmd_recruit_units(town_id, unit_type_idx, count),
            GameCommand::EndTurn => self.cmd_end_turn(),
        }
    }

    // -----------------------------------------------------------------------
    // MoveHero
    // -----------------------------------------------------------------------

    fn cmd_move_hero(
        &mut self,
        hero_id: HeroId,
        target: Position,
    ) -> Result<Vec<GameEvent>, CommandError> {
        // 1. Найти героя
        let hero = self.get_hero(hero_id).ok_or(CommandError::HeroNotFound)?;

        // 2. Герой принадлежит активному игроку
        let active_id = self.active_player_id;
        let player = self
            .get_player(active_id)
            .ok_or(CommandError::HeroNotFound)?;
        if !player.hero_ids.contains(&hero_id) {
            return Err(CommandError::NotYourHero);
        }

        // 3. Клетка смежна
        let from = hero.position;
        let dx = (target.x - from.x).abs();
        let dy = (target.y - from.y).abs();
        if !((dx == 1 && dy == 0) || (dx == 0 && dy == 1)) {
            return Err(CommandError::NotAdjacent);
        }

        // 4. Если на клетке нейтральная армия — начать бой (без перемещения)
        if matches!(
            self.map.get(target).and_then(|t| t.object.as_ref()),
            Some(MapObject::NeutralArmy(_))
        ) {
            let hero = self.get_hero(hero_id).ok_or(CommandError::HeroNotFound)?;
            if hero.movement_points == 0 {
                return Err(CommandError::NoMovementPoints);
            }
            info!(
                "[ADVENTURE] Hero \"{}\" initiates battle at ({},{}).",
                hero.name, target.x, target.y
            );
            return Ok(vec![GameEvent::BattleStarted {
                attacker: hero_id,
                defender_pos: target,
            }]);
        }

        // 5. Клетка проходима
        if !self.map.is_passable(target) {
            return Err(CommandError::TileNotPassable);
        }

        // 6. Есть очки движения
        let hero = self.get_hero(hero_id).ok_or(CommandError::HeroNotFound)?;
        if hero.movement_points == 0 {
            return Err(CommandError::NoMovementPoints);
        }
        let mp_before = hero.movement_points;

        // 7. Переместить
        let hero = self
            .get_hero_mut(hero_id)
            .ok_or(CommandError::HeroNotFound)?;
        hero.movement_points -= 1;
        hero.position = target;
        let hero_name = hero.name.clone();
        let mp_after = hero.movement_points;

        info!(
            "[ADVENTURE] Hero \"{}\" moved from ({},{}) to ({},{}). MP: {} -> {}",
            hero_name, from.x, from.y, target.x, target.y, mp_before, mp_after
        );

        Ok(vec![GameEvent::HeroMoved {
            hero_id,
            from,
            to: target,
        }])
    }

    // -----------------------------------------------------------------------
    // CollectResource
    // -----------------------------------------------------------------------

    fn cmd_collect_resource(&mut self, hero_id: HeroId) -> Result<Vec<GameEvent>, CommandError> {
        // 1. Найти героя
        let hero = self.get_hero(hero_id).ok_or(CommandError::HeroNotFound)?;
        let pos = hero.position;
        let hero_name = hero.name.clone();

        // 2. Есть ли ресурс на клетке
        let tile = self.map.get(pos).ok_or(CommandError::NothingToCollect)?;
        let amount = match &tile.object {
            Some(MapObject::ResourcePile(bag)) => bag.clone(),
            _ => return Err(CommandError::NothingToCollect),
        };

        // 3. Убрать объект с карты
        if let Some(tile) = self.map.get_mut(pos) {
            tile.object = None;
        }

        // 4. Зачислить игроку
        let player = self.active_player_mut();
        player.resources.add(&amount);
        let total = player.resources.gold;

        info!(
            "[ADVENTURE] Hero \"{}\" collected {} gold at ({},{}). Total gold: {}",
            hero_name, amount.gold, pos.x, pos.y, total
        );

        Ok(vec![GameEvent::ResourceCollected { hero_id, amount }])
    }

    // -----------------------------------------------------------------------
    // RecruitUnits
    // -----------------------------------------------------------------------

    fn cmd_recruit_units(
        &mut self,
        town_id: TownId,
        unit_type_idx: usize,
        count: u32,
    ) -> Result<Vec<GameEvent>, CommandError> {
        // 1. Найти город
        let town = self
            .towns
            .iter()
            .find(|t| t.id == town_id)
            .ok_or(CommandError::TownNotFound)?;
        let town_pos = town.position;

        // 2. Найти героя активного игрока, стоящего в городе
        let active_id = self.active_player_id;
        let player = self
            .get_player(active_id)
            .ok_or(CommandError::HeroNotFound)?;
        let hero_id = player
            .hero_ids
            .iter()
            .copied()
            .find(|&hid| {
                self.heroes
                    .iter()
                    .any(|h| h.id == hid && h.position == town_pos)
            })
            .ok_or(CommandError::NotInTown)?;

        // 3. Проверить доступные существа
        let town = self
            .towns
            .iter()
            .find(|t| t.id == town_id)
            .ok_or(CommandError::TownNotFound)?;

        let (unit_type, available) = town
            .available_recruits
            .get(unit_type_idx)
            .ok_or(CommandError::NotEnoughRecruitsAvailable)?;

        if *available < count {
            return Err(CommandError::NotEnoughRecruitsAvailable);
        }

        // 4. Стоимость найма
        let unit_cost_gold = unit_type.cost.gold;
        let unit_name = unit_type.name.clone();
        let total_cost = ResourceBag::gold(unit_cost_gold * count);

        let player = self
            .get_player(active_id)
            .ok_or(CommandError::HeroNotFound)?;
        if !player.resources.can_afford(&total_cost) {
            return Err(CommandError::InsufficientFunds);
        }

        // 5. Списать золото
        let player = self.active_player_mut();
        player
            .resources
            .subtract(&total_cost)
            .map_err(|_| CommandError::InsufficientFunds)?;

        // 6. Уменьшить доступное количество и создать стек
        let town = self
            .towns
            .iter_mut()
            .find(|t| t.id == town_id)
            .ok_or(CommandError::TownNotFound)?;
        town.available_recruits[unit_type_idx].1 -= count;
        let unit_type_clone = town.available_recruits[unit_type_idx].0.clone();
        let stack = UnitStack::new(unit_type_clone, count);

        // 7. Добавить стек в армию героя
        let hero = self
            .get_hero_mut(hero_id)
            .ok_or(CommandError::HeroNotFound)?;
        hero.army.add_stack(stack)?;

        let army_summary: Vec<String> = hero
            .army
            .0
            .iter()
            .map(|s| format!("{} x{}", s.unit_type.name, s.count))
            .collect();

        info!(
            "[TOWN] Hero recruited {} {} for {} gold. Army: [{}]",
            count,
            unit_name,
            total_cost.gold,
            army_summary.join(", ")
        );

        Ok(vec![])
    }

    // -----------------------------------------------------------------------
    // EndTurn
    // -----------------------------------------------------------------------

    fn cmd_end_turn(&mut self) -> Result<Vec<GameEvent>, CommandError> {
        let active_id = self.active_player_id;

        // 1. Восстановить очки движения героев активного игрока
        let player = self
            .get_player(active_id)
            .ok_or(CommandError::HeroNotFound)?;
        let hero_ids: Vec<HeroId> = player.hero_ids.clone();
        for hid in &hero_ids {
            if let Some(hero) = self.get_hero_mut(*hid) {
                hero.restore_movement();
            }
        }

        // 2. Доход городов
        let mut events: Vec<GameEvent> = Vec::new();
        let town_incomes: Vec<ResourceBag> = self.towns.iter().map(|t| t.income.clone()).collect();

        let player = self.active_player_mut();
        for income in &town_incomes {
            player.resources.add(income);
            events.push(GameEvent::DayIncome {
                player_id: active_id,
                amount: income.clone(),
            });
        }

        // 3. Пополнение доступных существ в городах
        let mut growth_log: Vec<String> = Vec::new();
        for town in &mut self.towns {
            for (i, &growth) in town.daily_growth.iter().enumerate() {
                if let Some(slot) = town.available_recruits.get_mut(i) {
                    slot.1 += growth;
                    growth_log.push(format!("+{} {}", growth, slot.0.name));
                }
            }
        }
        if !growth_log.is_empty() {
            info!("[TOWN] Daily growth: {}", growth_log.join(", "));
        }

        // 4. Следующий день
        self.current_day += 1;

        let income_total: u32 = town_incomes.iter().map(|b| b.gold).sum();
        info!(
            "[TURN] Day {} begins. Hero MP restored. Town income: +{} gold",
            self.current_day, income_total
        );

        events.push(GameEvent::TurnEnded {
            day: self.current_day,
        });

        Ok(events)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::hero::UnitType;
    use crate::core::hero::{Army, Hero, HeroId};
    use crate::core::map::{AdventureMap, MapObject, Position, TileKind};
    use crate::core::player::{Player, PlayerId, Town, TownId};
    use crate::core::resources::ResourceBag;
    use crate::core::state::GameState;

    fn make_unit_type() -> UnitType {
        UnitType {
            name: "Goblin".to_string(),
            damage_per_unit: 2,
            hp: 5,
            cost: ResourceBag::gold(50),
        }
    }

    fn make_test_state() -> GameState {
        let mut map = AdventureMap::new(10, 10);
        // Препятствие на (2, 0)
        map.get_mut(Position::new(2, 0)).unwrap().kind = TileKind::Obstacle;

        let hero = Hero {
            id: HeroId(1),
            name: "Aldric".to_string(),
            position: Position::new(1, 0),
            army: Army::new(),
            movement_points: 5,
            movement_points_max: 5,
        };

        let mut player = Player::new(PlayerId(1));
        player.hero_ids.push(HeroId(1));
        player.resources = ResourceBag::gold(500);

        GameState {
            map,
            players: vec![player],
            heroes: vec![hero],
            towns: vec![],
            current_day: 1,
            active_player_id: PlayerId(1),
        }
    }

    // 1. Герой движется на свободную клетку
    #[test]
    fn hero_moves_to_free_tile() {
        let mut state = make_test_state();
        let result = state.apply(GameCommand::MoveHero {
            hero_id: HeroId(1),
            target: Position::new(0, 0),
        });
        assert!(result.is_ok());
        let hero = state.get_hero(HeroId(1)).unwrap();
        assert_eq!(hero.position, Position::new(0, 0));
        assert_eq!(hero.movement_points, 4);
    }

    // 2. Герой движется на препятствие — ошибка
    #[test]
    fn hero_cannot_move_to_obstacle() {
        let mut state = make_test_state();
        let result = state.apply(GameCommand::MoveHero {
            hero_id: HeroId(1),
            target: Position::new(2, 0),
        });
        assert_eq!(result, Err(CommandError::TileNotPassable));
    }

    // 3. Герой движется без очков движения — ошибка
    #[test]
    fn hero_cannot_move_without_mp() {
        let mut state = make_test_state();
        state.get_hero_mut(HeroId(1)).unwrap().movement_points = 0;
        let result = state.apply(GameCommand::MoveHero {
            hero_id: HeroId(1),
            target: Position::new(0, 0),
        });
        assert_eq!(result, Err(CommandError::NoMovementPoints));
    }

    // 4. Герой собирает ресурс
    #[test]
    fn hero_collects_resource() {
        let mut state = make_test_state();
        // Положить ресурс на клетку под героем (1,0)
        state.map.get_mut(Position::new(1, 0)).unwrap().object =
            Some(MapObject::ResourcePile(ResourceBag::gold(100)));

        let result = state.apply(GameCommand::CollectResource { hero_id: HeroId(1) });
        assert!(result.is_ok());

        // Золото зачислено
        let player = state.get_player(PlayerId(1)).unwrap();
        assert_eq!(player.resources.gold, 600);

        // Объект удалён
        let tile = state.map.get(Position::new(1, 0)).unwrap();
        assert!(tile.object.is_none());
    }

    // 5. Герой пытается собрать с пустой клетки — ошибка
    #[test]
    fn hero_cannot_collect_from_empty_tile() {
        let mut state = make_test_state();
        let result = state.apply(GameCommand::CollectResource { hero_id: HeroId(1) });
        assert_eq!(result, Err(CommandError::NothingToCollect));
    }

    // 6. Найм существ — успех
    #[test]
    fn recruit_units_success() {
        let mut state = make_test_state();

        // Город на позиции героя
        let town = Town {
            id: TownId(1),
            position: Position::new(1, 0),
            garrison: Army::new(),
            available_recruits: vec![(make_unit_type(), 10)],
            income: ResourceBag::default(),
            daily_growth: vec![],
        };
        state.towns.push(town);

        let result = state.apply(GameCommand::RecruitUnits {
            town_id: TownId(1),
            unit_type_idx: 0,
            count: 3,
        });
        assert!(result.is_ok(), "{result:?}");

        // Золото списано: 3 × 50 = 150
        let player = state.get_player(PlayerId(1)).unwrap();
        assert_eq!(player.resources.gold, 350);

        // Стек добавлен в армию
        let hero = state.get_hero(HeroId(1)).unwrap();
        assert_eq!(hero.army.0.len(), 1);
        assert_eq!(hero.army.0[0].count, 3);
    }

    // 7. Найм при нехватке золота — ошибка
    #[test]
    fn recruit_units_insufficient_funds() {
        let mut state = make_test_state();
        state.active_player_mut().resources = ResourceBag::gold(10);

        let town = Town {
            id: TownId(1),
            position: Position::new(1, 0),
            garrison: Army::new(),
            available_recruits: vec![(make_unit_type(), 10)],
            income: ResourceBag::default(),
            daily_growth: vec![],
        };
        state.towns.push(town);

        let result = state.apply(GameCommand::RecruitUnits {
            town_id: TownId(1),
            unit_type_idx: 0,
            count: 1,
        });
        assert_eq!(result, Err(CommandError::InsufficientFunds));
    }

    // 8. EndTurn восстанавливает очки движения
    #[test]
    fn end_turn_restores_movement() {
        let mut state = make_test_state();
        state.get_hero_mut(HeroId(1)).unwrap().movement_points = 1;

        state.apply(GameCommand::EndTurn).unwrap();

        let hero = state.get_hero(HeroId(1)).unwrap();
        assert_eq!(hero.movement_points, hero.movement_points_max);
    }

    // 10. Герой не в городе — NotInTown
    #[test]
    fn recruit_units_hero_not_in_town() {
        let mut state = make_test_state();
        let town = Town {
            id: TownId(1),
            position: Position::new(5, 5), // герой стоит на (1,0)
            garrison: Army::new(),
            available_recruits: vec![(make_unit_type(), 10)],
            income: ResourceBag::default(),
            daily_growth: vec![],
        };
        state.towns.push(town);

        let result = state.apply(GameCommand::RecruitUnits {
            town_id: TownId(1),
            unit_type_idx: 0,
            count: 1,
        });
        assert_eq!(result, Err(CommandError::NotInTown));
    }

    // 11. EndTurn пополняет available_recruits
    #[test]
    fn end_turn_replenishes_recruits() {
        let mut state = make_test_state();
        let town = Town {
            id: TownId(1),
            position: Position::new(5, 5),
            garrison: Army::new(),
            available_recruits: vec![(make_unit_type(), 3)],
            income: ResourceBag::default(),
            daily_growth: vec![5],
        };
        state.towns.push(town);

        state.apply(GameCommand::EndTurn).unwrap();

        let town = state.towns.iter().find(|t| t.id == TownId(1)).unwrap();
        assert_eq!(town.available_recruits[0].1, 8); // 3 + 5
    }

    // 9. EndTurn начисляет доход города
    #[test]
    fn end_turn_grants_income() {
        let mut state = make_test_state();
        let town = Town {
            id: TownId(1),
            position: Position::new(5, 5),
            garrison: Army::new(),
            available_recruits: vec![],
            income: ResourceBag::gold(250),
            daily_growth: vec![],
        };
        state.towns.push(town);

        let gold_before = state.get_player(PlayerId(1)).unwrap().resources.gold;
        state.apply(GameCommand::EndTurn).unwrap();
        let gold_after = state.get_player(PlayerId(1)).unwrap().resources.gold;

        assert_eq!(gold_after - gold_before, 250);
    }

    // 12. Герой рядом с нейтральной армией — генерируется BattleStarted, позиция не меняется
    #[test]
    fn move_into_neutral_army_triggers_battle() {
        use crate::core::hero::UnitStack;
        let mut state = make_test_state();
        let goblin = make_unit_type();
        state.map.get_mut(Position::new(0, 0)).unwrap().object = Some(MapObject::NeutralArmy(
            Army(vec![UnitStack::new(goblin, 3)]),
        ));

        let result = state.apply(GameCommand::MoveHero {
            hero_id: HeroId(1),
            target: Position::new(0, 0),
        });

        let events = result.expect("command should succeed");
        assert!(events.contains(&GameEvent::BattleStarted {
            attacker: HeroId(1),
            defender_pos: Position::new(0, 0),
        }));
        // Герой остался на месте
        assert_eq!(
            state.get_hero(HeroId(1)).unwrap().position,
            Position::new(1, 0)
        );
    }
}
