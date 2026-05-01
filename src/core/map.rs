#![allow(dead_code)]

use crate::core::hero::Army;
use crate::core::player::TownId;
use crate::core::resources::ResourceBag;

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
    tiles: Vec<Tile>,
}

impl AdventureMap {
    /// Создаёт карту размером `width × height`, заполненную `Ground`-тайлами.
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        let tiles = (0..size).map(|_| Tile::ground()).collect();
        Self {
            width,
            height,
            tiles,
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
}
