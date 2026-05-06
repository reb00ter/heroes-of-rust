use bevy::prelude::*;

use crate::core::commands::{CommandError, GameCommand, GameEvent};
use crate::core::hero::{Army, HeroId};
use crate::core::map::{MapObject, Position};
use crate::core::player::TownId;
use crate::{PendingBattle};

use super::{GameStateResource, HoverHighlight, MAP_HEIGHT, MAP_WIDTH, world_to_grid};

// Размеры карты в i32 для сравнений (16 и 12 — точно в диапазоне i32)
const MAP_W: i32 = MAP_WIDTH.cast_signed();
const MAP_H: i32 = MAP_HEIGHT.cast_signed();

// ---------------------------------------------------------------------------
// Управление с клавиатуры
// ---------------------------------------------------------------------------

/// Обрабатывает WASD / стрелки: перемещает героя на одну клетку за нажатие.
#[allow(clippy::needless_pass_by_value)] // Res<T> — стандартный SystemParam Bevy
pub fn keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameStateResource>,
    mut next_state: ResMut<NextState<crate::GameScreen>>,
    mut commands: Commands,
) {
    let gs = &game_state.0;

    let Some(hero_id) = get_active_hero_id(gs) else {
        return;
    };
    let Some(hero_pos) = gs.get_hero(hero_id).map(|h| h.position) else {
        return;
    };

    let delta = if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        Some((0, -1))
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        Some((0, 1))
    } else if keyboard.just_pressed(KeyCode::ArrowLeft) || keyboard.just_pressed(KeyCode::KeyA) {
        Some((-1, 0))
    } else if keyboard.just_pressed(KeyCode::ArrowRight) || keyboard.just_pressed(KeyCode::KeyD) {
        Some((1, 0))
    } else {
        None
    };

    let Some((dx, dy)) = delta else { return };

    let target = Position::new(hero_pos.x + dx, hero_pos.y + dy);
    match apply_move(&mut game_state, hero_id, target) {
        MoveOutcome::Town(town_id) => {
            commands.insert_resource(crate::town::CurrentTownId(town_id));
            next_state.set(crate::GameScreen::Town);
        }
        MoveOutcome::Battle(pending) => {
            commands.insert_resource(pending);
            next_state.set(crate::GameScreen::Battle);
        }
        MoveOutcome::None => {}
    }
}

// ---------------------------------------------------------------------------
// Управление мышью: клик на клетку
// ---------------------------------------------------------------------------

/// Обрабатывает левый клик: вычисляет позицию в сетке и отправляет команду движения.
#[allow(clippy::needless_pass_by_value)] // Res<T> — стандартный SystemParam Bevy
pub fn mouse_click_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut game_state: ResMut<GameStateResource>,
    mut next_state: ResMut<NextState<crate::GameScreen>>,
    mut commands: Commands,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_q.single() else {
        return;
    };

    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };

    let grid_pos = world_to_grid(world_pos);

    // Проверяем что клик в пределах карты
    if grid_pos.x < 0 || grid_pos.y < 0 || grid_pos.x >= MAP_W || grid_pos.y >= MAP_H {
        return;
    }

    let Some(hero_id) = get_active_hero_id(&game_state.0) else {
        return;
    };

    match apply_move(&mut game_state, hero_id, grid_pos) {
        MoveOutcome::Town(town_id) => {
            commands.insert_resource(crate::town::CurrentTownId(town_id));
            next_state.set(crate::GameScreen::Town);
        }
        MoveOutcome::Battle(pending) => {
            commands.insert_resource(pending);
            next_state.set(crate::GameScreen::Battle);
        }
        MoveOutcome::None => {}
    }
}

// ---------------------------------------------------------------------------
// Hover-подсветка клетки под курсором
// ---------------------------------------------------------------------------

/// Перемещает спрайт hover-подсветки к тайлу под курсором мыши.
pub fn update_hover_highlight(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut hover_q: Query<(&mut Transform, &mut Visibility), With<HoverHighlight>>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_q.single() else {
        return;
    };
    let Ok((mut hover_transform, mut hover_visibility)) = hover_q.single_mut() else {
        return;
    };

    if let Some(cursor_pos) = window.cursor_position()
        && let Ok(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos)
    {
        let grid_pos = world_to_grid(world_pos);

        if grid_pos.x >= 0 && grid_pos.y >= 0 && grid_pos.x < MAP_W && grid_pos.y < MAP_H {
            let snap = super::grid_to_world(grid_pos);
            hover_transform.translation.x = snap.x;
            hover_transform.translation.y = snap.y;
            *hover_visibility = Visibility::Visible;
            return;
        }
    }

    *hover_visibility = Visibility::Hidden;
}

// ---------------------------------------------------------------------------
// Вспомогательные функции
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Завершение хода
// ---------------------------------------------------------------------------

/// Обрабатывает нажатие Space/Enter — завершение хода.
#[allow(clippy::needless_pass_by_value)]
pub fn handle_end_turn(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<GameStateResource>,
) {
    if !keyboard.just_pressed(KeyCode::Space) && !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }

    let gs = &mut game_state.0;
    if let Err(e) = gs.apply(GameCommand::EndTurn) {
        warn!("[ADVENTURE] EndTurn command failed: {:?}", e);
    }
}

fn get_active_hero_id(gs: &crate::core::state::GameState) -> Option<HeroId> {
    let player = gs.get_player(gs.active_player_id)?;
    player.hero_ids.first().copied()
}

enum MoveOutcome {
    Town(TownId),
    Battle(PendingBattle),
    None,
}

fn apply_move(
    game_state: &mut ResMut<GameStateResource>,
    hero_id: HeroId,
    target: Position,
) -> MoveOutcome {
    let gs = &mut game_state.0;
    let hero_name = gs
        .get_hero(hero_id)
        .map(|h| h.name.clone())
        .unwrap_or_default();

    match gs.apply(GameCommand::MoveHero { hero_id, target }) {
        Ok(events) => {
            // Бой?
            for event in &events {
                if let GameEvent::BattleStarted { attacker, defender_pos } = event {
                    let army = gs
                        .map
                        .get(*defender_pos)
                        .and_then(|t| t.object.as_ref())
                        .and_then(|obj| {
                            if let MapObject::NeutralArmy(a) = obj {
                                Some(a.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(Army::new);
                    return MoveOutcome::Battle(PendingBattle {
                        attacker_hero_id: *attacker,
                        defender_army: army,
                        defender_pos: *defender_pos,
                    });
                }
            }

            // Вошёл ли герой в город?
            if let Some(MapObject::Town(id)) = gs.map.get(target).and_then(|t| t.object.as_ref())
            {
                return MoveOutcome::Town(*id);
            }

            // Автосбор ресурса
            if matches!(
                gs.map.get(target).and_then(|t| t.object.as_ref()),
                Some(MapObject::ResourcePile(_))
            ) {
                if let Err(e) = gs.apply(GameCommand::CollectResource { hero_id }) {
                    warn!(
                        "[ADVENTURE] Auto-collect failed for hero \"{}\": {:?}",
                        hero_name, e
                    );
                }
            }
            MoveOutcome::None
        }
        Err(CommandError::NoMovementPoints) => {
            info!(
                "[ADVENTURE] Hero \"{}\" has no movement points left.",
                hero_name
            );
            MoveOutcome::None
        }
        Err(CommandError::NotAdjacent) => MoveOutcome::None,
        Err(CommandError::TileNotPassable) => {
            info!(
                "[ADVENTURE] Hero \"{}\" cannot move there — tile is not passable.",
                hero_name
            );
            MoveOutcome::None
        }
        Err(e) => {
            warn!(
                "[ADVENTURE] Move command failed for hero \"{}\": {:?}",
                hero_name, e
            );
            MoveOutcome::None
        }
    }
}
