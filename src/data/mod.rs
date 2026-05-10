use bevy::log::{info, warn};

use crate::core::hero::{Army, Hero, HeroId, UnitStack, UnitType};
use crate::core::map::{AdventureMap, MapObject, Position, TileKind};
use crate::core::player::{Player, PlayerId, Town, TownId};
use crate::core::resources::ResourceBag;
use crate::core::state::GameState;

// ---------------------------------------------------------------------------
// Типы справочника существ
// ---------------------------------------------------------------------------

/// Полное описание типа существа — загружается из units.ron.
#[derive(serde::Deserialize, Clone)]
pub struct UnitTypeDef {
    pub id: String,
    pub name: String,
    pub damage: u32,
    pub hp: u32,
    pub cost: u32,
    pub daily_growth: u32,
}

// ---------------------------------------------------------------------------
// Типы карты
// ---------------------------------------------------------------------------

/// Ссылка на существо в карте — только ASCII id и количество.
#[derive(serde::Deserialize, Clone)]
pub struct StackRef {
    pub id: String,
    pub count: u32,
}

/// Корневая структура файла карты.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapDefinition {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub width: u32,
    pub height: u32,
    pub obstacles: Vec<(i32, i32)>,
    pub water: Vec<(i32, i32)>,
    pub resource_piles: Vec<ResourcePileDef>,
    pub towns: Vec<TownDef>,
    pub neutral_armies: Vec<NeutralArmyDef>,
    pub hero: HeroDef,
}

#[derive(serde::Deserialize)]
pub struct ResourcePileDef {
    pub pos: (i32, i32),
    pub gold: u32,
}

#[derive(serde::Deserialize)]
pub struct TownDef {
    pub id: u32,
    pub pos: (i32, i32),
    pub daily_income: u32,
    pub recruits: Vec<StackRef>,
}

#[derive(serde::Deserialize)]
pub struct NeutralArmyDef {
    pub pos: (i32, i32),
    pub units: Vec<StackRef>,
}

#[derive(serde::Deserialize)]
pub struct HeroDef {
    pub pos: (i32, i32),
    pub name: String,
    pub movement_points: u32,
    pub army: Vec<StackRef>,
    pub starting_gold: u32,
}

// ---------------------------------------------------------------------------
// Метаинформация о карте (результат валидации)
// ---------------------------------------------------------------------------

/// Метаданные карты — возвращаются `discover_maps` и `validate_map`.
#[derive(Clone, Debug)]
pub struct MapInfo {
    pub path: String,
    pub name: String,
    pub description: String,
    pub width: u32,
    pub height: u32,
    pub neutral_count: usize,
    pub town_count: usize,
}

// ---------------------------------------------------------------------------
// Публичные функции загрузки
// ---------------------------------------------------------------------------

/// Читает `assets/data/units.ron` и возвращает справочник существ.
pub fn load_units(path: &str) -> Vec<UnitTypeDef> {
    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read units file '{path}': {e}"));
    ron::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse units file '{path}': {e}"))
}

/// Загружает карту из RON-файла и возвращает готовый `GameState`.
pub fn load_map(map_path: &str, units_path: &str) -> GameState {
    let units = load_units(units_path);
    let content = std::fs::read_to_string(map_path)
        .unwrap_or_else(|e| panic!("Failed to read map file '{map_path}': {e}"));
    let def: MapDefinition = ron::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse map file '{map_path}': {e}"));
    map_def_to_game_state(&def, &units)
}

/// Сканирует директорию, валидирует каждый `*.ron` файл.
/// Невалидные файлы логируются через `warn!` и пропускаются.
pub fn discover_maps(maps_dir: &str, units: &[UnitTypeDef]) -> Vec<MapInfo> {
    let Ok(entries) = std::fs::read_dir(maps_dir) else {
        warn!("[DATA] Cannot read maps directory '{maps_dir}'");
        return Vec::new();
    };

    let mut result = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("ron") {
            continue;
        }
        let path_str = path.to_string_lossy().into_owned();
        match validate_map_content_from_file(&path_str, units) {
            Ok(info) => result.push(info),
            Err(reason) => warn!("[DATA] Map '{}' skipped: {}", path_str, reason),
        }
    }
    result
}

// ---------------------------------------------------------------------------
// Валидация карты
// ---------------------------------------------------------------------------

/// Читает файл по пути и валидирует содержимое. Возвращает `MapInfo` или описание ошибки.
fn validate_map_content_from_file(path: &str, units: &[UnitTypeDef]) -> Result<MapInfo, String> {
    let content = std::fs::read_to_string(path).map_err(|e| format!("Cannot read file: {e}"))?;
    validate_map_content(&content, path, units)
}

/// Парсит RON-строку и выполняет семантическую валидацию карты.
fn validate_map_content(
    content: &str,
    path: &str,
    units: &[UnitTypeDef],
) -> Result<MapInfo, String> {
    let def: MapDefinition = ron::from_str(content).map_err(|e| format!("Parse error: {e}"))?;

    if def.name.is_empty() {
        return Err("Map name is empty".to_string());
    }

    // Все id существ должны быть в справочнике
    for army_def in &def.neutral_armies {
        for stack in &army_def.units {
            if !units.iter().any(|u| u.id == stack.id) {
                return Err(format!("Unknown unit: {}", stack.id));
            }
        }
    }
    for town_def in &def.towns {
        for stack in &town_def.recruits {
            if !units.iter().any(|u| u.id == stack.id) {
                return Err(format!("Unknown unit: {}", stack.id));
            }
        }
    }
    for stack in &def.hero.army {
        if !units.iter().any(|u| u.id == stack.id) {
            return Err(format!("Unknown unit: {}", stack.id));
        }
    }

    // Позиции всех объектов — в пределах карты
    #[allow(clippy::cast_possible_wrap)]
    let w = def.width as i32;
    #[allow(clippy::cast_possible_wrap)]
    let h = def.height as i32;
    let in_bounds = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h;

    for pile in &def.resource_piles {
        if !in_bounds(pile.pos.0, pile.pos.1) {
            return Err(format!("ResourcePile out of bounds: {:?}", pile.pos));
        }
    }
    for town_def in &def.towns {
        if !in_bounds(town_def.pos.0, town_def.pos.1) {
            return Err(format!(
                "Town {} out of bounds: {:?}",
                town_def.id, town_def.pos
            ));
        }
    }
    for army_def in &def.neutral_armies {
        if !in_bounds(army_def.pos.0, army_def.pos.1) {
            return Err(format!("NeutralArmy out of bounds: {:?}", army_def.pos));
        }
    }
    let (hx, hy) = def.hero.pos;
    if !in_bounds(hx, hy) {
        return Err(format!("Hero out of bounds: {:?}", def.hero.pos));
    }

    // Объекты не стоят на Obstacle/Water тайлах
    let obstacle_set: std::collections::HashSet<(i32, i32)> =
        def.obstacles.iter().copied().collect();
    let water_set: std::collections::HashSet<(i32, i32)> = def.water.iter().copied().collect();
    let not_passable =
        |x: i32, y: i32| obstacle_set.contains(&(x, y)) || water_set.contains(&(x, y));

    for pile in &def.resource_piles {
        if not_passable(pile.pos.0, pile.pos.1) {
            return Err(format!("ResourcePile on impassable tile: {:?}", pile.pos));
        }
    }
    for town_def in &def.towns {
        if not_passable(town_def.pos.0, town_def.pos.1) {
            return Err(format!(
                "Town {} on impassable tile: {:?}",
                town_def.id, town_def.pos
            ));
        }
    }
    for army_def in &def.neutral_armies {
        if not_passable(army_def.pos.0, army_def.pos.1) {
            return Err(format!(
                "NeutralArmy on impassable tile: {:?}",
                army_def.pos
            ));
        }
    }
    if not_passable(hx, hy) {
        return Err(format!("Hero on impassable tile: {:?}", def.hero.pos));
    }

    Ok(MapInfo {
        path: path.to_string(),
        name: def.name.clone(),
        description: def.description.clone(),
        width: def.width,
        height: def.height,
        neutral_count: def.neutral_armies.len(),
        town_count: def.towns.len(),
    })
}

// ---------------------------------------------------------------------------
// Вспомогательные функции
// ---------------------------------------------------------------------------

/// Ищет `UnitTypeDef` по ASCII `id` и строит `UnitType` для игрового ядра.
/// Паникует, если `id` не найден — ошибка данных, не рантайм.
fn resolve(id: &str, units: &[UnitTypeDef]) -> UnitType {
    let def = units
        .iter()
        .find(|u| u.id == id)
        .unwrap_or_else(|| panic!("Unknown unit id '{id}' — check units.ron"));
    UnitType {
        name: def.name.clone(),
        damage_per_unit: def.damage,
        hp: def.hp,
        cost: ResourceBag::gold(def.cost),
    }
}

fn map_def_to_game_state(def: &MapDefinition, units: &[UnitTypeDef]) -> GameState {
    let mut map = AdventureMap::new(def.width, def.height);

    for &(x, y) in &def.obstacles {
        if let Some(tile) = map.get_mut(Position::new(x, y)) {
            tile.kind = TileKind::Obstacle;
        }
    }

    for &(x, y) in &def.water {
        if let Some(tile) = map.get_mut(Position::new(x, y)) {
            tile.kind = TileKind::Water;
        }
    }

    for pile in &def.resource_piles {
        if let Some(tile) = map.get_mut(Position::new(pile.pos.0, pile.pos.1)) {
            tile.object = Some(MapObject::ResourcePile(ResourceBag::gold(pile.gold)));
        }
    }

    let mut towns: Vec<Town> = Vec::new();
    for town_def in &def.towns {
        let mut town = Town::new(
            TownId(town_def.id),
            Position::new(town_def.pos.0, town_def.pos.1),
            ResourceBag::gold(town_def.daily_income),
        );
        for stack_ref in &town_def.recruits {
            let unit_def = units
                .iter()
                .find(|u| u.id == stack_ref.id)
                .unwrap_or_else(|| panic!("Unknown unit id '{}' in town recruits", stack_ref.id));
            town.available_recruits
                .push((resolve(&stack_ref.id, units), stack_ref.count));
            town.daily_growth.push(unit_def.daily_growth);
        }
        if let Some(tile) = map.get_mut(Position::new(town_def.pos.0, town_def.pos.1)) {
            tile.object = Some(MapObject::Town(TownId(town_def.id)));
        }
        towns.push(town);
    }

    for army_def in &def.neutral_armies {
        let stacks: Vec<UnitStack> = army_def
            .units
            .iter()
            .map(|s| UnitStack::new(resolve(&s.id, units), s.count))
            .collect();
        if let Some(tile) = map.get_mut(Position::new(army_def.pos.0, army_def.pos.1)) {
            tile.object = Some(MapObject::NeutralArmy(Army(stacks)));
        }
    }

    let hero_def = &def.hero;
    let mut hero = Hero {
        id: HeroId(0),
        name: hero_def.name.clone(),
        position: Position::new(hero_def.pos.0, hero_def.pos.1),
        army: Army::new(),
        movement_points: hero_def.movement_points,
        movement_points_max: hero_def.movement_points,
    };
    for stack_ref in &hero_def.army {
        let _ = hero.army.add_stack(UnitStack::new(
            resolve(&stack_ref.id, units),
            stack_ref.count,
        ));
    }

    let mut player = Player::new(PlayerId(0));
    player.hero_ids.push(HeroId(0));
    player.resources = ResourceBag::gold(hero_def.starting_gold);

    info!(
        "[ADVENTURE] Map loaded. Size: {}x{}, obstacles: {}, water: {}, gold piles: {}, neutrals: {}. Hero: \"{}\" at ({},{}).",
        def.width,
        def.height,
        def.obstacles.len(),
        def.water.len(),
        def.resource_piles.len(),
        def.neutral_armies.len(),
        hero_def.name,
        hero_def.pos.0,
        hero_def.pos.1,
    );

    GameState {
        map,
        players: vec![player],
        heroes: vec![hero],
        towns,
        current_day: 1,
        active_player_id: PlayerId(0),
    }
}

// ---------------------------------------------------------------------------
// Тесты
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_units_parses() {
        let units = load_units("assets/data/units.ron");
        assert_eq!(units.len(), 5);
        let peasant = units
            .iter()
            .find(|u| u.id == "peasant")
            .expect("peasant exists");
        assert_eq!(peasant.name, "Крестьянин");
        assert_eq!(peasant.damage, 1);
        assert_eq!(peasant.daily_growth, 5);
    }

    #[test]
    #[should_panic(expected = "Unknown unit id 'dragon'")]
    fn unknown_unit_panics() {
        let units = load_units("assets/data/units.ron");
        let _ = resolve("dragon", &units);
    }

    #[test]
    fn load_default_map_parses() {
        let gs = load_map("assets/maps/default.ron", "assets/data/units.ron");
        assert_eq!(gs.map.width, 20);
        assert_eq!(gs.map.height, 15);
    }

    #[test]
    fn default_map_has_correct_neutrals() {
        let gs = load_map("assets/maps/default.ron", "assets/data/units.ron");
        assert_eq!(gs.count_neutral_armies(), 3);

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
    fn default_map_has_town() {
        let gs = load_map("assets/maps/default.ron", "assets/data/units.ron");
        assert_eq!(gs.towns.len(), 1);
        let town = &gs.towns[0];
        assert_eq!(town.position, Position::new(4, 2));
        assert_eq!(town.income.gold, 250);
        // daily_growth Крестьянина должен быть 5 (из units.ron)
        assert_eq!(town.daily_growth[0], 5);
    }

    #[test]
    fn default_map_hero_start() {
        let gs = load_map("assets/maps/default.ron", "assets/data/units.ron");
        let hero = gs.heroes.first().expect("hero exists");
        assert_eq!(hero.position, Position::new(1, 1));
        assert_eq!(hero.movement_points, 10);
        assert_eq!(hero.army.0.len(), 1);
        assert_eq!(hero.army.0[0].unit_type.name, "Крестьянин");
        assert_eq!(hero.army.0[0].count, 5);
        let gold = gs.players.first().expect("player exists").resources.gold;
        assert_eq!(gold, 500);
    }

    #[test]
    fn unknown_field_in_map_is_error() {
        let bad_ron = r#"(
            name: "Test", width: 5, height: 5,
            obstacles: [], water: [],
            resource_piles: [], towns: [], neutral_armies: [],
            hero: (pos: (0,0), name: "X", movement_points: 5, army: [], starting_gold: 0),
            bogus_field: 42,
        )"#;
        let result = ron::from_str::<MapDefinition>(bad_ron);
        assert!(result.is_err(), "expected parse error for unknown field");
    }

    // --- Новые тесты этапа 8 ---

    #[test]
    fn discover_maps_finds_default() {
        let units = load_units("assets/data/units.ron");
        let maps = discover_maps("assets/maps/", &units);
        assert!(!maps.is_empty(), "должна найти хотя бы одну карту");
        assert!(
            maps.iter().any(|m| m.name == "Равнины начала"),
            "должна найти default.ron"
        );
    }

    #[test]
    fn valid_map_passes_validation() {
        let units = load_units("assets/data/units.ron");
        let info =
            validate_map_content_from_file("assets/maps/default.ron", &units).expect("valid map");
        assert_eq!(info.width, 20);
        assert_eq!(info.height, 15);
        assert!(!info.name.is_empty(), "name must be non-empty");
        assert_eq!(info.neutral_count, 3);
        assert_eq!(info.town_count, 1);
    }

    #[test]
    fn missing_unit_id_fails_validation() {
        let units = load_units("assets/data/units.ron");
        let ron = r#"(
            name: "Bad Map",
            width: 5, height: 5,
            obstacles: [], water: [],
            resource_piles: [],
            towns: [],
            neutral_armies: [(pos: (2,2), units: [(id: "dragon", count: 1)])],
            hero: (pos: (0,0), name: "X", movement_points: 5, army: [], starting_gold: 0),
        )"#;
        let result = validate_map_content(ron, "test", &units);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dragon"));
    }

    #[test]
    fn out_of_bounds_position_fails() {
        let units = load_units("assets/data/units.ron");
        let ron = r#"(
            name: "Bad Map",
            width: 5, height: 5,
            obstacles: [], water: [],
            resource_piles: [(pos: (10,10), gold: 100)],
            towns: [],
            neutral_armies: [],
            hero: (pos: (0,0), name: "X", movement_points: 5, army: [], starting_gold: 0),
        )"#;
        let result = validate_map_content(ron, "test", &units);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("out of bounds"));
    }

    #[test]
    fn object_on_obstacle_fails() {
        let units = load_units("assets/data/units.ron");
        let ron = r#"(
            name: "Bad Map",
            width: 5, height: 5,
            obstacles: [(2,2)], water: [],
            resource_piles: [(pos: (2,2), gold: 100)],
            towns: [],
            neutral_armies: [],
            hero: (pos: (0,0), name: "X", movement_points: 5, army: [], starting_gold: 0),
        )"#;
        let result = validate_map_content(ron, "test", &units);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("impassable"));
    }
}
