# Stage 7 Plan — Динамическая загрузка карт

**Цель:** вынести карту из кода в RON-файл; уметь загружать любую карту без перекомпиляции; подготовить промпт для генерации карт через AI.

**Статус:** ⬜ Не начато

---

## Что уже есть в проекте

| Элемент | Файл | Состояние |
|---------|------|-----------|
| `build_initial_game_state()` — карта захардкожена | `adventure/render.rs:32` | ✅ нужно вынести в файл |
| `GameState`, `Town`, `Hero`, `Army`, `UnitType` | `core/` | ✅ без изменений |
| `bevy` уже тянет `serde` транзитивно | `Cargo.toml` | ✅ нужно добавить `derive`-фичу и `ron` |
| Тесты из этапа 6 (`three_neutral_armies_on_map`, `hero_has_starter_army` и др.) | `adventure/render.rs` | ✅ остаются, будут вызывать тот же `build_initial_game_state` |

### Итог: вся игровая логика готова; нужны формат файла, загрузчик и промпт.

---

## Что добавляется в этом этапе

### Новые файлы

```
src/data/mod.rs               — MapDefinition, вспомогательные Def-типы, fn load_map()
assets/maps/default.ron       — карта этапа 6 (20×15), точная копия build_initial_game_state
docs/map-generation-prompt.md — промпт для генерации карт через AI
```

### Изменяемые файлы

```
Cargo.toml              — добавить serde (derive) и ron
src/main.rs             — добавить mod data
src/adventure/render.rs — build_initial_game_state() становится однострочником
ROADMAP.md              — добавить Этап 7 и отметить задачи
```

---

## Зависимости (`Cargo.toml`)

- [ ] Добавить:
  ```toml
  serde = { version = "1", features = ["derive"] }
  ron   = "0.8"
  ```

> `bevy` уже использует `serde` внутри, но без фичи `derive` — она нужна для наших `Deserialize`-макросов.

---

## Задачи

### 7.1 Справочник существ (`assets/data/units.ron`)

Характеристики существ не являются частью карты — они живут отдельно. Правило дневного роста в замке также принадлежит типу существа, а не конкретной карте.

- [ ] Создать директорию `assets/data/`.
- [ ] Создать `assets/data/units.ron` — справочник всех существ в игре:

```ron
[
    (id: "peasant",   name: "Крестьянин", damage: 1, hp: 5,  cost: 25, daily_growth: 5),
    (id: "swordsman", name: "Мечник",     damage: 3, hp: 10, cost: 75, daily_growth: 2),
    (id: "goblin",    name: "Гоблин",     damage: 2, hp: 5,  cost: 0,  daily_growth: 0),
    (id: "orc",       name: "Орк",        damage: 4, hp: 10, cost: 0,  daily_growth: 0),
    (id: "troll",     name: "Тролль",     damage: 8, hp: 25, cost: 0,  daily_growth: 0),
]
```

- `id` — ASCII-идентификатор, используется в файлах карт для связки; без кириллицы.
- `name` — отображаемое название для UI; кириллица, только здесь.
- `daily_growth: 0` для нейтральных существ — они не восполняются в замке.
- Новые типы существ добавляются только в этот файл, карты не трогаются.

### 7.2 Формат карты (`assets/maps/default.ron`)

Карта ссылается на существ **только по имени и количеству** — без характеристик.

- [ ] Создать директорию `assets/maps/`.
- [ ] Записать карту этапа 6 (20×15):

```ron
(
    width:  20,
    height: 15,

    // Только нестандартные тайлы; остальные — Ground по умолчанию
    obstacles: [
        (3,0),(4,0),(3,1),
        (9,3),(9,4),(9,5),
        (14,1),(14,2),(15,8),(15,9),
        (6,12),(7,12),(7,13),
        (11,11),(12,11),
    ],
    water: [
        (0,13),(0,14),(1,14),(2,14),
        (19,0),(19,1),(18,0),
        (18,14),(19,14),(19,13),
    ],

    resource_piles: [
        (pos: (2,5),  gold: 150),
        (pos: (7,1),  gold: 200),
        (pos: (10,9), gold: 250),
        (pos: (15,3), gold: 300),
        (pos: (18,12),gold: 200),
    ],

    towns: [
        (
            id:           0,
            pos:          (4,2),
            daily_income: 250,
            recruits: [
                (id: "peasant",   count: 10),
                (id: "swordsman", count: 5),
            ],
        ),
    ],

    neutral_armies: [
        (pos: (7,5),   units: [(id: "goblin", count: 5)]),
        (pos: (13,6),  units: [(id: "orc",    count: 4)]),
        (pos: (17,10), units: [(id: "troll",  count: 2)]),
    ],

    hero: (
        pos:             (1,1),
        name:            "Aldric",
        movement_points: 10,
        army:            [(id: "peasant", count: 5)],
        starting_gold:   500,
    ),
)
```

**Ключевые решения формата:**
- Карта ссылается на существ через ASCII `id` — без кириллицы в файлах карт.
- Отображаемое `name` хранится только в `units.ron` и оттуда попадает в `UnitType.name` для UI.
- Тайлы не перечисляются поштучно — только исключения (`obstacles`, `water`). Все остальные — `Ground`.

---

### 7.3 Типы данных (`src/data/mod.rs`)

- [ ] Объявить типы для **справочника существ**:

```rust
/// Полное описание типа существа — загружается из units.ron.
#[derive(serde::Deserialize, Clone)]
pub struct UnitTypeDef {
    pub id:           String,  // ASCII-ключ для связки с картами
    pub name:         String,  // отображаемое название для UI (кириллица)
    pub damage:       u32,
    pub hp:           u32,
    pub cost:         u32,
    pub daily_growth: u32,
}
```

- [ ] Объявить типы для **карты** (`MapDefinition` и вспомогательные):

```rust
/// Ссылка на существо в карте — только id и количество.
#[derive(serde::Deserialize, Clone)]
pub struct StackRef {
    pub id:    String,  // ASCII, совпадает с UnitTypeDef.id
    pub count: u32,
}

#[derive(serde::Deserialize)]
pub struct MapDefinition {
    pub width:          u32,
    pub height:         u32,
    pub obstacles:      Vec<(i32, i32)>,
    pub water:          Vec<(i32, i32)>,
    pub resource_piles: Vec<ResourcePileDef>,
    pub towns:          Vec<TownDef>,
    pub neutral_armies: Vec<NeutralArmyDef>,
    pub hero:           HeroDef,
}

#[derive(serde::Deserialize)]
pub struct ResourcePileDef {
    pub pos:  (i32, i32),
    pub gold: u32,
}

#[derive(serde::Deserialize)]
pub struct TownDef {
    pub id:           u32,
    pub pos:          (i32, i32),
    pub daily_income: u32,
    pub recruits:     Vec<StackRef>,
}

#[derive(serde::Deserialize)]
pub struct NeutralArmyDef {
    pub pos:   (i32, i32),
    pub units: Vec<StackRef>,
}

#[derive(serde::Deserialize)]
pub struct HeroDef {
    pub pos:             (i32, i32),
    pub name:            String,
    pub movement_points: u32,
    pub army:            Vec<StackRef>,
    pub starting_gold:   u32,
}
```

---

### 7.4 Функция загрузки (`src/data/mod.rs`)

- [ ] Реализовать `pub fn load_units(path: &str) -> Vec<UnitTypeDef>`:
  - Читает `assets/data/units.ron`, парсит `Vec<UnitTypeDef>`.
  - При ошибке — `panic!` с понятным сообщением.

- [ ] Реализовать вспомогательную функцию `fn resolve(id: &str, units: &[UnitTypeDef]) -> UnitType`:
  - Ищет `UnitTypeDef` по `id`, конвертирует в `core::hero::UnitType` (кириллическое `name` идёт в `UnitType.name` для UI).
  - `panic!` если `id` не найден — ошибка данных, не рантайм.

- [ ] Реализовать `pub fn load_map(map_path: &str, units_path: &str) -> GameState`:
  - Загружает справочник: `load_units(units_path)`.
  - Загружает карту: `ron::from_str::<MapDefinition>(...)`.
  - Делегирует построение: `map_def_to_game_state(def, &units)`.

- [ ] Реализовать `fn map_def_to_game_state(def: MapDefinition, units: &[UnitTypeDef]) -> GameState`:
  - Создать `AdventureMap::new(def.width, def.height)`.
  - Расставить `Obstacle`/`Water` тайлы.
  - Разместить `ResourcePile`, `Town`, `NeutralArmy` объекты.
  - `Town`: `available_recruits` — резолвить каждый `StackRef` через `resolve()`, `daily_growth` брать из `UnitTypeDef.daily_growth`.
  - `NeutralArmy`: стеки из `StackRef` через `resolve()`.
  - `Hero`: армия из `StackRef` через `resolve()`.
  - `Player`: `starting_gold` из `def.hero.starting_gold`.
  - Логировать инициализацию.

---

### 7.4 Упрощение `adventure/render.rs`

- [ ] Заменить тело `build_initial_game_state()` на однострочник:
  ```rust
  pub fn build_initial_game_state() -> GameState {
      crate::data::load_map("assets/maps/default.ron", "assets/data/units.ron")
  }
  ```
- [ ] Рефакторить `startup_setup`: вместо дублирования логики спавна вызвать уже существующую `respawn_map_objects` (создана в этапе 6 для «Играть снова»). Спавн камеры и UI остаётся в `startup_setup`, спавн тайлов/объектов/героя — через `respawn_map_objects`.
- [ ] Удалить все импорты, которые больше не нужны в `render.rs` после переноса логики.
- [ ] Убедиться что тесты из этапа 6 (`three_neutral_armies_on_map`, `hero_has_starter_army` и др.) проходят без изменений — они вызывают `build_initial_game_state()`.

### 7.5 Динамические размеры карты (`adventure/mod.rs`)

Константы `MAP_WIDTH = 20` и `MAP_HEIGHT = 15` сейчас захардкожены, но после перехода на файлы карта может быть любого размера. Нужно убрать жёсткую привязку.

- [ ] Удалить (или сделать приватными/deprecated) `pub const MAP_WIDTH` и `pub const MAP_HEIGHT`.
- [ ] `startup_setup` и `respawn_map_objects` читают размеры из `game_state.0.map.width` / `game_state.0.map.height`.
- [ ] `grid_to_world` и `world_to_grid` принимают `map_w: u32, map_h: u32` как параметры вместо обращения к константам.
- [ ] Bounds-проверка мыши в `adventure/input.rs` читает размеры из `GameStateResource`, а не из констант.

> **Примечание:** это не требует поддержки resize во время игры — размеры читаются один раз при инициализации/рестарте. Но теперь `default.ron` с `width: 30, height: 20` заработает без изменений в коде.

---

### 7.6 Тесты (`src/data/mod.rs`)

- [ ] `load_units_parses` — `units.ron` читается, содержит 5 записей; запись с `id == "peasant"` имеет `name == "Крестьянин"`, `damage == 1`, `daily_growth == 5`.
- [ ] `unknown_unit_panics` — `resolve("dragon", &units)` паникует с понятным сообщением.
- [ ] `load_default_map_parses` — карта читается, `width == 20`, `height == 15`.
- [ ] `default_map_has_correct_neutrals` — 3 нейтрала; позиции `(7,5)`, `(13,6)`, `(17,10)`; характеристики взяты из справочника.
- [ ] `default_map_has_town` — 1 город, позиция `(4,2)`, `daily_income == 250`, `daily_growth` Крестьянина == 5 (из справочника).
- [ ] `default_map_hero_start` — герой в `(1,1)`, `movement_points == 10`, армия: 5 Крестьян, золото 500.
- [ ] `unknown_field_in_map_is_error` — лишнее поле в `map.ron` возвращает ошибку парсинга.

> Все тесты используют `std::fs` напрямую — не нужен Bevy рантайм, запускаются через `cargo test`.

---

### 7.7 Промпт для генерации карт (`docs/map-generation-prompt.md`)

- [ ] Создать `docs/map-generation-prompt.md` со следующим содержимым:
  - **Полная схема** — каждое поле с типом, описанием и допустимыми значениями.
  - **Ограничения** — список валидационных правил (позиции в пределах `width`×`height`, объекты не на `Obstacle`/`Water`, нейтралы не на тайле города и наоборот, не более 7 стеков в армии и т.д.).
  - **Рекомендации по балансу** — диапазоны силы армий для лёгких / средних / сложных отрядов.
  - **Полный пример** — `default.ron` целиком как референс.
  - **Инструкция** — «Верни только валидный RON-файл без пояснений и markdown-оберток».

---

## Структура файлов после этапа

```
assets/
├── data/
│   └── units.ron            — справочник существ (характеристики, daily_growth)
└── maps/
    └── default.ron          — карта 20×15 (только позиции и имена существ)

src/
├── main.rs                  — +mod data
├── data/
│   └── mod.rs               — MapDefinition, UnitEntry, TownDef, ..., load_map(), тесты
└── adventure/
    └── render.rs            — build_initial_game_state() → однострочник

docs/
└── map-generation-prompt.md — AI-промпт
```

---

## Порядок реализации

1. `Cargo.toml` — зависимости (`serde`, `ron`)
2. `src/data/mod.rs` — `UnitTypeDef`, `StackRef`, `MapDefinition` и прочие типы
3. `assets/data/units.ron` — справочник существ
4. `src/data/mod.rs` — `load_units`, `resolve`, `load_map`, `map_def_to_game_state` + тесты
5. `assets/maps/default.ron` — карта 20×15 (сверить с реальным `build_initial_game_state`)
6. `src/adventure/render.rs` — упростить `build_initial_game_state`; `startup_setup` → `respawn_map_objects`
7. `src/adventure/mod.rs` + `input.rs` — убрать константы, перейти на динамические размеры
8. `src/main.rs` — подключить `mod data`
9. `cargo test` — все тесты зелёные
10. `cargo clippy -- -D warnings` + `cargo fmt --check`
11. `docs/map-generation-prompt.md`
12. Коммит

---

## Критерий готовности

1. `cargo run` — игра запускается, карта 20×15 идентична состоянию после этапа 6.
2. `cargo test` — все тесты проходят, включая 5 новых в `src/data/`.
3. Удаление `assets/maps/default.ron` → `cargo run` падает с понятным сообщением об ошибке.
4. `cargo clippy -- -D warnings` и `cargo fmt --check` — чисто.
5. Файл `docs/map-generation-prompt.md` существует и содержит полную схему с примером.
