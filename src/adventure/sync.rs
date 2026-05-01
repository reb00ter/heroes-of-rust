use bevy::prelude::*;

use crate::core::map::MapObject;

use super::{
    DayText, GameStateResource, GoldText, HeroMarker, MovementHighlight, MovementPointsText,
    ResourcePileMarker, TILE_SIZE, grid_to_world,
};

// ---------------------------------------------------------------------------
// Синхронизация позиции героя
// ---------------------------------------------------------------------------

/// Обновляет `Transform` сущности героя в соответствии с `hero.position` из `GameState`.
#[allow(clippy::needless_pass_by_value)]
pub fn sync_hero_transform(
    game_state: Res<GameStateResource>,
    mut hero_q: Query<(&HeroMarker, &mut Transform)>,
) {
    let gs = &game_state.0;
    for (marker, mut transform) in &mut hero_q {
        if let Some(hero) = gs.get_hero(marker.0) {
            let world = grid_to_world(hero.position);
            transform.translation.x = world.x;
            transform.translation.y = world.y;
        }
    }
}

// ---------------------------------------------------------------------------
// Подсветка доступных ходов
// ---------------------------------------------------------------------------

/// Пересоздаёт подсветку проходимых соседних клеток для первого героя.
#[allow(clippy::needless_pass_by_value)]
pub fn update_available_moves(
    mut commands: Commands,
    highlights: Query<Entity, With<MovementHighlight>>,
    game_state: Res<GameStateResource>,
) {
    // Удаляем старые подсветки
    for entity in &highlights {
        commands.entity(entity).despawn();
    }

    let gs = &game_state.0;
    let Some(hero) = gs.heroes.first() else {
        return;
    };

    // Если нет очков движения — не показываем доступные ходы
    if hero.movement_points == 0 {
        return;
    }

    for neighbor in gs.map.neighbors(hero.position) {
        if gs.map.is_passable(neighbor) {
            let world = grid_to_world(neighbor);
            commands.spawn((
                Sprite {
                    color: Color::srgba(0.25, 0.80, 0.40, 0.45),
                    custom_size: Some(Vec2::splat(TILE_SIZE - 1.0)),
                    ..default()
                },
                Transform::from_xyz(world.x, world.y, 2.0),
                MovementHighlight,
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// Синхронизация кучек ресурсов
// ---------------------------------------------------------------------------

/// Удаляет спрайты кучек, которые уже были собраны.
#[allow(clippy::needless_pass_by_value)]
pub fn sync_resource_piles(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    piles_q: Query<(Entity, &ResourcePileMarker)>,
) {
    if !game_state.is_changed() {
        return;
    }

    let gs = &game_state.0;
    for (entity, marker) in &piles_q {
        let has_pile = matches!(
            gs.map.get(marker.0).and_then(|t| t.object.as_ref()),
            Some(MapObject::ResourcePile(_))
        );
        if !has_pile {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Обновление UI очков движения
// ---------------------------------------------------------------------------

/// Обновляет текст `"MP: X / Y"` при изменении `GameState`.
#[allow(clippy::needless_pass_by_value)]
pub fn update_movement_ui(
    game_state: Res<GameStateResource>,
    mut text_q: Query<&mut Text, With<MovementPointsText>>,
) {
    if !game_state.is_changed() {
        return;
    }

    let gs = &game_state.0;
    let Some(hero) = gs.heroes.first() else {
        return;
    };

    let new_text = format!(
        "MP: {} / {}",
        hero.movement_points, hero.movement_points_max
    );

    for mut text in &mut text_q {
        (**text).clone_from(&new_text);
    }
}

// ---------------------------------------------------------------------------
// Обновление UI золота
// ---------------------------------------------------------------------------

/// Обновляет текст `"Золото: X"` при изменении `GameState`.
#[allow(clippy::needless_pass_by_value)]
pub fn update_resource_ui(
    game_state: Res<GameStateResource>,
    mut text_q: Query<&mut Text, With<GoldText>>,
) {
    if !game_state.is_changed() {
        return;
    }

    let gs = &game_state.0;
    let gold = gs.players.first().map_or(0, |p| p.resources.gold);
    let new_text = format!("Золото: {gold}");

    for mut text in &mut text_q {
        (**text).clone_from(&new_text);
    }
}

// ---------------------------------------------------------------------------
// Обновление UI текущего дня
// ---------------------------------------------------------------------------

/// Обновляет текст `"День X"` при изменении `GameState`.
#[allow(clippy::needless_pass_by_value)]
pub fn update_day_ui(
    game_state: Res<GameStateResource>,
    mut text_q: Query<&mut Text, With<DayText>>,
) {
    if !game_state.is_changed() {
        return;
    }

    let gs = &game_state.0;
    let new_text = format!("День {}", gs.current_day);

    for mut text in &mut text_q {
        (**text).clone_from(&new_text);
    }
}
