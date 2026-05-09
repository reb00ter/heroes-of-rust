mod input;
mod render;
pub mod state;

use bevy::prelude::*;

use crate::adventure::{BannerKind, GameStateResource, NeutralArmyMarker, ShowBanner};
use crate::core::hero::UnitStack;
use crate::{GameOverResult, PendingBattle};
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
            (finish_battle, render::despawn_battle_ui).chain(),
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
    mut next_state: ResMut<NextState<crate::GameScreen>>,
) {
    let gs = &game_state.0;

    let attacker_stacks = gs
        .get_hero(pending.attacker_hero_id)
        .map_or(&[][..], |h| h.army.0.as_slice());

    if attacker_stacks.is_empty() {
        warn!("[BATTLE] Attacker has no army — immediate defeat.");
        commands.insert_resource(BattleResult {
            winner: Side::Defender,
        });
        next_state.set(crate::GameScreen::Adventure);
        return;
    }

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

// ---------------------------------------------------------------------------
// Завершение боя (5.10)
// ---------------------------------------------------------------------------

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
fn finish_battle(
    mut commands: Commands,
    result: Option<Res<BattleResult>>,
    pending: Option<Res<PendingBattle>>,
    battle_state: Option<Res<BattleState>>,
    mut game_state: ResMut<GameStateResource>,
    marker_q: Query<(Entity, &NeutralArmyMarker)>,
    mut show_banner: ResMut<ShowBanner>,
    mut next_state: ResMut<NextState<crate::GameScreen>>,
) {
    let (Some(result), Some(pending)) = (result, pending) else {
        return;
    };

    if result.winner == Side::Attacker {
        // Обновить армию героя — заменить на живые стеки из боя
        if let Some(ref bs) = battle_state {
            let live_stacks: Vec<UnitStack> = bs
                .stacks
                .iter()
                .filter(|s| s.side == Side::Attacker && s.count > 0)
                .map(|s| UnitStack {
                    unit_type: s.unit_type.clone(),
                    count: s.count,
                    hp_remaining: s.hp_remaining,
                })
                .collect();

            if let Some(hero) = game_state.0.get_hero_mut(pending.attacker_hero_id) {
                hero.army.0 = live_stacks;
            }
        }

        // Убрать нейтральную армию с тайла карты
        let pos = pending.defender_pos;
        if let Some(tile) = game_state.0.map.get_mut(pos) {
            tile.object = None;
        }

        // Despawn спрайта маркера на карте
        for (entity, marker) in &marker_q {
            if marker.0 == pos {
                commands.entity(entity).despawn();
                break;
            }
        }

        info!(
            "[BATTLE] Victory! Hero army updated. Neutral army at ({},{}) removed.",
            pos.x, pos.y
        );

        // Проверить условие победы: все нейтралы побеждены?
        if game_state.0.count_neutral_armies() == 0 {
            info!("[BATTLE] All neutrals defeated! Game over — Victory.");
            commands.insert_resource(GameOverResult { is_victory: true });
            next_state.set(crate::GameScreen::GameOver);
        } else {
            show_banner.0 = Some(BannerKind::Victory);
        }
    } else {
        // Поражение — очистить армию героя
        if let Some(hero) = game_state.0.get_hero_mut(pending.attacker_hero_id) {
            hero.army.0.clear();
        }
        info!("[BATTLE] Defeat! Hero army cleared. Game over.");
        commands.insert_resource(GameOverResult { is_victory: false });
        next_state.set(crate::GameScreen::GameOver);
    }

    // Убрать ресурсы боя
    commands.remove_resource::<BattleResult>();
    commands.remove_resource::<PendingBattle>();
    if battle_state.is_some() {
        commands.remove_resource::<BattleState>();
    }
}
