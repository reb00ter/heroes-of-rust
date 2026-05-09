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
| Тесты `neutral_armies_placed`, `neutral_army_blocks_movement` | `adventure/render.rs` | ✅ остаются, будут вызывать тот же `build_initial_game_state` |

### Итог: вся игровая логика готова; нужны формат файла, загрузчик и промпт.

---

## Что добавляется в этом этапе

### Новые файлы

```
src/data/mod.rs               — MapDefinition, вспомогательные Def-типы, fn load_map()
assets/maps/default.ron       — текущая карта 16×12, точная копия build_initial_game_state
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
- [ ] Записать текущую карту 16×12 в RON-файл, точно воспроизводя `build_initial_game_state`:

```ron
(
    width:  16,
    height: 12,

    obstacles: [
        (3,1),(3,2),(3,3),(3,4),
        (7,3),(7,4),(7,5),(7,6),
        (10,2),(10,3),(11,2),
        (13,7),(13,8),(14,7),(14,8),
        (5,8),(5,9),(6,9),
    ],
    water: [(0,10),(0,11),(1,11),(2,11),(15,0),(15,1)],

    resource_piles: [
        (pos: (5,2),  gold: 100),
        (pos: (2,6),  gold: 150),
        (pos: (8,1),  gold: 200),
        (pos: (11,6), gold: 250),
        (pos: (4,10), gold: 100),
    ],

    towns: [
        (
            id:           0,
            pos:          (12,5),
            daily_income: 250,
            recruits: [
                (name: "Крестьянин", damage: 1, hp: 5,  cost: 25, count: 10, daily_growth: 5),
                (name: "Мечник",     damage: 3, hp: 10, cost: 75, count: 5,  daily_growth: 2),
            ],
        ),
    ],

    neutral_armies: [
        (pos: (5,3),  units: [(name: "Гоблин", damage: 2, hp: 5,  cost: 0, count: 5,  daily_growth: 0)]),
        (pos: (10,7), units: [(name: "Орк",    damage: 4, hp: 10, cost: 0, count: 3, daily_growth: 0)]),
    ],

    hero: (
        pos:             (1,1),
        name:            "Aldric",
        movement_points: 10,
        army:            [],
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
- [ ] Удалить все импорты, которые больше не нужны в `render.rs` после переноса логики.
- [ ] Убедиться что старые тесты `neutral_armies_placed` и `neutral_army_blocks_movement` проходят без изменений — они по-прежнему вызывают `build_initial_game_state()`.

---

### 7.5 Тесты (`src/data/mod.rs`)

- [ ] `load_default_map_parses` — файл читается, `width == 16`, `height == 12`.
- [ ] `default_map_has_correct_neutrals` — ровно 2 нейтральных отряда; позиции `(5,3)` и `(10,7)`; состав соответствует файлу.
- [ ] `default_map_has_town` — 1 город, позиция `(12,5)`, `daily_income == 250`, 2 типа рекрутов.
- [ ] `default_map_hero_start` — герой в позиции `(1,1)`, `movement_points == 10`, армия пустая, золото 500.
- [ ] `unknown_field_is_error` — RON с неизвестным полем верхнего уровня возвращает ошибку парсинга (проверяет что формат строгий).

> Все тесты используют `std::fs` напрямую — не нужен Bevy рантайм, запускаются через `cargo test`.

---

### 7.6 Промпт для генерации карт (`docs/map-generation-prompt.md`)

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
    └── default.ron          — карта 16×12

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

1. `Cargo.toml` — зависимости
2. `src/data/mod.rs` — типы + `load_map` + `map_def_to_game_state` (переносим код)
3. `assets/maps/default.ron` — записываем текущую карту
4. `src/adventure/render.rs` — упрощаем `build_initial_game_state`
5. `src/main.rs` — подключаем `mod data`
6. `cargo test` — все тесты зелёные
7. `cargo clippy -- -D warnings` + `cargo fmt --check`
8. `docs/map-generation-prompt.md`
9. Коммит

---

## Критерий готовности

1. `cargo run` — игра запускается, карта идентична текущей.
2. `cargo test` — все тесты проходят, включая 5 новых в `src/data/`.
3. Удаление `assets/maps/default.ron` → `cargo run` падает с понятным сообщением об ошибке.
4. `cargo clippy -- -D warnings` и `cargo fmt --check` — чисто.
5. Файл `docs/map-generation-prompt.md` существует и содержит полную схему с примером.
