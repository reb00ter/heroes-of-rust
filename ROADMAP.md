# ROADMAP — Heroes of Rust

Детальный план разработки по этапам. Каждый этап разбит на конкретные задачи.
Статусы: ✅ готово | 🔄 в работе | ⬜ не начато

> **Живой документ.** Задачи начиная с Этапа 2 носят ориентировочный характер и будут уточняться по мере прохождения предыдущих этапов. Не нужно планировать далеко вперёд — детализировать следующий этап стоит только после завершения текущего.

---

## Этап 0 — Подготовка репозитория ✅

**Цель:** создать основу проекта, убедиться что `cargo run` открывает окно.

### 0.1 Инициализация проекта

- [x] Выполнить `git init`
- [x] Выполнить `cargo new heroes_of_rust --bin`
- [x] Проверить актуальную версию Rust (`rustup show`) и зафиксировать в `rust-toolchain.toml`
- [x] Добавить `.gitignore` (исключить `target/`, `.env`)
- [x] Добавить `README.md` с кратким описанием проекта и командами разработки
- [x] Сделать первый коммит: `git commit -m "chore: init project"`

### 0.2 Зависимости

- [x] Добавить `bevy` в `Cargo.toml` (последняя стабильная версия)
- [x] Добавить фичу `bevy/dynamic_linking` для ускорения сборки в dev-режиме
- [x] Добавить `[profile.dev]` оптимизации в `Cargo.toml`:
  - `opt-level = 1` для самого крейта
  - `opt-level = 3` для зависимостей (`[profile.dev.package."*"]`)

### 0.3 Инструменты качества кода

- [x] Добавить `rustfmt.toml` с базовыми настройками форматирования
- [x] Добавить `.clippy.toml` или `#![deny(clippy::...)]` в `main.rs`
- [x] Убедиться что `cargo fmt` и `cargo clippy` работают без ошибок

### 0.4 Минимальное окно Bevy

- [x] Написать `main.rs` с `App::new()` и `DefaultPlugins`
- [x] Задать заголовок окна: `"Heroes of Rust"`
- [x] Задать начальный размер окна (например, 1280×720)
- [x] Добавить однотонный фоновый цвет (`ClearColor`)
- [x] Убедиться что `cargo run` открывает окно без паники и ошибок

**Критерий готовности:** `cargo run` стабильно открывает пустое окно игры.

---

## Этап 1 — Чистая игровая модель ✅

**Цель:** реализовать ядро игровой логики без какой-либо графики. Всё покрыто тестами.

### 1.1 Структура модулей

- [x] Создать `src/core/mod.rs`
- [x] Создать подмодули: `map`, `hero`, `player`, `resources`, `town`, `commands`
- [x] Подключить `mod core;` в `main.rs`

### 1.2 Карта и позиции (`core::map`)

- [x] Объявить `Position { x: i32, y: i32 }` с `PartialEq`, `Eq`, `Hash`, `Clone`, `Copy`, `Debug`
- [x] Объявить `TileKind`: `Ground`, `Obstacle`, `Water`
- [x] Объявить `Tile { kind: TileKind, object: Option<MapObject> }`
- [x] Объявить `MapObject`: `ResourcePile(ResourceBag)`, `Town(TownId)`, `NeutralArmy(Army)`
- [x] Объявить `AdventureMap { width, height, tiles: Vec<Tile> }`
- [x] Реализовать `AdventureMap::new(width, height)` — заполнить `Ground`-тайлами
- [x] Реализовать `AdventureMap::get(pos)` / `get_mut(pos)` — возврат `Option<&Tile>`
- [x] Реализовать `AdventureMap::is_passable(pos)` — проверка проходимости
- [x] Реализовать `AdventureMap::neighbors(pos)` — 4 соседних клетки (в пределах карты)

### 1.3 Ресурсы (`core::resources`)

- [x] Объявить `ResourceKind`: `Gold` (только этот на старте)
- [x] Объявить `ResourceBag { gold: u32 }`
- [x] Реализовать `ResourceBag::add(other: &ResourceBag)`
- [x] Реализовать `ResourceBag::subtract(other: &ResourceBag) -> Result<(), InsufficientFunds>`
- [x] Реализовать `ResourceBag::can_afford(cost: &ResourceBag) -> bool`

### 1.4 Существа и армия (`core::hero`)

- [x] Объявить `UnitType { name, damage_per_unit: u32, hp: u32, cost: ResourceBag }`
- [x] Объявить `UnitStack { unit_type: UnitType, count: u32, hp_remaining: u32 }`
- [x] Реализовать `UnitStack::is_alive() -> bool`
- [x] Объявить `Army(Vec<UnitStack>)` (до 7 слотов — как в оригинале)
- [x] Реализовать `Army::add_stack(stack: UnitStack)`
- [x] Реализовать `Army::is_empty() -> bool`

### 1.5 Герой (`core::hero`)

- [x] Объявить `HeroId(u32)`
- [x] Объявить `Hero { id: HeroId, name: String, position: Position, army: Army, movement_points: u32, movement_points_max: u32 }`
- [x] Реализовать `Hero::can_move_to(pos, map) -> bool`
- [x] Реализовать `Hero::spend_movement(cost: u32) -> Result<(), NoMovement>`
- [x] Реализовать `Hero::restore_movement()`

### 1.6 Игрок и состояние игры (`core::player`, `core::state`)

- [x] Объявить `PlayerId(u32)`
- [x] Объявить `Player { id: PlayerId, resources: ResourceBag, hero_ids: Vec<HeroId> }`
- [x] Объявить `TownId(u32)`
- [x] Объявить `Town { id: TownId, position: Position, garrison: Army, available_recruits: Vec<(UnitType, u32)> }`
- [x] Объявить `GameState { map, players, heroes, towns, current_day, active_player_id }`

### 1.7 Команды игрока (`core::commands`)

- [x] Объявить `GameCommand` enum:
  - `MoveHero { hero_id, target: Position }`
  - `CollectResource { hero_id }`
  - `RecruitUnits { town_id, unit_type_idx: usize, count: u32 }`
  - `EndTurn`
- [x] Объявить `CommandError` enum с вариантами для всех ошибок
- [x] Реализовать `GameState::apply(command) -> Result<Vec<GameEvent>, CommandError>`
- [x] Реализовать логику `MoveHero`:
  - проверить что герой принадлежит активному игроку
  - проверить что клетка проходима и смежна с текущей позицией
  - проверить наличие очков движения
  - переместить героя, списать очки движения
  - если на клетке объект — вернуть событие взаимодействия
- [x] Реализовать логику `CollectResource`:
  - если на клетке героя есть `ResourcePile` — добавить ресурс игроку, убрать объект с карты
- [x] Реализовать логику `RecruitUnits`:
  - проверить что герой находится в городе
  - проверить что существ достаточно в городе
  - списать золото, добавить стек в армию героя
- [x] Реализовать логику `EndTurn`:
  - восстановить очки движения всех героев
  - начислить доход города
  - увеличить счётчик дня

### 1.8 События (`core::events`)

- [x] Объявить `GameEvent` enum:
  - `HeroMoved { hero_id, from: Position, to: Position }`
  - `ResourceCollected { hero_id, amount: ResourceBag }`
  - `BattleStarted { attacker: HeroId, defender_pos: Position }`
  - `TurnEnded { day: u32 }`
  - `DayIncome { player_id, amount: ResourceBag }`

### 1.9 Unit-тесты (`core`)

- [x] Тест: герой движется на свободную клетку — успех
- [x] Тест: герой движется на препятствие — ошибка
- [x] Тест: герой движется без очков движения — ошибка
- [x] Тест: герой собирает ресурс с клетки — золото зачислено, объект удалён
- [x] Тест: герой пытается собрать с пустой клетки — ошибка
- [x] Тест: найм существ за достаточное золото — успех
- [x] Тест: найм существ при нехватке золота — ошибка
- [x] Тест: завершение хода восстанавливает очки движения
- [x] Тест: завершение хода начисляет доход города

**Критерий готовности:** `cargo test` проходит все тесты без запуска окна.

---

## Этап 2 — Карта приключений на экране ✅

**Цель:** увидеть карту и управлять героем мышью или клавиатурой.

### 2.1 Модуль adventure

- [x] Создать `src/adventure/mod.rs`, `render.rs`, `input.rs`, `sync.rs`
- [x] Зарегистрировать `AdventurePlugin` в `main.rs`

### 2.2 Ресурс GameState в Bevy

- [x] Обернуть `GameState` в Bevy-ресурс (`Resource`) через newtype `GameStateResource`
- [x] Инициализировать тестовую карту 16×12 в `startup`-системе:
  - несколько клеток `Obstacle`
  - герой в стартовой позиции

### 2.3 Отрисовка карты

- [x] Выбрать размер тайла (48×48 пикселей)
- [x] В startup-системе заспавнить `Sprite` для каждого тайла
  - `Ground` — зелёный цвет
  - `Obstacle` — серый цвет
  - `Water` — синий цвет
- [x] Настроить камеру (`Camera2d`) по центру карты

### 2.4 Отрисовка героя

- [x] Заспавнить сущность героя с компонентом `HeroMarker(HeroId)` и спрайтом
- [x] Написать систему `sync_hero_transform` — синхронизировать `Transform` героя с `hero.position` из `GameState`

### 2.5 Отображение очков движения

- [x] Добавить текстовый UI-элемент `"MP: X / Y"` поверх сцены
- [x] Написать систему `update_movement_ui` — обновлять текст при изменении `Hero::movement_points`

### 2.6 Управление: клавиатура

- [x] Система `keyboard_input`:
  - `WASD` или стрелки → вычислить целевую позицию
  - Отправить `GameCommand::MoveHero` в `GameState::apply`
  - Обработать `CommandError` (залогировать или показать в UI)

### 2.7 Управление: мышь

- [x] Система `mouse_click_input`:
  - Перевести экранные координаты клика в `Position` на сетке
  - Отправить `GameCommand::MoveHero`
- [x] Визуально выделить клетку под курсором (hover highlight)

### 2.8 Отображение доступных ходов

- [x] Вычислить список проходимых соседних клеток от позиции героя
- [x] Визуально подсвечивать их (полупрозрачный спрайт поверх тайла)
- [x] Обновлять при каждом ходе

**Критерий готовности:** игрок двигает героя по карте мышью или клавиатурой, видит очки движения.

---

## Этап 3 — Ресурсы и экономика ✅

**Цель:** первая полезная интеракция — собрать золото.

### 3.1 Объекты на карте

- [x] В `AdventureMap` поддержать `MapObject::ResourcePile`
- [x] При инициализации карты разместить 3–5 кучек золота в случайных/фиксированных позициях

### 3.2 Отрисовка ресурсов

- [x] Заспавнить спрайт/маркер для каждой `ResourcePile` на карте
- [x] Привязать компонент `ResourcePileMarker(Position)` к сущности
- [x] Убирать спрайт при сборе ресурса (система `sync_resource_piles`)

### 3.3 Автосбор при входе на клетку

- [x] После успешного `MoveHero` проверять событие взаимодействия
- [x] Если герой вошёл на клетку с `ResourcePile` — автоматически применять `CollectResource`

### 3.4 UI ресурсов

- [x] Добавить панель ресурсов в верхней части экрана: `"Золото: X"`
- [x] Система `update_resource_ui` — обновляется при изменении `GameState`

### 3.5 Завершение хода и доход

- [x] Добавить кнопку «Завершить ход» (клавиша `Space`/`Enter`)
- [x] При нажатии применять `GameCommand::EndTurn`
- [x] Отображать текущий день: `"День X"`

**Критерий готовности:** игрок собирает золото, видит изменение баланса, завершает ход.

---

## Этап 4 — Город и найм существ ✅

**Цель:** тратить золото, увеличивать армию.

### 4.1 Город на карте

- [x] Добавить `MapObject::Town(TownId)` в тайл карты
- [x] Разместить один город на стартовой позиции
- [x] Отрисовать спрайт/маркер города (фиолетовый, Z=1)

### 4.2 Вход в город

- [x] При движении героя на клетку с городом — открывать экран города
- [x] Вводить состояние `GameScreen::Town` (Bevy States)

### 4.3 Экран города

- [x] Отрисовать простое окно поверх карты (Bevy UI, GlobalZIndex=100)
- [x] Показать список доступных для найма существ (название, стоимость, HP, урон, количество)
- [x] Кнопка `Нанять 1` для каждого типа существ
- [x] Кнопка `Покинуть город` / `Escape` — вернуться на карту

### 4.4 Отображение армии

- [x] На экране города показывать текущую армию героя
- [x] На панели карты — строка `"Армия: …"` (ArmyText, синхронизируется с GameState)

### 4.5 Прирост существ

- [x] При `EndTurn` пополнять количество доступных существ в городе (`daily_growth`)
- [x] Тест `end_turn_replenishes_recruits` — покрывает логику

**Критерий готовности:** игрок заходит в город, нанимает существ за золото, видит изменение армии.

---

## Этап 5 — Простой бой ✅

**Цель:** добавить конфликт. Герой атакует нейтральный отряд и побеждает.

### 5.1 Нейтральный отряд на карте

- [x] Добавить `MapObject::NeutralArmy(Army)` в карту
- [x] Разместить 1–2 нейтральных отряда на карте
- [x] Отрисовать маркер нейтрального отряда

### 5.2 Переход в режим боя

- [x] При движении героя на клетку с `NeutralArmy` — генерировать `GameEvent::BattleStarted`
- [x] Переключить состояние на `GameScreen::Battle`
- [x] Передать в `BattleState`:
  - атакующая армия (Hero.army)
  - защищающаяся армия (NeutralArmy)

### 5.3 Модуль боя (`src/battle/`)

- [x] Создать `src/battle/mod.rs`, `state.rs`, `render.rs`, `input.rs`
- [x] Объявить `BattleState`:
  - `attacker_stacks: Vec<BattleStack>`
  - `defender_stacks: Vec<BattleStack>`
  - `turn_order: VecDeque<StackId>` (пока просто чередовать)
  - `current_stack: StackId`
- [x] Объявить `BattleStack { id: StackId, unit_type, count, hp_remaining, side: Side }`
- [x] Объявить `Side`: `Attacker`, `Defender`

### 5.4 Механика боя

- [x] Функция `BattleState::current_stack()` — чей ход
- [x] Функция `BattleState::attack(target_id) -> BattleEvent`:
  - вычислить урон: `damage = count * unit_base_damage`
  - вычислить убитых: `killed = damage / target.unit_type.hp`
  - уменьшить `count` цели, обновить `hp_remaining`
  - если цель мертва — убрать из очереди
- [x] Функция `BattleState::next_turn()` — передать ход следующему стеку
- [x] Функция `BattleState::is_over() -> Option<Side>` — проверить победителя

### 5.5 Отрисовка боя

- [x] Отрисовать два ряда стеков (атакующие слева, защитники справа)
- [x] Показать для каждого стека: название, количество, HP
- [x] Подсветить текущий активный стек
- [x] Показать кнопку `Атаковать` (выбрать цель кликом)

### 5.6 Простой AI защитника

- [x] Когда ход переходит к нейтральному стеку — автоматически атаковать первый живой стек атакующего

### 5.7 Завершение боя

- [x] При победе атакующего:
  - убрать `NeutralArmy` с карты
  - обновить армию героя (учесть потери)
  - вернуться на карту (`GameScreen::Adventure`)
  - показать кратный лог или баннер `"Победа!"`
- [x] При поражении атакующего:
  - показать `"Поражение"`
  - (пока) перезапустить игру или заморозить состояние

**Критерий готовности:** игрок побеждает нейтральный отряд и продолжает игру на карте.

---

## Этап 6 — Вертикальный срез ✅

**Цель:** собрать играбельную мини-игру от старта до финала.

### 6.1 Дизайн стартовой карты

- [x] Карта 20×15 тайлов
- [x] Расстановка объектов:
  - стартовый город (позиция игрока)
  - 4–5 кучек золота
  - 3 нейтральных отряда разной силы (Гоблин×5, Орк×4, Тролль×2)
  - несколько препятствий и воды по углам

### 6.2 Стартовое состояние

- [x] Герой со стартовой армией (5 Крестьян)
- [x] Начальный запас золота (500)
- [x] Начальный доход города (250/день)

### 6.3 Условие победы

- [x] Метод `GameState::count_neutral_armies()` — считает оставшихся нейтралов
- [x] Проверка после каждого боя в `finish_battle`
- [x] Переход в `GameScreen::GameOver` при победе (нейтралов = 0) или поражении

### 6.4 Экран победы / поражения

- [x] Создать `GameScreen::GameOver` в enum
- [x] Ресурс `GameOverResult { is_victory: bool }`
- [x] `src/gameover.rs` — `GameOverPlugin` с полноэкранным UI
- [x] Текст `"ПОБЕДА!"` (зелёный) / `"ПОРАЖЕНИЕ"` (красный), 96px
- [x] Кнопка `"Играть снова"` — сбрасывает `GameState` и возвращает на карту

### 6.5 Полировка петли

- [x] Переходы Adventure → Town → Adventure → Battle → Adventure → GameOver → Adventure
- [x] `EndTurn` корректно обновляет MP, золото, день, рекрутов
- [x] Герой не может атаковать без армии (лог предупреждения)
- [x] Нельзя нанять больше существ, чем есть; нельзя нанять без золота
- [x] `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` — все зелёные

**Критерий готовности:** в игру можно сыграть от начала до конца за 5–10 минут.

---

## Этап 7 — Динамическая загрузка карт ✅

**Цель:** вынести карту из кода в RON-файл; уметь загружать любую карту без перекомпиляции; подготовить промпт для генерации карт через AI.

### 7.1 Зависимости

- [x] Добавить `serde = { version = "1", features = ["derive"] }` и `ron = "0.8"` в `Cargo.toml`

### 7.2 Справочник существ

- [x] Создать `assets/data/units.ron` — характеристики и дневной рост всех типов существ
- [x] `UnitTypeDef { id, name, damage, hp, cost, daily_growth }` в `src/data/mod.rs`

### 7.3 Формат и файл карты

- [x] Создать директорию `assets/maps/`
- [x] Создать `assets/maps/default.ron` — карта 20×15; ссылки на существ только по ASCII `id` и количеству (`StackRef { id, count }`)

### 7.4 Модуль данных (`src/data/mod.rs`)

- [x] Объявить `MapDefinition`, `StackRef`, `ResourcePileDef`, `TownDef`, `NeutralArmyDef`, `HeroDef` с `#[derive(serde::Deserialize)]`
- [x] Реализовать `pub fn load_units(path: &str) -> Vec<UnitTypeDef>`
- [x] Реализовать `fn resolve(id: &str, units: &[UnitTypeDef]) -> UnitType`
- [x] Реализовать `pub fn load_map(map_path: &str, units_path: &str) -> GameState`
- [x] Реализовать `fn map_def_to_game_state(def: &MapDefinition, units: &[UnitTypeDef]) -> GameState`
- [x] Подключить `mod data` в `main.rs`

### 7.5 Упрощение `adventure/render.rs`

- [x] Заменить тело `build_initial_game_state()` на однострочник `crate::data::load_map(...)`
- [x] Рефакторить `startup_setup` — извлечь `respawn_map_objects` для спавна тайлов и городов

### 7.6 Динамические размеры карты (`adventure/mod.rs`, `input.rs`, `sync.rs`)

- [x] Убрать `pub const MAP_WIDTH` / `MAP_HEIGHT`; читать размеры из `gs.map.width/height`
- [x] `grid_to_world(pos, map_w, map_h)` / `world_to_grid(world, map_w, map_h)` принимают размеры как параметры
- [x] `spawn_gold_pile`, `spawn_neutral_army`, `spawn_hero` принимают `map_w, map_h`
- [x] Bounds-проверка мыши читает размеры из `GameStateResource`

### 7.7 Тесты (`src/data/mod.rs`)

- [x] `load_units_parses` — справочник читается, 5 записей, характеристики верны
- [x] `unknown_unit_panics` — несуществующий `id` даёт panic с понятным сообщением
- [x] `load_default_map_parses` — карта читается, `width == 20`, `height == 15`
- [x] `default_map_has_correct_neutrals` — 3 нейтрала, позиции и характеристики (из справочника) верны
- [x] `default_map_has_town` — 1 город, `daily_growth` берётся из `units.ron`
- [x] `default_map_hero_start` — позиция (1,1), MP=10, армия 5 крестьян, золото 500
- [x] `unknown_field_in_map_is_error` — лишнее поле в RON даёт ошибку парсинга

### 7.8 Промпт для генерации карт

- [x] Создать `docs/map-generation-prompt.md` — полная схема формата, ограничения, рекомендации по балансу, пример `default.ron`, инструкция для AI

**Критерий готовности:** `cargo run` запускается с той же картой; удаление `default.ron` даёт понятную панику; все тесты зелёные; промпт готов.

---

## Этап 8 — Экран запуска ✅

**Цель:** главное меню перед игрой — ввести имя героя, выбрать карту из доступных; некорректная карта показывает ошибку, не ронит приложение.

**Зависит от:** Этап 7.

### 8.1 Валидация и сканирование карт (`src/data/mod.rs`)

- [x] `MapEntry`, `MapInfo`, `MapEntryResult` — типы результата сканирования
- [x] `discover_maps(dir) -> Vec<MapEntry>` — перебирает `*.ron` в папке
- [x] `validate_map(path, units) -> Result<MapInfo, String>` — парсинг + семантические проверки (id юнитов, позиции, тайлы)

### 8.2 Ресурс конфигурации старта (`main.rs`)

- [x] `GameStartConfig { map_path, hero_name }` — заполняется меню, читается в `OnEnter(Adventure)`
- [x] `GameScreen::MainMenu` — новый `#[default]` вместо `Adventure`

### 8.3 Главное меню (`src/menu/mod.rs`)

- [x] `setup_menu` / `despawn_menu` (OnEnter/OnExit MainMenu)
- [x] Поле ввода имени героя (ReceivedCharacter + Backspace, ≤20 символов)
- [x] Список карт: валидные — кликабельны с мета-инфо; невалидные — серые с ошибкой
- [x] Кнопка «Начать игру» — активна если имя ≠ "" и карта выбрана
- [x] «Играть снова» в `gameover.rs` → `MainMenu`

### 8.4 Интеграция с Adventure

- [x] `OnEnter(Adventure)` читает `GameStartConfig`, вызывает `load_map`, подставляет имя героя

### 8.5 Тесты (`src/data/mod.rs`)

- [x] `discover_maps_finds_default`
- [x] `valid_map_passes_validation`
- [x] `missing_unit_id_fails_validation`
- [x] `out_of_bounds_position_fails`
- [x] `object_on_obstacle_fails`

**Критерий готовности:** приложение стартует в меню; невалидный RON показывает ошибку без краша; игра стартует с выбранной картой и введённым именем.

---

## Этап 9 — Туман войны ✅

**Цель:** скрыть неисследованную карту; герой открывает область вокруг себя по мере движения.

### 9.1 Модель данных

- [x] `VisibilityState`: `Unexplored`, `Visited`, `Visible`
- [x] `AdventureMap::visibility: Vec<VisibilityState>` (параллельный массив)
- [x] `update_visibility(map, hero_pos, sight_range)` — квадратный радиус Чебышёва

### 9.2 Герой

- [x] Поле `Hero::sight_range: u32` (default 4)
- [x] `HeroDef::sight_range` с `#[serde(default)]` в формате карты

### 9.3 Рендер

- [x] Компонент `FogOverlay { pos }` — один спрайт на тайл, Z=2
- [x] `Unexplored` → чёрный непрозрачный; `Visited` → тёмный (α=0.55); `Visible` → прозрачный
- [x] Система `sync_fog_overlay` обновляет цвет оверлея при каждом ходе

### 9.4 Интеграция

- [x] `load_map` вызывает `update_visibility` для стартовой позиции героя
- [x] `input.rs`: `update_visibility` после каждого `MoveHero`

### 9.5 Тесты

- [x] `initial_fog_all_unexplored`
- [x] `update_visibility_reveals_area`
- [x] `moving_hero_marks_old_area_visited`
- [x] `visited_tiles_not_reset`
- [x] `load_map_reveals_hero_start`

**Критерий готовности:** карта при старте покрыта туманом; герой рассеивает его движением; посещённые тайлы остаются тёмными.

---

## Этап 10 — Выбор героя ⬜

**Цель:** дать выбор из ростера заранее определённых героев с уникальными характеристиками.

### 10.1 Ростер и данные

- [ ] `assets/data/heroes.ron` — 3 героя: Альдрик (⚔2/🛡1/👁4), Валерия (⚔1/🛡0/👁6), Горм (⚔3/🛡0/👁3)
- [ ] `HeroRosterDef { id, name, attack, defense, sight_range, portrait }` в `src/data/mod.rs`
- [ ] `load_heroes(path) -> Vec<HeroRosterDef>`

### 10.2 Модель и бой

- [ ] `Hero::attack` и `Hero::defense` — влияют на урон в бою
- [ ] `BattleState::attacker_attack/defense_bonus` — применяются в `attack()`

### 10.3 Портреты

- [ ] 3 SVG-портрета → PNG через `build.rs`

### 10.4 Меню и UI

- [ ] Панель выбора героя в `MainMenu` (3 карточки с портретами и характеристиками)
- [ ] Клик → предзаполнить имя; кнопка старта требует карту + героя + имя
- [ ] Портрет и характеристики героя в панели приключения

### 10.5 Тесты

- [ ] `load_heroes_parses`, `all_hero_ids_unique`
- [ ] `attack_bonus_increases_damage`, `defense_bonus_reduces_damage`, `defense_never_below_one`

**Критерий готовности:** разные герои дают разный начальный обзор и урон в бою.

---

## Этап 11 — Прогрессия героя ⬜

- [ ] Опыт за победу в бою
- [ ] Уровни героя (1–5)
- [ ] 2–3 навыка на выбор при повышении уровня (атака, защита, логистика)

---

## Этап 12 — Глубина боя ⬜

- [ ] Инициатива и скорость существ (порядок хода)
- [ ] Контратака (раз в раунд)
- [ ] Дальний бой (лучники, штраф в ближнем бою)
- [ ] Автобой

---

## Этап 13 — Ресурсы и шахты ⬜

- [ ] Несколько типов ресурсов (золото, дерево, руда)
- [ ] Шахты на карте (захват даёт ежедневный доход)
- [ ] Обновление экономики города под новые ресурсы

---

## Этап 14 — AI-противник ⬜

- [ ] Второй игрок под управлением AI
- [ ] Простой обход карты, захват ресурсов и шахт
- [ ] Оценка силы армий перед боем
- [ ] AI в бою: выбор оптимальной цели

---

## Этап 15 — Контент и полировка ⬜

- [ ] Больше типов существ и фракций
- [ ] Дерево зданий в городе
- [ ] Артефакты для героя
- [ ] Звук и музыка (`bevy_audio`)
- [ ] Процедурный генератор карт
- [ ] Hotseat для двух игроков

---

## Definition of Done

Фича считается готовой, если:

1. Она работает в игре.
2. Для чистой логики есть хотя бы один unit-тест.
3. Она не ломает существующую игровую петлю.
4. Код прошёл `cargo fmt` и `cargo clippy` без предупреждений.
5. Нет необработанных `unwrap()`, `panic!()`, `todo!()` в пользовательском пути.
