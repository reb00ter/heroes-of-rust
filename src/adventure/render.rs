use bevy::prelude::*;

use crate::core::hero::{Army, Hero, HeroId};
use crate::core::map::{AdventureMap, MapObject, TileKind};
use crate::core::player::{Player, PlayerId, Town, TownId};
use crate::core::resources::ResourceBag;
use crate::core::state::GameState;

use super::{
    DayText, GameStateResource, GoldText, HeroMarker, HoverHighlight, MAP_HEIGHT, MAP_WIDTH,
    MovementPointsText, ResourcePileMarker, TILE_SIZE, TileMarker, grid_to_world,
};

// ---------------------------------------------------------------------------
// Цвета тайлов и объектов
// ---------------------------------------------------------------------------

const COLOR_GROUND: Color = Color::srgb(0.20, 0.55, 0.20);
const COLOR_OBSTACLE: Color = Color::srgb(0.45, 0.45, 0.45);
const COLOR_WATER: Color = Color::srgb(0.15, 0.35, 0.80);
const COLOR_HERO: Color = Color::srgb(0.95, 0.80, 0.10);
const COLOR_HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 0.30);
const COLOR_RESOURCE_PILE: Color = Color::srgb(0.95, 0.75, 0.10);

// ---------------------------------------------------------------------------
// Стартовое состояние игры
// ---------------------------------------------------------------------------

/// Создаёт тестовое состояние игры для Этапа 2: карта 16×12 с препятствиями и героем.
pub fn build_initial_game_state() -> GameState {
    let mut map = AdventureMap::new(MAP_WIDTH, MAP_HEIGHT);

    // Расставляем препятствия — несколько групп скал
    let obstacles = [
        (3, 1),
        (3, 2),
        (3, 3),
        (3, 4),
        (7, 3),
        (7, 4),
        (7, 5),
        (7, 6),
        (10, 2),
        (10, 3),
        (11, 2),
        (13, 7),
        (13, 8),
        (14, 7),
        (14, 8),
        (5, 8),
        (5, 9),
        (6, 9),
    ];
    for (x, y) in obstacles {
        if let Some(tile) = map.get_mut(crate::core::map::Position::new(x, y)) {
            tile.kind = TileKind::Obstacle;
        }
    }

    // Несколько клеток воды у края
    let water = [(0, 10), (0, 11), (1, 11), (2, 11), (15, 0), (15, 1)];
    for (x, y) in water {
        if let Some(tile) = map.get_mut(crate::core::map::Position::new(x, y)) {
            tile.kind = TileKind::Water;
        }
    }

    // Кучки золота на карте
    let gold_piles: [(i32, i32, u32); 5] = [
        (5, 2, 100),
        (2, 6, 150),
        (8, 1, 200),
        (11, 6, 250),
        (4, 10, 100),
    ];
    for (x, y, amount) in gold_piles {
        if let Some(tile) = map.get_mut(crate::core::map::Position::new(x, y)) {
            tile.object = Some(MapObject::ResourcePile(ResourceBag::gold(amount)));
        }
    }

    // Город (без специального объекта пока, просто позиция для следующих этапов)
    let town = Town::new(
        TownId(0),
        crate::core::map::Position::new(12, 5),
        ResourceBag::gold(250),
    );

    // Герой
    let hero = Hero {
        id: HeroId(0),
        name: "Aldric".to_string(),
        position: crate::core::map::Position::new(1, 1),
        army: Army::new(),
        movement_points: 10,
        movement_points_max: 10,
    };

    // Игрок
    let mut player = Player::new(PlayerId(0));
    player.hero_ids.push(HeroId(0));
    player.resources = ResourceBag::gold(500);

    let obj_count = obstacles.len() + water.len();
    info!(
        "[ADVENTURE] Game initialized. Map: {}x{}, obstacles+water: {}, gold piles: {}.",
        MAP_WIDTH,
        MAP_HEIGHT,
        obj_count,
        gold_piles.len()
    );

    GameState {
        map,
        players: vec![player],
        heroes: vec![hero],
        towns: vec![town],
        current_day: 1,
        active_player_id: PlayerId(0),
    }
}

// ---------------------------------------------------------------------------
// Startup-система: спавн сущностей
// ---------------------------------------------------------------------------

#[allow(clippy::needless_pass_by_value)] // Res<T> — стандартный SystemParam, не может быть &Res<T>
pub fn startup_setup(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    asset_server: Res<AssetServer>,
) {
    let gs = &game_state.0;
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");

    // Камера по центру карты (мировой центр = (0,0))
    commands.spawn(Camera2d);

    // --- Тайлы карты (Z=0) ---
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            #[allow(clippy::cast_possible_wrap)] // x,y < 16/12, не переполнят i32
            let pos = crate::core::map::Position::new(x as i32, y as i32);
            let world = grid_to_world(pos);

            let color = match gs.map.get(pos) {
                Some(tile) => match tile.kind {
                    TileKind::Ground => COLOR_GROUND,
                    TileKind::Obstacle => COLOR_OBSTACLE,
                    TileKind::Water => COLOR_WATER,
                },
                None => COLOR_GROUND,
            };

            commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(Vec2::splat(TILE_SIZE - 1.0)), // зазор 1px
                    ..default()
                },
                Transform::from_xyz(world.x, world.y, 0.0),
                TileMarker { pos },
            ));
        }
    }

    // --- Кучки золота (Z=1) ---
    #[allow(clippy::cast_possible_wrap)]
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let pos = crate::core::map::Position::new(x as i32, y as i32);
            if matches!(
                gs.map.get(pos).and_then(|t| t.object.as_ref()),
                Some(MapObject::ResourcePile(_))
            ) {
                let world = grid_to_world(pos);
                commands.spawn((
                    Sprite {
                        color: COLOR_RESOURCE_PILE,
                        custom_size: Some(Vec2::splat(TILE_SIZE * 0.5)),
                        ..default()
                    },
                    Transform::from_xyz(world.x, world.y, 1.0),
                    ResourcePileMarker(pos),
                ));
            }
        }
    }

    // --- Герой (Z=4) ---
    if let Some(hero) = gs.heroes.first() {
        let world = grid_to_world(hero.position);
        commands.spawn((
            Sprite {
                color: COLOR_HERO,
                custom_size: Some(Vec2::splat(TILE_SIZE * 0.65)),
                ..default()
            },
            Transform::from_xyz(world.x, world.y, 4.0),
            HeroMarker(hero.id),
        ));
    }

    // --- Подсветка курсора (изначально скрыта) ---
    commands.spawn((
        Sprite {
            color: COLOR_HOVER,
            custom_size: Some(Vec2::splat(TILE_SIZE - 1.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 3.0),
        Visibility::Hidden,
        HoverHighlight,
    ));

    spawn_ui(&mut commands, gs, font);
}

/// Спавнит все UI-элементы на экране (MP, золото, день, подсказка).
fn spawn_ui(commands: &mut Commands, gs: &crate::core::state::GameState, font: Handle<Font>) {
    let mp = gs.heroes.first().map_or(0, |h| h.movement_points);
    let mp_max = gs.heroes.first().map_or(0, |h| h.movement_points_max);

    commands.spawn((
        Text::new(format!("MP: {mp} / {mp_max}")),
        TextFont {
            font: font.clone(),
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(12.0),
            ..default()
        },
        MovementPointsText,
    ));

    let gold = gs.players.first().map_or(0, |p| p.resources.gold);
    commands.spawn((
        Text::new(format!("Золото: {gold}")),
        TextFont {
            font: font.clone(),
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.85, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(40.0),
            ..default()
        },
        GoldText,
    ));

    commands.spawn((
        Text::new(format!("День {}", gs.current_day)),
        TextFont {
            font: font.clone(),
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(12.0),
            top: Val::Px(12.0),
            ..default()
        },
        DayText,
    ));

    commands.spawn((
        Text::new("WASD / стрелки — ход | ЛКМ — кликнуть клетку | Space — завершить ход"),
        TextFont {
            font,
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            bottom: Val::Px(12.0),
            ..default()
        },
    ));
}
