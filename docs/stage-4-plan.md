# Stage 4 Plan — Город и найм существ

**Цель:** герой заходит в город, тратит золото, нанимает существ, видит армию.

**Статус:** ⬜ Не начато

---

## Что уже есть в проекте

### Готово в core

| Элемент | Файл | Состояние |
|---------|------|-----------|
| `MapObject::Town(TownId)` | `core/map.rs` | ✅ определён |
| `Town { position, income, garrison, available_recruits }` | `core/player.rs` | ✅ определён |
| `GameCommand::RecruitUnits { town_id, unit_type_idx, count }` | `core/commands.rs` | ✅ реализован с логом |
| `CommandError::NotInTown`, `InsufficientFunds`, `ArmyFull` и др. | `core/commands.rs` | ✅ определены |
| `UnitType { name, damage_per_unit, hp, cost }` | `core/hero.rs` | ✅ определён |

### Частично готово

| Элемент | Файл | Состояние |
|---------|------|-----------|
| Город создан в `build_initial_game_state` | `adventure/render.rs` | ⚠️ `available_recruits` пуст, тайл без `MapObject::Town` |
| `cmd_end_turn` | `core/commands.rs` | ⚠️ не пополняет `available_recruits` |
| `apply_move` | `adventure/input.rs` | ⚠️ не обрабатывает вход на тайл с городом |

### Итог: вся бизнес-логика `RecruitUnits` готова. Нужны: данные существ, экран города, переходы состояний.

---

## Что добавляется в этом этапе

### Новый модуль

```
src/town/mod.rs   — TownPlugin, компоненты экрана города
src/town/ui.rs    — спавн/деспавн UI города, обработка кнопок
```

### Новые файлы и изменения

```
src/main.rs                — GameScreen states, подключение TownPlugin
src/adventure/mod.rs       — TownMarker, ArmyText; adventure-системы в состоянии Adventure
src/adventure/render.rs    — MapObject::Town на тайл, спрайт города, данные существ
src/adventure/input.rs     — обнаружение входа в город, переход в GameScreen::Town
src/adventure/sync.rs      — sync_town_marker (опционально), update_army_ui
src/core/commands.rs       — пополнение available_recruits в cmd_end_turn
```

---

## Задачи

### 4.1 Типы существ и данные города (`adventure/render.rs`)

- [ ] Определить два типа существ прямо в `build_initial_game_state`:

| Существо | Стоимость | Урон | HP | Доступно | Прирост/день |
|----------|-----------|------|----|----------|--------------|
| Крестьянин | 25 gold | 1 | 5 | 10 | 5 |
| Мечник | 75 gold | 3 | 10 | 5 | 2 |

- [ ] Добавить их в `town.available_recruits` как `Vec<(UnitType, u32)>`.
- [ ] Поставить `MapObject::Town(TownId(0))` на тайл `(12, 5)`:
  ```rust
  map.get_mut(Position::new(12, 5)).unwrap().object = Some(MapObject::Town(TownId(0)));
  ```

### 4.2 Спрайт города на карте (`adventure/render.rs` + `adventure/mod.rs`)

- [ ] Объявить компонент `TownMarker(TownId)` в `adventure/mod.rs`.
- [ ] В `startup_setup` обойти тайлы, найти `MapObject::Town` и заспавнить спрайт:
  - цвет: `srgb(0.55, 0.40, 0.80)` — фиолетовый, хорошо отличается от зелёного и жёлтого
  - размер: `Vec2::splat(TILE_SIZE * 0.8)` — крупнее кучки золота
  - Z-слой: `1.0`
  - компонент: `TownMarker(town_id)`

### 4.3 Состояния экрана (`src/main.rs`)

- [ ] Объявить enum `GameScreen` и зарегистрировать его как State Bevy:
  ```rust
  #[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
  pub enum GameScreen { #[default] Adventure, Town }
  ```
- [ ] В `main.rs`: `app.init_state::<GameScreen>()`.
- [ ] Объявить ресурс `CurrentTownId(pub TownId)` в `src/town/mod.rs` для передачи контекста.

### 4.4 Adventure-системы: ограничить состоянием (`adventure/mod.rs`)

- [ ] Перевести цепочку `Update`-систем приключения на `.run_if(in_state(GameScreen::Adventure))`.

> Это предотвратит обработку ввода карты, пока открыт экран города.

### 4.5 Обнаружение входа в город (`adventure/input.rs`)

- [ ] В `apply_move`: после успешного `MoveHero` проверить тайл `target`.
- [ ] Если `MapObject::Town(town_id)` — записать `commands.insert_resource(CurrentTownId(town_id))` и перейти в `NextState(GameScreen::Town)`.
- [ ] `apply_move` должен принять дополнительные параметры: `next_state: ResMut<NextState<GameScreen>>`, `commands: Commands`.

### 4.6 Пополнение существ при `EndTurn` (`core/commands.rs`)

- [ ] В `cmd_end_turn` добавить шаг: для каждого города пополнять `available_recruits`.
- [ ] Прирост хранить в `Town` как `replenish_rates: Vec<u32>` — по одному значению на слот `available_recruits`.
  - Альтернатива проще: добавить поле `pub daily_growth: Vec<u32>` в `Town`.
- [ ] Обновить `Town::new()` и `build_initial_game_state()` соответственно.
- [ ] Добавить лог: `[TOWN] Daily growth: +5 Крестьянин, +2 Мечник`.

### 4.7 Экран города — UI (`src/town/ui.rs`)

Экран появляется поверх карты приключений (карта не деспавнится, просто не принимает ввод).

**Структура UI:**
```
┌─────────────────────────────────────┐
│  Замок                  [Покинуть]  │
│  Золото: 500                        │
├─────────────────────────────────────┤
│  Найм существ:                      │
│  Крестьянин  25g  HP:5  Dmg:1  ×10  [Нанять 1]  │
│  Мечник      75g  HP:10 Dmg:3  ×5   [Нанять 1]  │
├─────────────────────────────────────┤
│  Армия героя:                       │
│  (пусто)                            │
└─────────────────────────────────────┘
```

- [ ] Объявить компонент-маркер `TownScreenRoot` для корневого узла UI.
- [ ] Система `spawn_town_ui` (запускается в `OnEnter(GameScreen::Town)`):
  - тёмный полупрозрачный фон `rgba(0.05, 0.05, 0.1, 0.95)`
  - заголовок «Замок»
  - отображение текущего золота
  - список существ: строка на каждый тип из `town.available_recruits`
  - кнопка `Нанять 1` для каждого типа
  - секция армии героя
  - кнопка «Покинуть город»
- [ ] Система `despawn_town_ui` (запускается в `OnExit(GameScreen::Town)`):
  - `commands.entity(root).despawn_recursive()`

### 4.8 Обработка кнопок в городе (`src/town/ui.rs`)

- [ ] Компонент `HireButton(usize)` — хранит индекс существа в `available_recruits`.
- [ ] Система `handle_town_input` (в состоянии `GameScreen::Town`):
  - Обработать клик по `HireButton(idx)`:
    - получить `CurrentTownId`
    - получить id героя активного игрока
    - применить `GameCommand::RecruitUnits { town_id, unit_type_idx: idx, count: 1 }`
    - при успехе — пересоздать UI (или обновить текст в нужных узлах)
  - Обработать клик по «Покинуть город» или клавишу `Escape`:
    - перейти в `NextState(GameScreen::Adventure)`

### 4.9 Отображение армии на карте (`adventure/mod.rs` + `adventure/sync.rs`)

- [ ] Объявить компонент `ArmyText`.
- [ ] В `startup_setup` заспавнить текстовый UI-элемент:
  - позиция: `top: 68px, left: 12px` (под строкой «Золото»)
  - начальный текст: `"Армия: (пусто)"`
- [ ] Система `update_army_ui` (в `sync.rs`):
  - запускается при `game_state.is_changed()`
  - формирует строку: `"Армия: Мечник ×3, Крестьянин ×5"`

### 4.10 Регистрация систем (`src/main.rs` + `src/town/mod.rs`)

- [ ] В `TownPlugin`:
  ```rust
  app
    .add_systems(OnEnter(GameScreen::Town), ui::spawn_town_ui)
    .add_systems(OnExit(GameScreen::Town),  ui::despawn_town_ui)
    .add_systems(Update, ui::handle_town_input.run_if(in_state(GameScreen::Town)));
  ```
- [ ] `update_army_ui` добавить в цепочку `AdventurePlugin` (в состоянии `Adventure`).

---

## Схема переходов состояний

```
GameScreen::Adventure
    │
    │  герой заходит на тайл с городом
    ▼
GameScreen::Town
    │
    │  клик «Покинуть город» или Escape
    ▼
GameScreen::Adventure
```

---

## Структура файлов после этапа

```
src/
├── main.rs              — GameScreen, TownPlugin
├── core/
│   ├── commands.rs      — пополнение recruits в EndTurn
│   └── player.rs        — поле daily_growth в Town
├── adventure/
│   ├── mod.rs           — TownMarker, ArmyText; run_if(Adventure)
│   ├── render.rs        — MapObject::Town, спрайт, юниты в городе
│   ├── input.rs         — обнаружение города, NextState
│   └── sync.rs          — update_army_ui
└── town/
    ├── mod.rs           — TownPlugin, CurrentTownId, HireButton
    └── ui.rs            — spawn/despawn/handle town UI
```

---

## Новые тесты (`core/commands.rs`)

- [ ] Тест: `recruit_units_hero_not_in_town` — герой не на клетке города → `Err(NotInTown)`
- [ ] Тест: `end_turn_replenishes_recruits` — после `EndTurn` количество доступных существ увеличивается на `daily_growth`

---

## Что остаётся в ROADMAP без изменений

Этапы 5–7 не меняются. После завершения обновить в `ROADMAP.md`:
- задачи 4.1–4.5 пометить `[x]`
- заголовок: `## Этап 4 — Город и найм существ ✅`

---

## Критерий готовности

1. На карте виден фиолетовый маркер города в `(12, 5)`.
2. Герой заходит на клетку города — открывается экран найма.
3. Игрок нанимает существ — золото уменьшается, армия обновляется.
4. Кнопка «Покинуть город» / `Escape` возвращает на карту.
5. `EndTurn` пополняет доступных существ в городе.
6. `cargo test` — все тесты проходят, включая два новых.
7. `cargo clippy -- -D warnings` — нет предупреждений.
