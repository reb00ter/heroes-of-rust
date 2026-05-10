#![allow(dead_code)]

use crate::core::hero::Army;
use crate::core::player::TownId;
use crate::core::resources::ResourceBag;

// ---------------------------------------------------------------------------
// VisibilityState
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibilityState {
    Unexplored,
    Visited,
    Visible,
}

// ---------------------------------------------------------------------------
// Position
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    #[must_use]
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

// ---------------------------------------------------------------------------
// TileKind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TileKind {
    Ground,
    Obstacle,
    Water,
}

// ---------------------------------------------------------------------------
// MapObject
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum MapObject {
    ResourcePile(ResourceBag),
    Town(TownId),
    NeutralArmy(Army),
}

// ---------------------------------------------------------------------------
// Tile
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Tile {
    pub kind: TileKind,
    pub object: Option<MapObject>,
}

impl Tile {
    #[must_use]
    pub fn ground() -> Self {
        Self {
            kind: TileKind::Ground,
            object: None,
        }
    }
}

// ---------------------------------------------------------------------------
// AdventureMap
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AdventureMap {
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<Tile>,
    pub visibility: Vec<VisibilityState>,
}

impl AdventureMap {
    /// Создаёт карту размером `width × height`, заполненную `Ground`-тайлами.
    /// Все тайлы изначально `Unexplored`.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        let tiles = (0..size).map(|_| Tile::ground()).collect();
        let visibility = vec![VisibilityState::Unexplored; size];
        Self {
            width,
            height,
            tiles,
            visibility,
        }
    }

    fn idx(&self, pos: Position) -> Option<usize> {
        let x = u32::try_from(pos.x).ok()?;
        let y = u32::try_from(pos.y).ok()?;
        if x >= self.width || y >= self.height {
            return None;
        }
        Some((y * self.width + x) as usize)
    }

    #[must_use]
    pub fn get(&self, pos: Position) -> Option<&Tile> {
        self.idx(pos).map(|i| &self.tiles[i])
    }

    pub fn get_mut(&mut self, pos: Position) -> Option<&mut Tile> {
        self.idx(pos).map(|i| &mut self.tiles[i])
    }

    /// Клетка проходима, если она `Ground` и на ней нет `NeutralArmy`.
    #[must_use]
    pub fn is_passable(&self, pos: Position) -> bool {
        match self.get(pos) {
            Some(tile) => {
                tile.kind == TileKind::Ground
                    && !matches!(tile.object, Some(MapObject::NeutralArmy(_)))
            }
            None => false,
        }
    }

    #[must_use]
    pub fn get_visibility(&self, pos: Position) -> Option<VisibilityState> {
        self.idx(pos).map(|i| self.visibility[i])
    }

    pub fn set_visibility(&mut self, pos: Position, state: VisibilityState) {
        if let Some(i) = self.idx(pos) {
            self.visibility[i] = state;
        }
    }

    /// Возвращает до 4 смежных клеток (вверх, вниз, влево, вправо) в пределах карты.
    #[must_use]
    pub fn neighbors(&self, pos: Position) -> Vec<Position> {
        [
            Position::new(pos.x, pos.y - 1),
            Position::new(pos.x, pos.y + 1),
            Position::new(pos.x - 1, pos.y),
            Position::new(pos.x + 1, pos.y),
        ]
        .into_iter()
        .filter(|p| self.idx(*p).is_some())
        .collect()
    }
}

// ---------------------------------------------------------------------------
// Логика видимости
// ---------------------------------------------------------------------------

/// Обновляет туман войны после перемещения героя.
///
/// 1. Все `Visible` → `Visited` (герой покинул ту зону).
/// 2. Все клетки с евклидовым расстоянием ≤ `sight_range` → `Visible`.
pub fn update_visibility(map: &mut AdventureMap, hero_pos: Position, sight_range: u32) {
    let size = (map.width * map.height) as usize;
    // Шаг 1: сбросить текущую видимость в Visited
    for v in &mut map.visibility[..size] {
        if *v == VisibilityState::Visible {
            *v = VisibilityState::Visited;
        }
    }
    // Шаг 2: открыть круг вокруг героя (евклидово расстояние)
    #[allow(clippy::cast_possible_wrap)]
    let r = sight_range as i32;
    let r2 = sight_range * sight_range;
    for dy in -r..=r {
        for dx in -r..=r {
            #[allow(clippy::cast_sign_loss)]
            if (dx * dx + dy * dy) as u32 <= r2 {
                let pos = Position::new(hero_pos.x + dx, hero_pos.y + dy);
                map.set_visibility(pos, VisibilityState::Visible);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_get_valid() {
        let map = AdventureMap::new(5, 5);
        assert!(map.get(Position::new(0, 0)).is_some());
        assert!(map.get(Position::new(4, 4)).is_some());
    }

    #[test]
    fn map_get_out_of_bounds() {
        let map = AdventureMap::new(5, 5);
        assert!(map.get(Position::new(5, 0)).is_none());
        assert!(map.get(Position::new(-1, 0)).is_none());
    }

    #[test]
    fn ground_is_passable() {
        let map = AdventureMap::new(3, 3);
        assert!(map.is_passable(Position::new(1, 1)));
    }

    #[test]
    fn obstacle_not_passable() {
        let mut map = AdventureMap::new(3, 3);
        map.get_mut(Position::new(1, 1)).unwrap().kind = TileKind::Obstacle;
        assert!(!map.is_passable(Position::new(1, 1)));
    }

    #[test]
    fn neighbors_corner() {
        let map = AdventureMap::new(3, 3);
        let n = map.neighbors(Position::new(0, 0));
        assert_eq!(n.len(), 2);
    }

    #[test]
    fn neighbors_center() {
        let map = AdventureMap::new(3, 3);
        let n = map.neighbors(Position::new(1, 1));
        assert_eq!(n.len(), 4);
    }

    // --- Туман войны ---

    #[test]
    fn initial_fog_all_unexplored() {
        let map = AdventureMap::new(5, 5);
        for y in 0..5_i32 {
            for x in 0..5_i32 {
                assert_eq!(
                    map.get_visibility(Position::new(x, y)),
                    Some(VisibilityState::Unexplored)
                );
            }
        }
    }

    #[test]
    fn update_visibility_reveals_area() {
        let mut map = AdventureMap::new(10, 10);
        update_visibility(&mut map, Position::new(5, 5), 2);
        // Клетки строго внутри круга r=2 — Visible
        assert_eq!(
            map.get_visibility(Position::new(5, 5)),
            Some(VisibilityState::Visible)
        );
        assert_eq!(
            map.get_visibility(Position::new(5 + 1, 5)),
            Some(VisibilityState::Visible)
        );
        assert_eq!(
            map.get_visibility(Position::new(5, 5 + 2)),
            Some(VisibilityState::Visible)
        );
        // Угол квадрата (dx=2,dy=2) имеет расстояние ~2.83 > 2 — Unexplored
        assert_eq!(
            map.get_visibility(Position::new(5 + 2, 5 + 2)),
            Some(VisibilityState::Unexplored)
        );
        // Клетка за пределами радиуса — Unexplored
        assert_eq!(
            map.get_visibility(Position::new(5 + 3, 5)),
            Some(VisibilityState::Unexplored)
        );
    }

    #[test]
    fn moving_hero_marks_old_area_visited() {
        let mut map = AdventureMap::new(20, 20);
        update_visibility(&mut map, Position::new(2, 2), 2);
        // Переместить героя далеко
        update_visibility(&mut map, Position::new(15, 15), 2);
        // Старая зона — Visited (была Visible)
        assert_eq!(
            map.get_visibility(Position::new(2, 2)),
            Some(VisibilityState::Visited)
        );
        // Новая зона — Visible
        assert_eq!(
            map.get_visibility(Position::new(15, 15)),
            Some(VisibilityState::Visible)
        );
    }

    #[test]
    fn visited_tiles_not_reset() {
        let mut map = AdventureMap::new(20, 20);
        update_visibility(&mut map, Position::new(2, 2), 2);
        // Переместить дважды
        update_visibility(&mut map, Position::new(10, 10), 2);
        update_visibility(&mut map, Position::new(15, 15), 2);
        // Первая зона осталась Visited — не сбросилась в Unexplored
        assert_eq!(
            map.get_visibility(Position::new(2, 2)),
            Some(VisibilityState::Visited)
        );
    }
}
