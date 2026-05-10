mod input;
mod render;
mod sync;

use bevy::prelude::*;

use crate::GameScreen;
use crate::core::hero::{Hero, HeroId};
use crate::core::map::Position;
use crate::core::player::TownId;

// ---------------------------------------------------------------------------
// Константы отображения
// ---------------------------------------------------------------------------

pub const TILE_SIZE: f32 = 48.0;

// ---------------------------------------------------------------------------
// Bevy-ресурс, оборачивающий чистое GameState
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct GameStateResource(pub crate::core::state::GameState);

// ---------------------------------------------------------------------------
// Компоненты Bevy
// ---------------------------------------------------------------------------

/// Маркер тайла карты.
#[derive(Component)]
pub struct TileMarker {
    #[allow(dead_code)]
    pub pos: Position,
}

/// Маркер сущности героя.
#[derive(Component)]
pub struct HeroMarker(pub HeroId);

/// Подсветка доступного хода.
#[derive(Component)]
pub struct MovementHighlight;

/// Подсветка клетки под курсором мыши.
#[derive(Component)]
pub struct HoverHighlight;

/// Текстовая метка очков движения.
#[derive(Component)]
pub struct MovementPointsText;

/// Маркер спрайта кучки ресурсов.
#[derive(Component)]
pub struct ResourcePileMarker(pub Position);

/// Маркер спрайта города.
#[derive(Component)]
pub struct TownMarker(#[allow(dead_code)] pub TownId);

/// Маркер спрайта нейтрального отряда.
#[derive(Component)]
pub struct NeutralArmyMarker(pub Position);

// ---------------------------------------------------------------------------
// Баннер результата боя
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BannerKind {
    Victory,
}

/// Ресурс-сигнал: показать баннер при следующем входе в Adventure.
#[derive(Resource, Default)]
pub struct ShowBanner(pub Option<BannerKind>);

/// Текстовая метка армии героя.
#[derive(Component)]
pub struct ArmyText;

/// Текстовая метка с количеством золота.
#[derive(Component)]
pub struct GoldText;

/// Текстовая метка с текущим днём.
#[derive(Component)]
pub struct DayText;

/// Подсказка управления внизу экрана.
#[derive(Component)]
pub struct HintText;

// ---------------------------------------------------------------------------
// Spawn-хелперы
// ---------------------------------------------------------------------------

pub(super) fn spawn_gold_pile(
    commands: &mut Commands,
    asset_server: &AssetServer,
    pos: Position,
    map_w: u32,
    map_h: u32,
) {
    let world = grid_to_world(pos, map_w, map_h);
    let tex: Handle<Image> = asset_server.load("sprites/gold_pile.png");
    commands.spawn((
        Sprite {
            image: tex,
            custom_size: Some(Vec2::splat(TILE_SIZE * 0.9)),
            ..default()
        },
        Transform::from_xyz(world.x, world.y, 1.0),
        ResourcePileMarker(pos),
    ));
}

pub(super) fn spawn_neutral_army(commands: &mut Commands, pos: Position, map_w: u32, map_h: u32) {
    let world = grid_to_world(pos, map_w, map_h);
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

pub(super) fn spawn_hero(commands: &mut Commands, hero: &Hero, map_w: u32, map_h: u32) {
    let world = grid_to_world(hero.position, map_w, map_h);
    commands.spawn((
        Sprite {
            color: Color::srgb(0.95, 0.80, 0.10),
            custom_size: Some(Vec2::splat(TILE_SIZE * 0.65)),
            ..default()
        },
        Transform::from_xyz(world.x, world.y, 4.0),
        HeroMarker(hero.id),
    ));
}

// ---------------------------------------------------------------------------
// Системы
// ---------------------------------------------------------------------------

/// Загружает карту из `GameStartConfig` при первом входе в Adventure (старт/рестарт).
/// Если конфига нет — возврат из Town/Battle, ничего не делать.
#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
fn load_map_from_config(
    mut commands: Commands,
    config: Option<Res<crate::GameStartConfig>>,
    mut game_state: ResMut<GameStateResource>,
    tile_q: Query<Entity, With<TileMarker>>,
    town_q: Query<Entity, With<TownMarker>>,
    mp_q: Query<Entity, With<MovementPointsText>>,
    gold_q: Query<Entity, With<GoldText>>,
    day_q: Query<Entity, With<DayText>>,
    army_q: Query<Entity, With<ArmyText>>,
    hint_q: Query<Entity, With<HintText>>,
    asset_server: Res<AssetServer>,
) {
    let Some(config) = config else {
        return;
    };

    let mut gs = crate::data::load_map(&config.map_path, "assets/data/units.ron");
    if let Some(hero) = gs.heroes.first_mut() {
        hero.name.clone_from(&config.hero_name);
    }
    game_state.0 = gs;

    // Очистить старые статические тайлы и UI
    for e in &tile_q {
        commands.entity(e).despawn();
    }
    for e in &town_q {
        commands.entity(e).despawn();
    }
    for e in &mp_q {
        commands.entity(e).despawn();
    }
    for e in &gold_q {
        commands.entity(e).despawn();
    }
    for e in &day_q {
        commands.entity(e).despawn();
    }
    for e in &army_q {
        commands.entity(e).despawn();
    }
    for e in &hint_q {
        commands.entity(e).despawn();
    }

    // Заспавнить новые тайлы и UI
    render::respawn_map_objects(&mut commands, &game_state.0);
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    render::spawn_adventure_ui(&mut commands, &game_state.0, font);

    commands.remove_resource::<crate::GameStartConfig>();
}

/// Если после боя установлен `GameOverResult` — немедленно переходить на `GameOver`.
#[allow(clippy::needless_pass_by_value)]
fn check_game_over(
    game_over: Option<Res<crate::GameOverResult>>,
    mut next_state: ResMut<NextState<GameScreen>>,
) {
    if game_over.is_some() {
        next_state.set(GameScreen::GameOver);
    }
}

// ---------------------------------------------------------------------------
// Вспомогательные функции конвертации координат
// ---------------------------------------------------------------------------

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn grid_to_world(pos: Position, map_w: u32, map_h: u32) -> Vec2 {
    let offset_x = -(map_w as f32 - 1.0) * TILE_SIZE / 2.0;
    let offset_y = (map_h as f32 - 1.0) * TILE_SIZE / 2.0;
    Vec2::new(
        pos.x as f32 * TILE_SIZE + offset_x,
        offset_y - pos.y as f32 * TILE_SIZE,
    )
}

#[must_use]
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
pub fn world_to_grid(world: Vec2, map_w: u32, map_h: u32) -> Position {
    let offset_x = -(map_w as f32 - 1.0) * TILE_SIZE / 2.0;
    let offset_y = (map_h as f32 - 1.0) * TILE_SIZE / 2.0;
    Position::new(
        ((world.x - offset_x) / TILE_SIZE).round() as i32,
        ((offset_y - world.y) / TILE_SIZE).round() as i32,
    )
}

// ---------------------------------------------------------------------------
// Плагин
// ---------------------------------------------------------------------------

pub struct AdventurePlugin;

impl Plugin for AdventurePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameStateResource(render::build_initial_game_state()))
            .init_resource::<ShowBanner>()
            .add_systems(Startup, render::startup_setup)
            .add_systems(
                OnEnter(GameScreen::Adventure),
                (
                    load_map_from_config,
                    check_game_over,
                    render::show_result_banner,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    input::keyboard_input,
                    input::mouse_click_input,
                    input::handle_end_turn,
                    input::update_hover_highlight,
                    sync::sync_map_objects,
                    sync::update_available_moves,
                    sync::update_movement_ui,
                    sync::update_resource_ui,
                    sync::update_day_ui,
                    sync::update_army_ui,
                    render::tick_banner,
                )
                    .chain()
                    .run_if(in_state(GameScreen::Adventure)),
            );
    }
}

// ---------------------------------------------------------------------------
// Тесты конвертации координат
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const W: u32 = 20;
    const H: u32 = 15;

    #[test]
    fn round_trip_center() {
        let pos = Position::new(7, 5);
        let world = grid_to_world(pos, W, H);
        let back = world_to_grid(world, W, H);
        assert_eq!(back, pos);
    }

    #[test]
    fn round_trip_origin() {
        let pos = Position::new(0, 0);
        let world = grid_to_world(pos, W, H);
        let back = world_to_grid(world, W, H);
        assert_eq!(back, pos);
    }

    #[test]
    fn round_trip_far_corner() {
        let pos = Position::new(19, 14);
        let world = grid_to_world(pos, W, H);
        let back = world_to_grid(world, W, H);
        assert_eq!(back, pos);
    }
}
