use bevy::prelude::*;

use crate::core::hero::{Army, Hero, HeroId, UnitStack, UnitType};
use crate::core::map::{AdventureMap, MapObject, TileKind};
use crate::core::player::{Player, PlayerId, Town, TownId};
use crate::core::resources::ResourceBag;
use crate::core::state::GameState;

use super::{
    ArmyText, BannerKind, DayText, GameStateResource, GoldText, HeroMarker, HoverHighlight,
    MAP_HEIGHT, MAP_WIDTH, MovementPointsText, NeutralArmyMarker, ResourcePileMarker, ShowBanner,
    TILE_SIZE, TileMarker, TownMarker, grid_to_world,
};

// ---------------------------------------------------------------------------
// Цвета тайлов и объектов
// ---------------------------------------------------------------------------

const COLOR_GROUND: Color = Color::srgb(0.20, 0.55, 0.20);
const COLOR_OBSTACLE: Color = Color::srgb(0.45, 0.45, 0.45);
const COLOR_WATER: Color = Color::srgb(0.15, 0.35, 0.80);
const COLOR_HERO: Color = Color::srgb(0.95, 0.80, 0.10);
const COLOR_HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 0.30);

// ---------------------------------------------------------------------------
// Стартовое состояние игры
// ---------------------------------------------------------------------------

/// Создаёт стартовое состояние игры для вертикального среза: карта 20×15 с тремя нейтральными
/// отрядами, городом и стартовой армией героя.
#[allow(clippy::too_many_lines)]
pub fn build_initial_game_state() -> GameState {
    let mut map = AdventureMap::new(MAP_WIDTH, MAP_HEIGHT);

    // Группы скал-препятствий
    let obstacles = [
        // Скалы севернее старта
        (3, 0),
        (4, 0),
        (3, 1),
        // Центральный барьер
        (9, 3),
        (9, 4),
        (9, 5),
        // Восточный регион
        (14, 1),
        (14, 2),
        (15, 8),
        (15, 9),
        // Юг
        (6, 12),
        (7, 12),
        (7, 13),
        (11, 11),
        (12, 11),
    ];
    for (x, y) in obstacles {
        if let Some(tile) = map.get_mut(crate::core::map::Position::new(x, y)) {
            tile.kind = TileKind::Obstacle;
        }
    }

    // Вода по углам карты
    let water = [
        (0, 13),
        (0, 14),
        (1, 14),
        (2, 14),
        (19, 0),
        (19, 1),
        (18, 0),
        (18, 14),
        (19, 14),
        (19, 13),
    ];
    for (x, y) in water {
        if let Some(tile) = map.get_mut(crate::core::map::Position::new(x, y)) {
            tile.kind = TileKind::Water;
        }
    }

    // Кучки золота на карте
    let gold_piles: [(i32, i32, u32); 5] = [
        (2, 5, 150),
        (7, 1, 200),
        (10, 9, 250),
        (15, 3, 300),
        (18, 12, 200),
    ];
    for (x, y, amount) in gold_piles {
        if let Some(tile) = map.get_mut(crate::core::map::Position::new(x, y)) {
            tile.object = Some(MapObject::ResourcePile(ResourceBag::gold(amount)));
        }
    }

    // Типы существ для найма в городе
    let peasant = UnitType {
        name: "Крестьянин".to_string(),
        damage_per_unit: 1,
        hp: 5,
        cost: ResourceBag::gold(25),
    };
    let swordsman = UnitType {
        name: "Мечник".to_string(),
        damage_per_unit: 3,
        hp: 10,
        cost: ResourceBag::gold(75),
    };

    // Стартовая армия героя — 5 крестьян
    let peasant_starter = UnitType {
        name: "Крестьянин".to_string(),
        damage_per_unit: 1,
        hp: 5,
        cost: ResourceBag::gold(25),
    };

    // Город ближе к старту
    let mut town = Town::new(
        TownId(0),
        crate::core::map::Position::new(4, 2),
        ResourceBag::gold(250),
    );
    town.available_recruits = vec![(peasant, 10), (swordsman, 5)];
    town.daily_growth = vec![5, 2];

    // Объект города на тайле карты
    if let Some(tile) = map.get_mut(crate::core::map::Position::new(4, 2)) {
        tile.object = Some(MapObject::Town(TownId(0)));
    }

    // Нейтральные отряды трёх уровней сложности
    let goblin = UnitType {
        name: "Гоблин".to_string(),
        damage_per_unit: 2,
        hp: 5,
        cost: ResourceBag::gold(0),
    };
    let orc = UnitType {
        name: "Орк".to_string(),
        damage_per_unit: 4,
        hp: 10,
        cost: ResourceBag::gold(0),
    };
    let troll = UnitType {
        name: "Тролль".to_string(),
        damage_per_unit: 8,
        hp: 25,
        cost: ResourceBag::gold(0),
    };

    if let Some(tile) = map.get_mut(crate::core::map::Position::new(7, 5)) {
        tile.object = Some(MapObject::NeutralArmy(Army(vec![UnitStack::new(
            goblin, 5,
        )])));
    }
    if let Some(tile) = map.get_mut(crate::core::map::Position::new(13, 6)) {
        tile.object = Some(MapObject::NeutralArmy(Army(vec![UnitStack::new(orc, 4)])));
    }
    if let Some(tile) = map.get_mut(crate::core::map::Position::new(17, 10)) {
        tile.object = Some(MapObject::NeutralArmy(Army(vec![UnitStack::new(troll, 2)])));
    }

    // Герой со стартовой армией
    let mut hero = Hero {
        id: HeroId(0),
        name: "Aldric".to_string(),
        position: crate::core::map::Position::new(1, 1),
        army: Army::new(),
        movement_points: 10,
        movement_points_max: 10,
    };
    let _ = hero.army.add_stack(UnitStack::new(peasant_starter, 5));

    // Игрок
    let mut player = Player::new(PlayerId(0));
    player.hero_ids.push(HeroId(0));
    player.resources = ResourceBag::gold(500);

    let obj_count = obstacles.len() + water.len();
    info!(
        "[ADVENTURE] Game initialized. Map: {}x{}, obstacles+water: {}, gold piles: {}, neutrals: 3. Hero army: [Peasants x5].",
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

#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
pub fn startup_setup(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    asset_server: Res<AssetServer>,
) {
    let gs = &game_state.0;
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    // Спрайт кучки золота — генерируется из SVG при сборке (build.rs + resvg)
    let gold_pile_tex: Handle<Image> = asset_server.load("sprites/gold_pile.png");

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
                        image: gold_pile_tex.clone(),
                        custom_size: Some(Vec2::splat(TILE_SIZE * 0.9)),
                        ..default()
                    },
                    Transform::from_xyz(world.x, world.y, 1.0),
                    ResourcePileMarker(pos),
                ));
            }
        }
    }

    // --- Города (Z=1) ---
    for town in &gs.towns {
        let world = grid_to_world(town.position);
        commands.spawn((
            Sprite {
                color: Color::srgb(0.55, 0.40, 0.80),
                custom_size: Some(Vec2::splat(TILE_SIZE * 0.8)),
                ..default()
            },
            Transform::from_xyz(world.x, world.y, 1.0),
            TownMarker(town.id),
        ));
    }

    // --- Нейтральные отряды (Z=1) ---
    #[allow(clippy::cast_possible_wrap)]
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let pos = crate::core::map::Position::new(x as i32, y as i32);
            if matches!(
                gs.map.get(pos).and_then(|t| t.object.as_ref()),
                Some(MapObject::NeutralArmy(_))
            ) {
                let world = grid_to_world(pos);
                commands.spawn((
                    Sprite {
                        color: Color::srgb(0.85, 0.15, 0.15),
                        custom_size: Some(Vec2::splat(TILE_SIZE * 0.75)),
                        ..default()
                    },
                    Transform::from_xyz(world.x, world.y, 1.0),
                    NeutralArmyMarker(pos),
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
        Text::new("Армия: (пусто)"),
        TextFont {
            font: font.clone(),
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(12.0),
            top: Val::Px(68.0),
            ..default()
        },
        ArmyText,
    ));

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

// ---------------------------------------------------------------------------
// Баннер результата боя (5.11)
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct BannerRoot;

#[derive(Component)]
pub struct BannerTimer(pub Timer);

/// Спавнит баннер «Победа!» / «Поражение» если `ShowBanner` установлен.
/// Запускается в `OnEnter(GameScreen::Adventure)`.
#[allow(clippy::needless_pass_by_value)]
pub fn show_result_banner(
    mut commands: Commands,
    mut show_banner: ResMut<ShowBanner>,
    asset_server: Res<AssetServer>,
) {
    let Some(kind) = show_banner.0.take() else {
        return;
    };

    let (text, color) = match kind {
        BannerKind::Victory => ("Победа!", Color::srgb(0.15, 0.90, 0.15)),
    };

    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");

    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            GlobalZIndex(50),
            BannerRoot,
            BannerTimer(Timer::from_seconds(2.0, TimerMode::Once)),
        ))
        .id();

    let label = commands
        .spawn((
            Text::new(text),
            TextFont {
                font,
                font_size: 80.0,
                ..default()
            },
            TextColor(color),
        ))
        .id();
    commands.entity(root).add_child(label);
}

/// Убирает баннер по истечении 2 секунд или по клику ЛКМ.
#[allow(clippy::needless_pass_by_value)]
pub fn tick_banner(
    mut commands: Commands,
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut banner_q: Query<(Entity, &mut BannerTimer), With<BannerRoot>>,
) {
    for (entity, mut timer) in &mut banner_q {
        let just_done = timer.0.tick(time.delta()).just_finished();
        if just_done || mouse.just_pressed(MouseButton::Left) {
            commands.entity(entity).despawn();
        }
    }
}

// ---------------------------------------------------------------------------
// Тесты
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::map::Position;

    #[test]
    fn three_neutral_armies_on_map() {
        let gs = build_initial_game_state();
        assert_eq!(gs.count_neutral_armies(), 3);
    }

    #[test]
    fn neutral_armies_placed() {
        let gs = build_initial_game_state();

        let goblin_tile = gs.map.get(Position::new(7, 5)).expect("tile (7,5) exists");
        match goblin_tile.object.as_ref().expect("object on (7,5)") {
            MapObject::NeutralArmy(army) => {
                assert_eq!(army.0.len(), 1);
                assert_eq!(army.0[0].count, 5);
                assert_eq!(army.0[0].unit_type.name, "Гоблин");
                assert_eq!(army.0[0].unit_type.damage_per_unit, 2);
                assert_eq!(army.0[0].unit_type.hp, 5);
            }
            other => panic!("expected NeutralArmy at (7,5), got {other:?}"),
        }

        let orc_tile = gs
            .map
            .get(Position::new(13, 6))
            .expect("tile (13,6) exists");
        match orc_tile.object.as_ref().expect("object on (13,6)") {
            MapObject::NeutralArmy(army) => {
                assert_eq!(army.0.len(), 1);
                assert_eq!(army.0[0].count, 4);
                assert_eq!(army.0[0].unit_type.name, "Орк");
                assert_eq!(army.0[0].unit_type.damage_per_unit, 4);
                assert_eq!(army.0[0].unit_type.hp, 10);
            }
            other => panic!("expected NeutralArmy at (13,6), got {other:?}"),
        }

        let troll_tile = gs
            .map
            .get(Position::new(17, 10))
            .expect("tile (17,10) exists");
        match troll_tile.object.as_ref().expect("object on (17,10)") {
            MapObject::NeutralArmy(army) => {
                assert_eq!(army.0.len(), 1);
                assert_eq!(army.0[0].count, 2);
                assert_eq!(army.0[0].unit_type.name, "Тролль");
                assert_eq!(army.0[0].unit_type.damage_per_unit, 8);
                assert_eq!(army.0[0].unit_type.hp, 25);
            }
            other => panic!("expected NeutralArmy at (17,10), got {other:?}"),
        }
    }

    #[test]
    fn neutral_army_blocks_movement() {
        let gs = build_initial_game_state();
        assert!(!gs.map.is_passable(Position::new(7, 5)));
        assert!(!gs.map.is_passable(Position::new(13, 6)));
        assert!(!gs.map.is_passable(Position::new(17, 10)));
    }

    #[test]
    fn hero_has_starter_army() {
        let gs = build_initial_game_state();
        let hero = gs.heroes.first().expect("hero exists");
        assert!(!hero.army.0.is_empty(), "hero must start with an army");
        assert_eq!(hero.army.0[0].unit_type.name, "Крестьянин");
        assert_eq!(hero.army.0[0].count, 5);
    }

    #[test]
    fn town_near_start() {
        let gs = build_initial_game_state();
        let town = gs.towns.first().expect("town exists");
        assert_eq!(town.position, Position::new(4, 2));
    }
}
