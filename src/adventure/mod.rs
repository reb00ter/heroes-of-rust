mod input;
mod render;
mod sync;

use bevy::prelude::*;

use crate::GameScreen;
use crate::core::hero::HeroId;
use crate::core::map::{MapObject, Position};
use crate::core::player::TownId;

pub use render::build_initial_game_state;

// ---------------------------------------------------------------------------
// Константы отображения
// ---------------------------------------------------------------------------

pub const TILE_SIZE: f32 = 48.0;
pub const MAP_WIDTH: u32 = 20;
pub const MAP_HEIGHT: u32 = 15;

// ---------------------------------------------------------------------------
// Bevy-ресурс, оборачивающий чистое GameState
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct GameStateResource(pub crate::core::state::GameState);

// ---------------------------------------------------------------------------
// Компоненты Bevy
// ---------------------------------------------------------------------------

/// Маркер тайла карты. Поле `pos` используется в Этапе 3+ для взаимодействия с клетками.
#[derive(Component)]
pub struct TileMarker {
    #[allow(dead_code)]
    pub pos: Position,
}

/// Маркер сущности героя.
#[derive(Component)]
pub struct HeroMarker(pub HeroId);

/// Подсветка доступного хода (соседняя проходимая клетка).
#[derive(Component)]
pub struct MovementHighlight;

/// Подсветка клетки под курсором мыши.
#[derive(Component)]
pub struct HoverHighlight;

/// Текстовая метка очков движения.
#[derive(Component)]
pub struct MovementPointsText;

/// Маркер спрайта кучки ресурсов на карте приключений.
#[derive(Component)]
pub struct ResourcePileMarker(pub Position);

/// Маркер спрайта города на карте приключений.
#[derive(Component)]
pub struct TownMarker(#[allow(dead_code)] pub TownId);

/// Маркер спрайта нейтрального отряда на карте приключений.
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

/// Текстовая метка армии героя на карте приключений.
#[derive(Component)]
pub struct ArmyText;

/// Текстовая метка с количеством золота.
#[derive(Component)]
pub struct GoldText;

/// Текстовая метка с текущим днём.
#[derive(Component)]
pub struct DayText;

/// Маркер-ресурс: при следующем входе в Adventure пересоздать спрайты объектов карты.
/// Вставляется при нажатии «Играть снова» из экрана `GameOver`.
#[derive(Resource, Default)]
pub struct NeedsMapReset;

// ---------------------------------------------------------------------------
// Системы сброса карты и проверки победы
// ---------------------------------------------------------------------------

/// При перезапуске из `GameOver` — пересоздаёт спрайты объектов карты.
/// Запускается первым в `OnEnter(GameScreen::Adventure)`.
#[allow(clippy::needless_pass_by_value)]
fn reset_map_on_restart(
    mut commands: Commands,
    reset: Option<Res<NeedsMapReset>>,
    game_state: Res<GameStateResource>,
    pile_q: Query<Entity, With<ResourcePileMarker>>,
    neutral_q: Query<Entity, With<NeutralArmyMarker>>,
    hero_q: Query<Entity, With<HeroMarker>>,
) {
    let Some(_) = reset else {
        return;
    };
    commands.remove_resource::<NeedsMapReset>();

    // Удалить старые спрайты
    for e in &pile_q {
        commands.entity(e).despawn();
    }
    for e in &neutral_q {
        commands.entity(e).despawn();
    }
    for e in &hero_q {
        commands.entity(e).despawn();
    }

    // Пересоздать из сброшенного GameState
    let gs = &game_state.0;
    #[allow(clippy::cast_possible_wrap)]
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let pos = Position::new(x as i32, y as i32);
            match gs.map.get(pos).and_then(|t| t.object.as_ref()) {
                Some(MapObject::ResourcePile(_)) => {
                    let world = grid_to_world(pos);
                    commands.spawn((
                        Sprite {
                            color: Color::srgb(0.95, 0.75, 0.10),
                            custom_size: Some(Vec2::splat(TILE_SIZE * 0.5)),
                            ..default()
                        },
                        Transform::from_xyz(world.x, world.y, 1.0),
                        ResourcePileMarker(pos),
                    ));
                }
                Some(MapObject::NeutralArmy(_)) => {
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
                _ => {}
            }
        }
    }

    // Пересоздать героя
    if let Some(hero) = gs.heroes.first() {
        let world = grid_to_world(hero.position);
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
}

/// Если после боя установлен `GameOverResult` — немедленно переходить на `GameOver`.
/// Запускается вторым в `OnEnter(GameScreen::Adventure)`, после `reset_map_on_restart`.
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

/// Переводит позицию в сетке в мировые координаты Bevy (центр тайла).
/// Карта центрируется вокруг начала координат.
///
/// # Precision
/// `MAP_WIDTH` (16) и `MAP_HEIGHT` (12) точно представимы в f32; касты безопасны.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn grid_to_world(pos: Position) -> Vec2 {
    let offset_x = -(MAP_WIDTH as f32 - 1.0) * TILE_SIZE / 2.0;
    let offset_y = (MAP_HEIGHT as f32 - 1.0) * TILE_SIZE / 2.0;
    Vec2::new(
        pos.x as f32 * TILE_SIZE + offset_x,
        offset_y - pos.y as f32 * TILE_SIZE,
    )
}

/// Переводит мировые координаты Bevy в позицию в сетке карты.
///
/// # Truncation
/// После `round()` значение гарантированно в диапазоне тайлов; усечение безопасно.
#[must_use]
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
pub fn world_to_grid(world: Vec2) -> Position {
    let offset_x = -(MAP_WIDTH as f32 - 1.0) * TILE_SIZE / 2.0;
    let offset_y = (MAP_HEIGHT as f32 - 1.0) * TILE_SIZE / 2.0;
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
        app.insert_resource(GameStateResource(build_initial_game_state()))
            .init_resource::<ShowBanner>()
            .add_systems(Startup, render::startup_setup)
            .add_systems(
                OnEnter(GameScreen::Adventure),
                (
                    reset_map_on_restart,
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
                    sync::sync_hero_transform,
                    sync::sync_resource_piles,
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

    #[test]
    fn round_trip_center() {
        let pos = Position::new(7, 5);
        let world = grid_to_world(pos);
        let back = world_to_grid(world);
        assert_eq!(back, pos);
    }

    #[test]
    fn round_trip_origin() {
        let pos = Position::new(0, 0);
        let world = grid_to_world(pos);
        let back = world_to_grid(world);
        assert_eq!(back, pos);
    }

    #[test]
    fn round_trip_far_corner() {
        let pos = Position::new(19, 14);
        let world = grid_to_world(pos);
        let back = world_to_grid(world);
        assert_eq!(back, pos);
    }
}
