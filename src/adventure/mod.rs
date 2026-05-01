mod input;
mod render;
mod sync;

use bevy::prelude::*;

use crate::core::hero::HeroId;
use crate::core::map::Position;

pub use render::build_initial_game_state;

// ---------------------------------------------------------------------------
// Константы отображения
// ---------------------------------------------------------------------------

pub const TILE_SIZE: f32 = 48.0;
pub const MAP_WIDTH: u32 = 16;
pub const MAP_HEIGHT: u32 = 12;

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
            .add_systems(Startup, render::startup_setup)
            .add_systems(
                Update,
                (
                    input::keyboard_input,
                    input::mouse_click_input,
                    input::update_hover_highlight,
                    sync::sync_hero_transform,
                    sync::update_available_moves,
                    sync::update_movement_ui,
                )
                    .chain(),
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
        let pos = Position::new(15, 11);
        let world = grid_to_world(pos);
        let back = world_to_grid(world);
        assert_eq!(back, pos);
    }
}
