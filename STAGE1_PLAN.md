# STAGE1_PLAN.md — Детальный план реализации Этапа 1

**Цель:** реализовать ядро игровой логики без какой-либо графики. Всё покрыто тестами.
**Критерий готовности:** `cargo test` проходит все тесты без запуска окна.

---

## Структура файлов

```
src/
├── main.rs                  ← добавить mod core;
└── core/
    ├── mod.rs               ← корневой модуль, реэкспорты публичных типов
    ├── map.rs               ← карта, позиции, тайлы, объекты
    ├── resources.rs         ← ресурсы и экономика
    ├── hero.rs              ← существа, армия, герой
    ├── player.rs            ← игрок, город, ID-типы
    ├── state.rs             ← GameState
    └── commands.rs          ← команды, события, обработка логики
```

---

## Шаг 1 — `core::map`

### Типы данных

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position { pub x: i32, pub y: i32 }

#[derive(Debug, Clone, PartialEq)]
pub enum TileKind { Ground, Obstacle, Water }

// MapObject ссылается на типы из других модулей — объявить после них
pub enum MapObject {
    ResourcePile(ResourceBag),
    Town(TownId),
    NeutralArmy(Army),
}

pub struct Tile {
    pub kind: TileKind,
    pub object: Option<MapObject>,
}

pub struct AdventureMap {
    pub width: u32,
    pub height: u32,
    tiles: Vec<Tile>,   // плоский массив, индекс = y * width + x
}
```

### Методы `AdventureMap`

| Метод | Сигнатура | Описание |
|-------|-----------|----------|
| `new` | `fn new(width: u32, height: u32) -> Self` | Создаёт карту, заполненную `Ground`-тайлами без объектов |
| `get` | `fn get(&self, pos: Position) -> Option<&Tile>` | Возвращает тайл по позиции, `None` если за пределами |
| `get_mut` | `fn get_mut(&mut self, pos: Position) -> Option<&mut Tile>` | Мутабельная версия |
| `is_passable` | `fn is_passable(&self, pos: Position) -> bool` | `true` только если тайл `Ground` и на нём нет `NeutralArmy` |
| `neighbors` | `fn neighbors(&self, pos: Position) -> Vec<Position>` | 4 соседние клетки (вверх, вниз, влево, вправо) в пределах карты |

---

## Шаг 2 — `core::resources`

### Типы данных

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceKind { Gold }   // расширим позже

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ResourceBag { pub gold: u32 }

#[derive(Debug, Clone, PartialEq)]
pub struct InsufficientFunds;
```

### Методы `ResourceBag`

| Метод | Сигнатура | Описание |
|-------|-----------|----------|
| `add` | `fn add(&mut self, other: &ResourceBag)` | Прибавляет ресурсы in-place |
| `subtract` | `fn subtract(&mut self, other: &ResourceBag) -> Result<(), InsufficientFunds>` | Вычитает ресурсы; ошибка если не хватает |
| `can_afford` | `fn can_afford(&self, cost: &ResourceBag) -> bool` | Проверяет достаточность без изменения |

---

## Шаг 3 — `core::hero`

### Типы данных

```rust
pub struct UnitType {
    pub name: String,
    pub damage_per_unit: u32,
    pub hp: u32,
    pub cost: ResourceBag,   // стоимость одного существа
}

pub struct UnitStack {
    pub unit_type: UnitType,
    pub count: u32,
    pub hp_remaining: u32,   // здоровье последнего существа в стеке
}

pub struct Army(pub Vec<UnitStack>);  // максимум 7 слотов (как в оригинале)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HeroId(pub u32);

pub struct Hero {
    pub id: HeroId,
    pub name: String,
    pub position: Position,
    pub army: Army,
    pub movement_points: u32,
    pub movement_points_max: u32,
}
```

### Методы `UnitStack`

| Метод | Сигнатура | Описание |
|-------|-----------|----------|
| `is_alive` | `fn is_alive(&self) -> bool` | `count > 0` |

### Методы `Army`

| Метод | Сигнатура | Описание |
|-------|-----------|----------|
| `add_stack` | `fn add_stack(&mut self, stack: UnitStack) -> Result<(), ArmyFull>` | Добавляет стек; ошибка если уже 7 слотов |
| `is_empty` | `fn is_empty(&self) -> bool` | Нет живых стеков |

### Методы `Hero`

| Метод | Сигнатура | Описание |
|-------|-----------|----------|
| `can_move_to` | `fn can_move_to(&self, pos: Position, map: &AdventureMap) -> bool` | Позиция смежна с текущей И клетка проходима |
| `spend_movement` | `fn spend_movement(&mut self, cost: u32) -> Result<(), NoMovementPoints>` | Списывает очки движения; ошибка если недостаточно |
| `restore_movement` | `fn restore_movement(&mut self)` | Устанавливает `movement_points = movement_points_max` |

### Вспомогательные ошибки

```rust
pub struct ArmyFull;
pub struct NoMovementPoints;
```

---

## Шаг 4 — `core::player`

### Типы данных

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TownId(pub u32);

pub struct Town {
    pub id: TownId,
    pub position: Position,
    pub garrison: Army,
    pub available_recruits: Vec<(UnitType, u32)>,  // (тип существа, доступное количество)
    pub income: ResourceBag,                         // доход в начале каждого дня
}

pub struct Player {
    pub id: PlayerId,
    pub resources: ResourceBag,
    pub hero_ids: Vec<HeroId>,
}
```

---

## Шаг 5 — `core::state`

### Тип данных

```rust
pub struct GameState {
    pub map: AdventureMap,
    pub players: Vec<Player>,
    pub heroes: Vec<Hero>,
    pub towns: Vec<Town>,
    pub current_day: u32,
    pub active_player_id: PlayerId,
}
```

### Вспомогательные методы (приватные)

| Метод | Описание |
|-------|----------|
| `get_hero(&self, id: HeroId) -> Option<&Hero>` | Поиск героя по ID |
| `get_hero_mut(&mut self, id: HeroId) -> Option<&mut Hero>` | Мутабельный поиск героя |
| `get_player_mut(&mut self, id: PlayerId) -> Option<&mut Player>` | Мутабельный поиск игрока |
| `get_town_mut(&mut self, id: TownId) -> Option<&mut Town>` | Мутабельный поиск города |
| `get_active_player_mut(&mut self) -> &mut Player` | Активный игрок (всегда существует) |

---

## Шаг 6 — `core::commands`

### Команды

```rust
pub enum GameCommand {
    MoveHero { hero_id: HeroId, target: Position },
    CollectResource { hero_id: HeroId },
    RecruitUnits { town_id: TownId, unit_type_idx: usize, count: u32 },
    EndTurn,
}
```

### События

```rust
pub enum GameEvent {
    HeroMoved { hero_id: HeroId, from: Position, to: Position },
    ResourceCollected { hero_id: HeroId, amount: ResourceBag },
    BattleStarted { attacker: HeroId, defender_pos: Position },
    TurnEnded { day: u32 },
    DayIncome { player_id: PlayerId, amount: ResourceBag },
}
```

### Ошибки

```rust
pub enum CommandError {
    HeroNotFound,
    NotYourHero,           // герой принадлежит другому игроку
    NotAdjacent,           // целевая клетка не смежна с текущей позицией
    TileNotPassable,       // клетка непроходима
    NoMovementPoints,      // у героя нет очков движения
    NothingToCollect,      // на клетке нет ресурса
    TownNotFound,
    NotInTown,             // герой не стоит на клетке города
    NotEnoughRecruitsAvailable,
    InsufficientFunds,
    ArmyFull,
}
```

### Логика `GameState::apply`

```rust
pub fn apply(&mut self, command: GameCommand) -> Result<Vec<GameEvent>, CommandError>
```

#### `MoveHero { hero_id, target }`

1. Найти героя; если не найден → `HeroNotFound`
2. Проверить что герой принадлежит активному игроку → `NotYourHero`
3. Проверить что `target` смежна с `hero.position` → `NotAdjacent`
4. Проверить `map.is_passable(target)` → `TileNotPassable`
5. Проверить `hero.movement_points > 0` → `NoMovementPoints`
6. Списать 1 очко движения, обновить `hero.position = target`
7. Логировать: `[ADVENTURE] Hero "X" moved from (a,b) to (c,d). MP: N -> M`
8. Вернуть `HeroMoved`; если на клетке `NeutralArmy` — дополнительно `BattleStarted`

#### `CollectResource { hero_id }`

1. Найти героя → `HeroNotFound`
2. Получить тайл под героем; если нет `ResourcePile` → `NothingToCollect`
3. Забрать ресурс с карты (заменить `object` на `None`)
4. Добавить ресурс активному игроку
5. Логировать: `[ADVENTURE] Hero "X" collected N gold at (a,b). Total gold: M`
6. Вернуть `ResourceCollected`

#### `RecruitUnits { town_id, unit_type_idx, count }`

1. Найти город → `TownNotFound`
2. Найти героя активного игрока на позиции города → `NotInTown`
3. Проверить `unit_type_idx` в пределах списка; проверить доступное количество → `NotEnoughRecruitsAvailable`
4. Вычислить стоимость `cost = unit.cost * count`; проверить `can_afford` → `InsufficientFunds`
5. Списать золото, уменьшить `available_recruits[idx].1 -= count`
6. Создать `UnitStack` и добавить в армию героя → `ArmyFull`
7. Логировать: `[TOWN] Hero "X" recruited N UnitType for M gold. Army: [...]`

#### `EndTurn`

1. Восстановить `movement_points` всем героям активного игрока
2. Для каждого города: добавить `town.income` активному игроку, вернуть `DayIncome`
3. Увеличить `current_day += 1`
4. Логировать: `[TURN] Day N begins. Hero MP restored. Town income: +M gold`
5. Вернуть `[TurnEnded { day }]` + события `DayIncome`

> **Примечание:** пока один игрок, логику смены активного игрока добавим в Этапе 7.

---

## Шаг 7 — Unit-тесты

Все тесты в `src/core/commands.rs` в блоке `#[cfg(test)]`. Для каждого теста создаётся минимальный `GameState` через вспомогательную функцию `make_test_state()`.

| # | Название | Сценарий | Ожидаемый результат |
|---|----------|----------|---------------------|
| 1 | `hero_moves_to_free_tile` | Герой движется на смежный `Ground`-тайл | `Ok([HeroMoved])`, позиция обновлена, MP уменьшен на 1 |
| 2 | `hero_cannot_move_to_obstacle` | Герой движется на `Obstacle` | `Err(TileNotPassable)` |
| 3 | `hero_cannot_move_without_mp` | `movement_points = 0`, попытка движения | `Err(NoMovementPoints)` |
| 4 | `hero_collects_resource` | На клетке героя `ResourcePile(100 gold)` | `Ok([ResourceCollected])`, золото зачислено, объект удалён |
| 5 | `hero_cannot_collect_from_empty_tile` | На клетке нет объекта | `Err(NothingToCollect)` |
| 6 | `recruit_units_success` | Достаточно золота, есть существа в городе | `Ok(...)`, золото списано, стек добавлен в армию |
| 7 | `recruit_units_insufficient_funds` | Нехватка золота | `Err(InsufficientFunds)` |
| 8 | `end_turn_restores_movement` | `EndTurn` вызван | `movement_points == movement_points_max` у героя |
| 9 | `end_turn_grants_income` | Город с `income = 250 gold` | После `EndTurn` у игрока `+250 gold` |

---

## Интеграция в `main.rs`

```rust
mod core;   // добавить в начало main.rs
```

Модуль `core` не затрагивает Bevy-логику и не влияет на визуальную часть приложения.

---

## Порядок реализации

1. `resources.rs` (нет зависимостей внутри core)
2. `hero.rs` (зависит от `resources`)
3. `player.rs` (зависит от `hero`, `resources`)
4. `map.rs` (зависит от `resources`, `hero`, `player`)
5. `state.rs` (зависит от всех выше)
6. `commands.rs` (зависит от `state`)
7. `mod.rs` (реэкспорты)
8. Тесты в `commands.rs`
