use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::core::hero::HeroId;
use crate::core::map::{MapObject, Position};

use super::{
    ArmyText, DayText, GameStateResource, GoldText, HeroMarker, MovementHighlight,
    MovementPointsText, NeutralArmyMarker, ResourcePileMarker, TILE_SIZE, grid_to_world,
    spawn_gold_pile, spawn_hero, spawn_neutral_army,
};

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

    let map_w = gs.map.width;
    let map_h = gs.map.height;

    for neighbor in gs.map.neighbors(hero.position) {
        if gs.map.is_passable(neighbor) {
            let world = grid_to_world(neighbor, map_w, map_h);
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
// Синхронизация динамических объектов карты
// ---------------------------------------------------------------------------

/// Сверяет entity на сцене с `GameState` и приводит их в соответствие.
/// Запускается каждый кадр, но реально работает только при изменении `GameState`.
///
/// Принцип: визуальное состояние = функция от игрового состояния.
/// Явного «сброса карты» не нужно — эта система делает всё сама.
#[allow(clippy::needless_pass_by_value)]
pub fn sync_map_objects(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_state: Res<GameStateResource>,
    pile_q: Query<(Entity, &ResourcePileMarker)>,
    army_q: Query<(Entity, &NeutralArmyMarker)>,
    mut hero_q: Query<(Entity, &HeroMarker, &mut Transform)>,
) {
    if !game_state.is_changed() {
        return;
    }
    let gs = &game_state.0;
    let map_w = gs.map.width;
    let map_h = gs.map.height;

    // --- Кучки золота ---
    let pile_entities: HashMap<Position, Entity> = pile_q.iter().map(|(e, m)| (m.0, e)).collect();

    #[allow(clippy::cast_possible_wrap)]
    for y in 0..map_h {
        for x in 0..map_w {
            let pos = Position::new(x as i32, y as i32);
            if matches!(
                gs.map.get(pos).and_then(|t| t.object.as_ref()),
                Some(MapObject::ResourcePile(_))
            ) && !pile_entities.contains_key(&pos)
            {
                spawn_gold_pile(&mut commands, &asset_server, pos, map_w, map_h);
            }
        }
    }
    for (entity, marker) in &pile_q {
        if !matches!(
            gs.map.get(marker.0).and_then(|t| t.object.as_ref()),
            Some(MapObject::ResourcePile(_))
        ) {
            commands.entity(entity).despawn();
        }
    }

    // --- Нейтральные отряды ---
    let army_entities: HashMap<Position, Entity> = army_q.iter().map(|(e, m)| (m.0, e)).collect();

    #[allow(clippy::cast_possible_wrap)]
    for y in 0..map_h {
        for x in 0..map_w {
            let pos = Position::new(x as i32, y as i32);
            if matches!(
                gs.map.get(pos).and_then(|t| t.object.as_ref()),
                Some(MapObject::NeutralArmy(_))
            ) && !army_entities.contains_key(&pos)
            {
                spawn_neutral_army(&mut commands, pos, map_w, map_h);
            }
        }
    }
    for (entity, marker) in &army_q {
        if !matches!(
            gs.map.get(marker.0).and_then(|t| t.object.as_ref()),
            Some(MapObject::NeutralArmy(_))
        ) {
            commands.entity(entity).despawn();
        }
    }

    // --- Герой ---
    // За один проход: обновляем Transform существующих entity и собираем их ID.
    // Затем спауним тех, кого ещё нет.
    let mut seen_ids: HashSet<HeroId> = HashSet::new();
    for (_, marker, mut transform) in &mut hero_q {
        if let Some(hero) = gs.heroes.iter().find(|h| h.id == marker.0) {
            let world = grid_to_world(hero.position, map_w, map_h);
            transform.translation.x = world.x;
            transform.translation.y = world.y;
            seen_ids.insert(marker.0);
        }
    }
    for hero in &gs.heroes {
        if !seen_ids.contains(&hero.id) {
            spawn_hero(&mut commands, hero, map_w, map_h);
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

// ---------------------------------------------------------------------------
// Обновление UI армии
// ---------------------------------------------------------------------------

/// Обновляет строку `"Армия: …"` при изменении `GameState`.
#[allow(clippy::needless_pass_by_value)]
pub fn update_army_ui(
    game_state: Res<GameStateResource>,
    mut text_q: Query<&mut Text, With<ArmyText>>,
) {
    if !game_state.is_changed() {
        return;
    }
    let gs = &game_state.0;
    let new_text = gs.heroes.first().map_or_else(
        || "Армия: (пусто)".to_string(),
        |hero| {
            if hero.army.0.is_empty() {
                "Армия: (пусто)".to_string()
            } else {
                let parts: Vec<String> = hero
                    .army
                    .0
                    .iter()
                    .map(|s| format!("{} ×{}", s.unit_type.name, s.count))
                    .collect();
                format!("Армия: {}", parts.join(", "))
            }
        },
    );
    for mut text in &mut text_q {
        (**text).clone_from(&new_text);
    }
}
