# Stage 8 Plan — Экран запуска

**Цель:** перед началом игры показать главное меню — ввести имя героя и выбрать карту из доступных. Некорректная карта показывает ошибку, не ронит приложение.

**Зависит от:** Этап 7 (карты загружаются из файлов).

**Статус:** ⬜ Не начато

---

## Что уже есть в проекте

| Элемент | Файл | Состояние |
|---------|------|-----------|
| `GameScreen::{Adventure, Town, Battle, GameOver}` | `main.rs` | ✅ нужно добавить `MainMenu` |
| `load_map(map_path, units_path) -> GameState` | `src/data/mod.rs` | ✅ используется для загрузки выбранной карты |
| `respawn_map_objects` — спавн тайлов/объектов | `adventure/render.rs` | ✅ вызывается в `OnEnter(Adventure)` |
| Спавн камеры в `startup_setup` | `adventure/render.rs` | ✅ остаётся в `Startup` |
| `GameStateResource` — Bevy-обёртка над `GameState` | `adventure/mod.rs` | ✅ заполняется при старте игры |

### Итог: инфраструктура готова; нужны UI главного меню, скан карт и валидация.

---

## Схема переходов состояний

```
Старт приложения
  └── GameScreen::MainMenu  ← новый #[default]
        │
        │  поле ввода: имя героя
        │  список карт: выбрать валидную
        │  кнопка «Начать игру» (активна если имя ≠ "" и карта выбрана)
        │
        ▼
  GameScreen::Adventure  (загрузить выбранную карту, подставить имя героя)
        │
        └── ... (существующие переходы без изменений)

  «Играть снова» из GameOver → MainMenu  (не в Adventure напрямую)
```

---

## Задачи

### 8.1 Новое состояние (`main.rs`)

- [ ] Добавить `GameScreen::MainMenu` и сделать его `#[default]`:
  ```rust
  pub enum GameScreen {
      #[default]
      MainMenu,
      Adventure,
      Town,
      Battle,
      GameOver,
  }
  ```
- [ ] Убрать `#[default]` с `Adventure`.
- [ ] Зарегистрировать `MainMenuPlugin` в `main`.
- [ ] Обновить «Играть снова» в `gameover.rs` — переходить в `MainMenu` вместо `Adventure`.

---

### 8.2 Сканирование и валидация карт (`src/data/mod.rs`)

- [ ] Реализовать `pub fn discover_maps(maps_dir: &str) -> Vec<MapEntry>`:
  - Перебирает `*.ron` файлы в папке через `std::fs::read_dir`.
  - Для каждого файла вызывает `validate_map`.
  - Возвращает список записей — валидных и невалидных.

- [ ] Объявить:
  ```rust
  pub struct MapEntry {
      pub path:   String,       // путь к файлу
      pub result: MapEntryResult,
  }

  pub enum MapEntryResult {
      Valid(MapInfo),
      Invalid(String),          // человекочитаемое сообщение об ошибке
  }

  pub struct MapInfo {
      pub width:          u32,
      pub height:         u32,
      pub neutral_count:  usize,
      pub town_count:     usize,
  }
  ```

- [ ] Реализовать `fn validate_map(path: &str, units: &[UnitTypeDef]) -> Result<MapInfo, String>`:
  - Читает и парсит файл (`ron::from_str`) — ошибка парсинга → `Err`.
  - Все `StackRef.id` существуют в справочнике → иначе `Err("Unknown unit: goblin2")`.
  - Позиции объектов в пределах `width × height` → иначе `Err`.
  - Объекты не стоят на `Obstacle`/`Water` тайлах → иначе `Err`.
  - Ровно 1 стартовая позиция героя на проходимом тайле → иначе `Err`.
  - При успехе возвращает `MapInfo` с мета-информацией для отображения.

> Любая ошибка → `MapEntryResult::Invalid(msg)`, приложение **не паникует**.

---

### 8.3 Ресурс выбора (`main.rs` или `src/menu/mod.rs`)

- [ ] Объявить ресурс, заполняемый при старте игры:
  ```rust
  #[derive(Resource)]
  pub struct GameStartConfig {
      pub map_path:   String,
      pub hero_name:  String,
  }
  ```
- [ ] В `OnEnter(GameScreen::Adventure)` читать `GameStartConfig`, вызывать
  `load_map(&config.map_path, "assets/data/units.ron")`, подставлять `config.hero_name`
  в `hero.name` перед сохранением в `GameStateResource`.

---

### 8.4 Модуль главного меню (`src/menu/mod.rs`)

Новый файл. `MainMenuPlugin` регистрирует системы.

**Компоненты:**
```rust
#[derive(Component)] struct MenuRoot;
#[derive(Component)] struct HeroNameInput;
#[derive(Component)] struct MapListItem { path: String, valid: bool }
#[derive(Component)] struct StartButton;
```

**Локальный ресурс состояния меню:**
```rust
#[derive(Resource, Default)]
struct MenuState {
    hero_name:    String,
    selected_map: Option<String>,   // path
    maps:         Vec<MapEntry>,
}
```

**Системы:**

`setup_menu` (в `OnEnter(GameScreen::MainMenu)`):
- [ ] Полноэкранный фон.
- [ ] Заголовок `"Heroes of Rust"`.
- [ ] Поле ввода имени героя (placeholder `"Введите имя героя"`).
- [ ] Список карт: одна кнопка на файл.
  - Валидная карта: имя файла + `20×15 | нейтралов: 3` — кликабельна.
  - Невалидная карта: имя файла + сообщение об ошибке — серая, некликабельна.
- [ ] Кнопка `"Начать игру"` — изначально неактивна.
- [ ] Заполнить `MenuState.maps` через `data::discover_maps("assets/maps/")`.

`despawn_menu` (в `OnExit(GameScreen::MainMenu)`):
- [ ] `commands.entity(root).despawn_recursive()`.

`handle_hero_name_input` (в `Update`, `in_state(MainMenu)`):
- [ ] Считывать вводимые символы через `EventReader<KeyboardInput>` / `ReceivedCharacter`.
- [ ] Backspace удаляет последний символ.
- [ ] Обновлять текст `HeroNameInput` и `MenuState.hero_name`.
- [ ] Ограничение длины: 20 символов.

`handle_map_selection` (в `Update`, `in_state(MainMenu)`):
- [ ] При клике на `MapListItem { valid: true }` — записать путь в `MenuState.selected_map`.
- [ ] Визуально выделить выбранную карту (рамка или цвет).

`update_start_button` (в `Update`, `in_state(MainMenu)`):
- [ ] Кнопка активна если `!hero_name.is_empty() && selected_map.is_some()`.
- [ ] Менять цвет/прозрачность кнопки в зависимости от состояния.

`handle_start_button` (в `Update`, `in_state(MainMenu)`):
- [ ] При клике на активную `StartButton`:
  1. Вставить `GameStartConfig { map_path, hero_name }`.
  2. Перейти в `GameScreen::Adventure`.

---

### 8.5 Интеграция с Adventure (`adventure/mod.rs`, `adventure/render.rs`)

- [ ] В `OnEnter(GameScreen::Adventure)`:
  - Читать `GameStartConfig`.
  - Вызывать `load_map(...)` → `GameStateResource`.
  - Подставить `hero_name` из конфига.
  - Вызывать `respawn_map_objects` (уже существует).
- [ ] При переходе «Играть снова» → `MainMenu` (не `Adventure`) сбрасывать `GameStateResource` не нужно — он заполнится при следующем `OnEnter(Adventure)`.

---

### 8.6 Тесты (`src/data/mod.rs`)

- [ ] `discover_maps_finds_default` — `discover_maps("assets/maps/")` возвращает ≥1 записи.
- [ ] `valid_map_passes_validation` — `default.ron` проходит валидацию, `MapInfo` содержит корректные размеры.
- [ ] `missing_unit_id_fails_validation` — карта с `id: "dragon"` (нет в справочнике) → `Invalid`.
- [ ] `out_of_bounds_position_fails` — карта с объектом за пределами `width×height` → `Invalid`.
- [ ] `object_on_obstacle_fails` — карта с объектом на `Obstacle`-тайле → `Invalid`.

---

## Новые файлы

```
src/
└── menu/
    └── mod.rs   — MainMenuPlugin, MenuState, все системы меню
```

## Изменяемые файлы

```
src/main.rs             — +GameScreen::MainMenu (#[default]), +MainMenuPlugin
src/gameover.rs         — «Играть снова» → MainMenu вместо Adventure
src/data/mod.rs         — +MapEntry, +MapInfo, +discover_maps(), +validate_map()
src/adventure/mod.rs    — OnEnter(Adventure): читать GameStartConfig, load_map, подставить имя
src/adventure/render.rs — убрать build_initial_game_state из инициализации (заменяется конфигом)
ROADMAP.md              — добавить Этап 8, сдвинуть расширение в Этап 9
```

---

## Порядок реализации

1. `src/data/mod.rs` — `MapEntry`, `MapInfo`, `discover_maps`, `validate_map` + тесты
2. `main.rs` — `GameScreen::MainMenu`, `GameStartConfig`, `MainMenuPlugin`
3. `src/menu/mod.rs` — `setup_menu`, `despawn_menu`, логика ввода и выбора
4. `adventure/mod.rs` — `OnEnter(Adventure)` читает `GameStartConfig`
5. `gameover.rs` — «Играть снова» → `MainMenu`
6. `cargo test` — все тесты зелёные
7. `cargo clippy -- -D warnings` + `cargo fmt --check`
8. Коммит

---

## Критерий готовности

1. Приложение стартует на экране главного меню, не на карте.
2. Список карт отображает все `*.ron` файлы из `assets/maps/`.
3. Невалидный RON-файл в `assets/maps/` показывает понятную ошибку рядом с именем файла — приложение не падает.
4. После ввода имени и выбора карты кнопка «Начать игру» становится активной.
5. Игра стартует с введённым именем героя.
6. «Играть снова» возвращает в главное меню (не запускает игру немедленно).
7. `cargo test` — все тесты зелёные, включая 5 новых в `src/data/`.
8. `cargo clippy -- -D warnings` и `cargo fmt --check` — чисто.
