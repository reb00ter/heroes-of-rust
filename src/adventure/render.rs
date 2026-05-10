use bevy::prelude::*;

use crate::core::map::TileKind;
use crate::core::state::GameState;

use crate::core::map::VisibilityState;

use super::{
    ArmyText, BannerKind, DayText, FogOverlay, GoldText, GridLine, HintText, HoverHighlight,
    MovementPointsText, ShowBanner, TILE_SIZE, TileMarker, TownMarker, grid_to_world,
};

// ---------------------------------------------------------------------------
// Цвета тайлов и объектов
// ---------------------------------------------------------------------------

const COLOR_GROUND: Color = Color::srgb(0.13, 0.32, 0.13);
const COLOR_OBSTACLE: Color = Color::srgb(0.28, 0.28, 0.28);
const COLOR_WATER: Color = Color::srgb(0.08, 0.20, 0.50);
const COLOR_HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 0.30);

fn fog_color(state: VisibilityState) -> Color {
    match state {
        VisibilityState::Unexplored => Color::srgba(0.0, 0.0, 0.0, 1.0),
        VisibilityState::Visited => Color::srgba(0.0, 0.0, 0.0, 0.55),
        VisibilityState::Visible => Color::srgba(0.0, 0.0, 0.0, 0.0),
    }
}

// ---------------------------------------------------------------------------
// Стартовое состояние игры
// ---------------------------------------------------------------------------

/// Делегирует построение состояния загрузчику карт.
pub fn build_initial_game_state() -> GameState {
    crate::data::load_map("assets/maps/default.ron", "assets/data/units.ron")
}

// ---------------------------------------------------------------------------
// Startup-система: только камера и hover-highlight
// ---------------------------------------------------------------------------

#[allow(clippy::needless_pass_by_value)]
pub fn startup_setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Hover-highlight (изначально скрыт)
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
}

// ---------------------------------------------------------------------------
// Статические объекты карты (тайлы и города)
// ---------------------------------------------------------------------------

/// Спавнит линии сетки поверх тайлов (Z=0.5, под туманом Z=2).
#[allow(clippy::cast_precision_loss)]
fn spawn_grid_lines(commands: &mut Commands, map_w: u32, map_h: u32) {
    let color = Color::srgba(1.0, 1.0, 1.0, 0.07);
    let total_w = map_w as f32 * TILE_SIZE;
    let total_h = map_h as f32 * TILE_SIZE;
    let x_left = -total_w / 2.0;
    let y_top = total_h / 2.0;

    for i in 0..=map_w {
        let x = x_left + i as f32 * TILE_SIZE;
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(1.0, total_h)),
                ..default()
            },
            Transform::from_xyz(x, 0.0, 0.5),
            GridLine,
        ));
    }
    for j in 0..=map_h {
        let y = y_top - j as f32 * TILE_SIZE;
        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(total_w, 1.0)),
                ..default()
            },
            Transform::from_xyz(0.0, y, 0.5),
            GridLine,
        ));
    }
}

/// Спавнит статические тайлы карты и города.
/// Вызывается из `load_map_from_config` при каждом старте новой игры.
pub(super) fn respawn_map_objects(commands: &mut Commands, gs: &GameState) {
    let map_w = gs.map.width;
    let map_h = gs.map.height;

    // --- Тайлы карты (Z=0) ---
    for y in 0..map_h {
        for x in 0..map_w {
            #[allow(clippy::cast_possible_wrap)]
            let pos = crate::core::map::Position::new(x as i32, y as i32);
            let world = grid_to_world(pos, map_w, map_h);

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
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                },
                Transform::from_xyz(world.x, world.y, 0.0),
                TileMarker { pos },
            ));

            // Туман войны (Z=2)
            let vis = gs
                .map
                .get_visibility(pos)
                .unwrap_or(VisibilityState::Unexplored);
            commands.spawn((
                Sprite {
                    color: fog_color(vis),
                    custom_size: Some(Vec2::splat(TILE_SIZE - 1.0)),
                    ..default()
                },
                Transform::from_xyz(world.x, world.y, 2.0),
                FogOverlay { pos },
            ));
        }
    }

    // --- Сетка (Z=0.5) ---
    spawn_grid_lines(commands, map_w, map_h);

    // --- Города (Z=1, статика) ---
    for town in &gs.towns {
        let world = grid_to_world(town.position, map_w, map_h);
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
}

/// Спавнит UI-панель приключения (MP, золото, день, армия, подсказка).
/// Вызывается из `load_map_from_config` при каждом старте новой игры.
pub(super) fn spawn_adventure_ui(commands: &mut Commands, gs: &GameState, font: Handle<Font>) {
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
        HintText,
    ));
}

// ---------------------------------------------------------------------------
// Баннер результата боя
// ---------------------------------------------------------------------------

#[derive(Component)]
pub struct BannerRoot;

#[derive(Component)]
pub struct BannerTimer(pub Timer);

/// Спавнит баннер «Победа!» если `ShowBanner` установлен.
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
    use crate::core::map::{MapObject, Position};

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
