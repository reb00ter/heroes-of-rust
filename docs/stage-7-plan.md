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

### 7.1 Формат файла (`assets/maps/default.ron`)

- [ ] Создать директорию `assets/maps/`.
- [ ] Записать карту этапа 6 (20×15) в RON-файл, точно воспроизводя `build_initial_game_state` после завершения этапа 6:

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
                (name: "Крестьянин", damage: 1, hp: 5,  cost: 25, count: 10, daily_growth: 5),
                (name: "Мечник",     damage: 3, hp: 10, cost: 75, count: 5,  daily_growth: 2),
            ],
        ),
    ],

    neutral_armies: [
        (pos: (7,5),   units: [(name: "Гоблин", damage: 2, hp: 5,  cost: 0, count: 5, daily_growth: 0)]),
        (pos: (13,6),  units: [(name: "Орк",    damage: 4, hp: 10, cost: 0, count: 4, daily_growth: 0)]),
        (pos: (17,10), units: [(name: "Тролль", damage: 8, hp: 25, cost: 0, count: 2, daily_growth: 0)]),
    ],

    hero: (
        pos:             (1,1),
        name:            "Aldric",
        movement_points: 10,
        army:            [(name: "Крестьянин", damage: 1, hp: 5, cost: 25, count: 5, daily_growth: 0)],
        starting_gold:   500,
    ),
)
```

**Ключевые решения формата:**
- Позиции — кортежи `(i32, i32)`, не именованные структуры.
- `UnitEntry` — единый тип для существ везде: в `recruits`, `neutral_armies.units` и `hero.army`. Поля `cost` и `daily_growth` для нейтралов = 0.
- Тайлы не перечисляются поштучно — только исключения (`obstacles`, `water`). Все остальные — `Ground`.

---

### 7.2 Типы данных (`src/data/mod.rs`)

- [ ] Объявить `MapDefinition` и все вспомогательные типы с `#[derive(serde::Deserialize)]`:

```rust
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
    pub recruits:     Vec<UnitEntry>,
}

/// Единый тип записи существа — используется в city.recruits, neutral_armies и hero.army.
#[derive(serde::Deserialize, Clone)]
pub struct UnitEntry {
    pub name:         String,
    pub damage:       u32,
    pub hp:           u32,
    pub cost:         u32,   // gold; 0 для нейтралов и стартовой армии
    pub count:        u32,
    pub daily_growth: u32,   // 0 для нейтралов и армии героя
}

#[derive(serde::Deserialize)]
pub struct NeutralArmyDef {
    pub pos:   (i32, i32),
    pub units: Vec<UnitEntry>,
}

#[derive(serde::Deserialize)]
pub struct HeroDef {
    pub pos:             (i32, i32),
    pub name:            String,
    pub movement_points: u32,
    pub army:            Vec<UnitEntry>,
    pub starting_gold:   u32,
}
```

---

### 7.3 Функция загрузки (`src/data/mod.rs`)

- [ ] Реализовать `pub fn load_map(path: &str) -> GameState`:
  - Читает файл через `std::fs::read_to_string(path)`.
  - Парсит через `ron::from_str::<MapDefinition>(&content)`.
  - При ошибке — `panic!` с понятным сообщением (файл обязан быть в репозитории).
  - Делегирует построение `GameState` в `fn map_def_to_game_state(def: MapDefinition) -> GameState`.

- [ ] Реализовать `fn map_def_to_game_state(def: MapDefinition) -> GameState` — содержит весь текущий код `build_initial_game_state`, но читает данные из `def`, а не из констант:
  - Создать `AdventureMap::new(def.width, def.height)`.
  - Расставить `Obstacle`/`Water` тайлы из `def.obstacles` / `def.water`.
  - Разместить `ResourcePile`, `Town`, `NeutralArmy` объекты.
  - Создать `Town` с `available_recruits` и `daily_growth` из `def.towns`.
  - Создать `Hero` с армией из `def.hero.army`.
  - Создать `Player` с `starting_gold`.
  - Логировать инициализацию в том же формате что сейчас.

---

### 7.4 Упрощение `adventure/render.rs`

- [ ] Заменить тело `build_initial_game_state()` на однострочник:
  ```rust
  pub fn build_initial_game_state() -> GameState {
      crate::data::load_map("assets/maps/default.ron")
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

- [ ] `load_default_map_parses` — файл читается, `width == 20`, `height == 15`.
- [ ] `default_map_has_correct_neutrals` — ровно 3 нейтральных отряда; позиции `(7,5)`, `(13,6)`, `(17,10)`; состав соответствует файлу.
- [ ] `default_map_has_town` — 1 город, позиция `(4,2)`, `daily_income == 250`, 2 типа рекрутов.
- [ ] `default_map_hero_start` — герой в позиции `(1,1)`, `movement_points == 10`, армия содержит 5 Крестьян, золото 500.
- [ ] `unknown_field_is_error` — RON с неизвестным полем верхнего уровня возвращает ошибку парсинга (проверяет что формат строгий).

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
└── maps/
    └── default.ron          — карта 20×15 (этап 6)

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
2. `src/data/mod.rs` — типы + `load_map` + `map_def_to_game_state` + тесты
3. `assets/maps/default.ron` — записать карту 20×15 (сверить с реальным `build_initial_game_state` после этапа 6)
4. `src/adventure/render.rs` — упростить `build_initial_game_state`; рефакторить `startup_setup` → `respawn_map_objects`
5. `src/adventure/mod.rs` + `input.rs` — убрать константы, перейти на динамические размеры
6. `src/main.rs` — подключить `mod data`
7. `cargo test` — все тесты зелёные
8. `cargo clippy -- -D warnings` + `cargo fmt --check`
9. `docs/map-generation-prompt.md`
10. Коммит

---

## Критерий готовности

1. `cargo run` — игра запускается, карта 20×15 идентична состоянию после этапа 6.
2. `cargo test` — все тесты проходят, включая 5 новых в `src/data/`.
3. Удаление `assets/maps/default.ron` → `cargo run` падает с понятным сообщением об ошибке.
4. `cargo clippy -- -D warnings` и `cargo fmt --check` — чисто.
5. Файл `docs/map-generation-prompt.md` существует и содержит полную схему с примером.
