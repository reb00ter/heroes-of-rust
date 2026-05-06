use bevy::prelude::*;

use crate::GameScreen;
use crate::battle::state::{BattleState, Side, StackId};

use super::BattleResult;

#[derive(Component)]
pub struct AttackButton(pub StackId);

// ---------------------------------------------------------------------------
// Система обработки ввода игрока
// ---------------------------------------------------------------------------

#[allow(clippy::needless_pass_by_value)]
pub fn handle_battle_input(
    mut battle_state: ResMut<BattleState>,
    attack_q: Query<(&Interaction, &AttackButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<GameScreen>>,
    mut commands: Commands,
) {
    let current_side = battle_state.current_stack().map(|s| s.side);

    match current_side {
        Some(Side::Attacker) => {
            for (interaction, btn) in &attack_q {
                if *interaction == Interaction::Pressed {
                    battle_state.attack(btn.0);

                    if let Some(winner) = battle_state.is_over() {
                        commands.insert_resource(BattleResult { winner });
                        next_state.set(GameScreen::Adventure);
                        return;
                    }

                    // После хода игрока — AI ходит пока очередь у защитника
                    if let Some(winner) = run_defender_ai(&mut battle_state) {
                        commands.insert_resource(BattleResult { winner });
                        next_state.set(GameScreen::Adventure);
                    }
                    return;
                }
            }
        }
        // Если ход оказался у защитника (например, атакующих нет) — запустить AI
        Some(Side::Defender) => {
            if let Some(winner) = run_defender_ai(&mut battle_state) {
                commands.insert_resource(BattleResult { winner });
                next_state.set(GameScreen::Adventure);
            }
        }
        None => {}
    }
}

// ---------------------------------------------------------------------------
// AI защитника (5.9)
// ---------------------------------------------------------------------------

/// Ходит за всех защитников подряд, пока не вернётся ход атакующего или бой не завершится.
/// Возвращает победителя если бой закончился.
pub fn run_defender_ai(battle_state: &mut BattleState) -> Option<Side> {
    loop {
        if battle_state.current_stack().map(|s| s.side) != Some(Side::Defender) {
            break;
        }
        let target_id = battle_state
            .stacks
            .iter()
            .find(|s| s.side == Side::Attacker && s.count > 0)
            .map(|s| s.id);

        let Some(target_id) = target_id else { break };

        battle_state.attack(target_id);

        if let Some(winner) = battle_state.is_over() {
            return Some(winner);
        }
    }
    None
}
