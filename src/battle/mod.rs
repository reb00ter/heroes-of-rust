pub mod state;

use bevy::prelude::*;

use crate::adventure::GameStateResource;
use crate::PendingBattle;
use state::BattleState;

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(crate::GameScreen::Battle), setup_battle);
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

    let battle_state =
        BattleState::from_armies(attacker_stacks, &pending.defender_army.0);

    info!(
        "[BATTLE] Setup: {} attacker stacks vs {} defender stacks.",
        battle_state.stacks.iter().filter(|s| s.side == state::Side::Attacker).count(),
        battle_state.stacks.iter().filter(|s| s.side == state::Side::Defender).count(),
    );

    commands.insert_resource(battle_state);
}
