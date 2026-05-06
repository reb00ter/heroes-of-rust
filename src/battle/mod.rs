pub mod state;
mod input;
mod render;

use bevy::prelude::*;

use crate::adventure::GameStateResource;
use crate::PendingBattle;
use state::{BattleState, Side};

/// Результат боя — доступен в Adventure после завершения (читается в 5.10/5.11).
#[derive(Resource)]
pub struct BattleResult {
    #[allow(dead_code)]
    pub winner: Side,
}

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
                OnEnter(crate::GameScreen::Battle),
                // apply_deferred нужен чтобы BattleState из setup_battle
                // был доступен для spawn_battle_ui в том же расписании
                (setup_battle, ApplyDeferred, render::spawn_battle_ui).chain(),
            )
            .add_systems(
                OnExit(crate::GameScreen::Battle),
                render::despawn_battle_ui,
            )
            .add_systems(
                Update,
                // input первым — меняет BattleState; render вторым — отражает изменения
                (input::handle_battle_input, render::update_battle_ui)
                    .chain()
                    .run_if(in_state(crate::GameScreen::Battle)),
            );
    }
}

#[allow(clippy::needless_pass_by_value)]
fn setup_battle(
    mut commands: Commands,
    pending: Res<PendingBattle>,
    game_state: Res<GameStateResource>,
) {
    let gs = &game_state.0;

    let attacker_stacks = gs
        .get_hero(pending.attacker_hero_id)
        .map(|h| h.army.0.as_slice())
        .unwrap_or(&[]);

    let battle_state = BattleState::from_armies(attacker_stacks, &pending.defender_army.0);

    info!(
        "[BATTLE] Setup: {} attacker stacks vs {} defender stacks.",
        battle_state
            .stacks
            .iter()
            .filter(|s| s.side == Side::Attacker)
            .count(),
        battle_state
            .stacks
            .iter()
            .filter(|s| s.side == Side::Defender)
            .count(),
    );

    commands.insert_resource(battle_state);
}
