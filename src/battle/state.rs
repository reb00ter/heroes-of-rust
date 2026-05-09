use std::collections::VecDeque;

use bevy::log::info;
use bevy::prelude::*;

use crate::core::hero::UnitType;

// ---------------------------------------------------------------------------
// Идентификаторы и стороны
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StackId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Attacker,
    Defender,
}

// ---------------------------------------------------------------------------
// BattleStack
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BattleStack {
    pub id: StackId,
    pub unit_type: UnitType,
    pub count: u32,
    pub hp_remaining: u32,
    pub side: Side,
}

// ---------------------------------------------------------------------------
// BattleEvent
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BattleEvent {
    Attacked {
        attacker_id: StackId,
        target_id: StackId,
        damage: u32,
        killed: u32,
    },
    StackDied {
        id: StackId,
    },
    TurnPassed {
        next_id: StackId,
    },
    BattleOver {
        winner: Side,
    },
}

// ---------------------------------------------------------------------------
// BattleState
// ---------------------------------------------------------------------------

#[derive(Resource)]
#[allow(dead_code)]
pub struct BattleState {
    pub stacks: Vec<BattleStack>,
    pub turn_order: VecDeque<StackId>,
    pub current_stack_id: StackId,
    pub log: Vec<BattleEvent>,
}

#[allow(dead_code)]
impl BattleState {
    /// Создаёт `BattleState` из армий атакующего и защитника.
    /// Порядок хода: чередование атакующих и защитников по индексу.
    pub fn from_armies(
        attacker_stacks: &[crate::core::hero::UnitStack],
        defender_stacks: &[crate::core::hero::UnitStack],
    ) -> Self {
        let mut stacks: Vec<BattleStack> = Vec::new();
        let mut next_id = 0u32;

        for us in attacker_stacks {
            stacks.push(BattleStack {
                id: StackId(next_id),
                hp_remaining: us.unit_type.hp,
                unit_type: us.unit_type.clone(),
                count: us.count,
                side: Side::Attacker,
            });
            next_id += 1;
        }
        for us in defender_stacks {
            stacks.push(BattleStack {
                id: StackId(next_id),
                hp_remaining: us.unit_type.hp,
                unit_type: us.unit_type.clone(),
                count: us.count,
                side: Side::Defender,
            });
            next_id += 1;
        }

        // Порядок хода: чередуем атакующих и защитников
        let attackers: Vec<StackId> = stacks
            .iter()
            .filter(|s| s.side == Side::Attacker)
            .map(|s| s.id)
            .collect();
        let defenders: Vec<StackId> = stacks
            .iter()
            .filter(|s| s.side == Side::Defender)
            .map(|s| s.id)
            .collect();

        let max_len = attackers.len().max(defenders.len());
        let mut turn_order = VecDeque::new();
        for i in 0..max_len {
            if let Some(&id) = attackers.get(i) {
                turn_order.push_back(id);
            }
            if let Some(&id) = defenders.get(i) {
                turn_order.push_back(id);
            }
        }

        let current_stack_id = turn_order.front().copied().unwrap_or(StackId(0));

        Self {
            stacks,
            turn_order,
            current_stack_id,
            log: Vec::new(),
        }
    }

    pub fn current_stack(&self) -> Option<&BattleStack> {
        self.stacks.iter().find(|s| s.id == self.current_stack_id)
    }

    /// Атакует `target_id` текущим стеком. Возвращает список событий.
    pub fn attack(&mut self, target_id: StackId) -> Vec<BattleEvent> {
        let attacker_id = self.current_stack_id;

        let Some(a) = self.stacks.iter().find(|s| s.id == attacker_id) else {
            return vec![];
        };
        let (attacker_count, attacker_dmg, attacker_side) =
            (a.count, a.unit_type.damage_per_unit, a.side);

        let Some(tgt) = self.stacks.iter().find(|s| s.id == target_id) else {
            return vec![];
        };
        let target_side = tgt.side;

        // Цель должна быть противоположной стороны
        if attacker_side == target_side {
            return vec![];
        }

        let damage = attacker_count * attacker_dmg;
        let Some(target) = self.stacks.iter_mut().find(|s| s.id == target_id) else {
            return vec![];
        };

        let target_hp = target.unit_type.hp;
        let full_kills = damage / target_hp;
        let remainder = damage % target_hp;

        let mut killed = full_kills;
        if remainder >= target.hp_remaining {
            killed += 1;
            target.hp_remaining = target_hp - (remainder - target.hp_remaining);
            if target.hp_remaining == 0 {
                target.hp_remaining = target_hp;
            }
        } else {
            target.hp_remaining -= remainder;
        }

        let killed = killed.min(target.count);
        target.count = target.count.saturating_sub(killed);
        let target_count_after = target.count;
        let target_name = target.unit_type.name.clone();

        let attacker_name = self.stacks.iter()
            .find(|s| s.id == attacker_id)
            .map(|s| s.unit_type.name.clone())
            .unwrap_or_default();
        let attacker_side = self.stacks.iter()
            .find(|s| s.id == attacker_id)
            .map(|s| s.side)
            .unwrap_or(Side::Attacker);
        let side_label = if attacker_side == Side::Attacker { "Att" } else { "Def" };

        info!(
            "[BATTLE] [{}] {} attacks {}: {} dmg, killed {} (remaining {})",
            side_label, attacker_name, target_name, damage, killed, target_count_after
        );

        let mut events = vec![BattleEvent::Attacked {
            attacker_id,
            target_id,
            damage,
            killed,
        }];

        if target_count_after == 0 {
            self.turn_order.retain(|&id| id != target_id);
            info!("[BATTLE] Stack {} destroyed.", target_name);
            events.push(BattleEvent::StackDied { id: target_id });
        }

        let turn_events = self.next_turn();
        events.extend(turn_events);
        self.log.extend(events.clone());
        events
    }

    /// Передаёт ход следующему стеку в очереди.
    pub fn next_turn(&mut self) -> Vec<BattleEvent> {
        // Убрать текущий стек с головы и положить в хвост (если жив)
        if let Some(front) = self.turn_order.pop_front() {
            let alive = self.stacks.iter().any(|s| s.id == front && s.count > 0);
            if alive {
                self.turn_order.push_back(front);
            }
        }

        if let Some(&next_id) = self.turn_order.front() {
            self.current_stack_id = next_id;
            if let Some(next) = self.stacks.iter().find(|s| s.id == next_id) {
                let side_label = if next.side == Side::Attacker { "Att" } else { "Def" };
                info!(
                    "[BATTLE] Turn: {} [{}] x{}",
                    next.unit_type.name, side_label, next.count
                );
            }
            vec![BattleEvent::TurnPassed { next_id }]
        } else {
            vec![]
        }
    }

    /// Возвращает победителя, если бой закончен.
    pub fn is_over(&self) -> Option<Side> {
        let any_attacker = self.stacks.iter().any(|s| s.side == Side::Attacker && s.count > 0);
        let any_defender = self.stacks.iter().any(|s| s.side == Side::Defender && s.count > 0);
        match (any_attacker, any_defender) {
            (false, _) => Some(Side::Defender),
            (_, false) => Some(Side::Attacker),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Тесты (5.4 + 5.5)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::hero::{UnitStack, UnitType};
    use crate::core::resources::ResourceBag;

    fn goblin() -> UnitType {
        UnitType { name: "Goblin".into(), damage_per_unit: 2, hp: 5, cost: ResourceBag::gold(0) }
    }

    fn swordsman() -> UnitType {
        UnitType { name: "Swordsman".into(), damage_per_unit: 3, hp: 10, cost: ResourceBag::gold(0) }
    }

    fn make_battle(att: &[(UnitType, u32)], def: &[(UnitType, u32)]) -> BattleState {
        let att_stacks: Vec<UnitStack> = att.iter().map(|(ut, n)| UnitStack::new(ut.clone(), *n)).collect();
        let def_stacks: Vec<UnitStack> = def.iter().map(|(ut, n)| UnitStack::new(ut.clone(), *n)).collect();
        BattleState::from_armies(&att_stacks, &def_stacks)
    }

    #[test]
    fn battle_attack_kills_units() {
        // Атакующий: 5 гоблинов (2 урона) → 10 урона → убивает 1 мечника (hp=10)
        let mut b = make_battle(&[(goblin(), 5)], &[(swordsman(), 3)]);
        let def_id = b.stacks.iter().find(|s| s.side == Side::Defender).unwrap().id;
        b.attack(def_id);
        let def = b.stacks.iter().find(|s| s.id == def_id).unwrap();
        assert_eq!(def.count, 2);
    }

    #[test]
    fn battle_stack_removed_when_dead() {
        // 10 гоблинов (2 урона) → 20 урона убивают 2 мечников (hp=10, count=2)
        let mut b = make_battle(&[(goblin(), 10)], &[(swordsman(), 2)]);
        let def_id = b.stacks.iter().find(|s| s.side == Side::Defender).unwrap().id;
        b.attack(def_id);
        assert!(!b.turn_order.contains(&def_id));
        let def = b.stacks.iter().find(|s| s.id == def_id).unwrap();
        assert_eq!(def.count, 0);
    }

    #[test]
    fn battle_over_when_all_defenders_dead() {
        let mut b = make_battle(&[(goblin(), 10)], &[(swordsman(), 2)]);
        let def_id = b.stacks.iter().find(|s| s.side == Side::Defender).unwrap().id;
        b.attack(def_id);
        assert_eq!(b.is_over(), Some(Side::Attacker));
    }

    #[test]
    fn battle_turn_order_alternates() {
        // 2 атакующих стека + 2 защитника → чередование A0,D0,A1,D1
        let b = make_battle(
            &[(goblin(), 1), (goblin(), 1)],
            &[(swordsman(), 1), (swordsman(), 1)],
        );
        let order: Vec<Side> = b
            .turn_order
            .iter()
            .map(|id| b.stacks.iter().find(|s| s.id == *id).unwrap().side)
            .collect();
        assert_eq!(order, vec![Side::Attacker, Side::Defender, Side::Attacker, Side::Defender]);
    }

    #[test]
    fn battle_ai_attacks_attacker() {
        // Ход защитника: атакует первый живой стек атакующего
        let mut b = make_battle(&[(swordsman(), 3)], &[(goblin(), 5)]);
        // Первый в очереди — атакующий, следующий — защитник
        // Сдвигаем очередь вручную на защитника
        b.next_turn();
        let current = b.current_stack().unwrap();
        assert_eq!(current.side, Side::Defender);

        let att_id = b.stacks.iter().find(|s| s.side == Side::Attacker).unwrap().id;
        let events = b.attack(att_id);
        assert!(events.iter().any(|e| matches!(e, BattleEvent::Attacked { attacker_id, target_id, .. }
            if *target_id == att_id && b.stacks.iter().any(|s| s.id == *attacker_id && s.side == Side::Defender)
        )));
    }
}
