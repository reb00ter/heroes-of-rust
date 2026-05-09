# Stage 6 Plan — Вертикальный срез

**Цель:** собрать играбельную мини-игру от старта до финала — победить всех нейтральных существ и увидеть экран «Победа!».

**Статус:** ⬜ Не начато

---

## Что уже есть в проекте

| Элемент | Файл | Состояние |
|---------|------|-----------|
| Карта 16×12, 2 нейтральных отряда | `adventure/render.rs` | ✅ нужно расширить |
| Герой стартует с пустой армией | `adventure/render.rs:134` | ✅ нужно добавить стартовую армию |
| `GameScreen::{Adventure, Town, Battle}` | `main.rs:17` | ✅ нужно добавить `GameOver` |
| `finish_battle` — победа/поражение | `battle/mod.rs:87` | ✅ нужно расширить |
| Баннер `ShowBanner` (временный оверлей) | `adventure/mod.rs:78` | ✅ остаётся без изменений |
| `BattleResult`, `PendingBattle` ресурсы | `main.rs`, `battle/mod.rs` | ✅ без изменений |

### Итог: вся механика работает; нужны карта, стартовая армия, условие победы и экран финала.

---

## Что добавляется в этом этапе

### Новые файлы

```
src/gameover.rs   — GameOverPlugin, spawn/despawn UI, кнопка «Играть снова»
```

### Изменяемые файлы

```
src/main.rs                — добавить GameScreen::GameOver, GameOverResult, GameOverPlugin
src/adventure/mod.rs       — добавить NeedsMapReset resource, maybe_reset_map system
src/adventure/render.rs    — карта 20×15, 3 нейтрала, стартовая армия героя
src/battle/mod.rs          — finish_battle: победный check, очистка армии при поражении,
                             переход в GameOver
src/core/state.rs          — добавить count_neutral_armies()
ROADMAP.md                 — отметить выполненные задачи
```

---

## Схема переходов состояний

```
Старт
  └── GameScreen::Adventure (hero с 5 крестьянами, карта 20×15)
         │
         │ герой входит в город → Town → рекрутинг → Adventure
         │ герой входит на NeutralArmy → Battle
         │
         ▼
      GameScreen::Battle
         │
         ├── Победа атакующего:
         │     ├── остались нейтралы → Adventure + баннер «Победа!»
         │     └── нейтралов нет → GameScreen::GameOver (Victory)
         │
         └── Победа защитника:
               армия героя очищается → GameScreen::GameOver (Defeat)

      GameScreen::GameOver
         │
         └── кнопка «Играть снова» → сброс состояния → Adventure
```

---

## Задачи

### 6.1 Карта 20×15 (`adventure/render.rs`, `adventure/mod.rs`)

Расширить карту с 16×12 до 20×15 и перебалансировать расстановку объектов.

**Изменить константы** в `adventure/mod.rs`:
```rust
pub const MAP_WIDTH:  u32 = 20;
pub const MAP_HEIGHT: u32 = 15;
```

**Новая расстановка объектов** в `build_initial_game_state`:

| Объект | Позиция | Параметры |
|--------|---------|-----------|
| Герой | (1, 1) | без изменений |
| Город | (4, 2) | перенести с (12,5); ближе к старту |
| Золото 1 | (2, 5, 150) | юг от старта |
| Золото 2 | (7, 1, 200) | восток |
| Золото 3 | (10, 9, 250) | центр карты |
| Золото 4 | (15, 3, 300) | дальний восток |
| Золото 5 | (18, 12, 200) | угол карты |
| Нейтрал 1 (лёгкий) | (7, 5) | Гоблин ×5 (dmg=2, hp=5) |
| Нейтрал 2 (средний) | (13, 6) | Орк ×4 (dmg=4, hp=10) |
| Нейтрал 3 (сложный) | (17, 10) | Тролль ×2 (dmg=8, hp=25) |

**Новые типы существ** (только для нейтральных, стоимость 0):
```rust
let goblin = UnitType { name: "Гоблин".to_string(),  damage_per_unit: 2, hp: 5,  cost: ResourceBag::gold(0) };
let orc    = UnitType { name: "Орк".to_string(),     damage_per_unit: 4, hp: 10, cost: ResourceBag::gold(0) };
let troll  = UnitType { name: "Тролль".to_string(),  damage_per_unit: 8, hp: 25, cost: ResourceBag::gold(0) };
```

**Препятствия** — перегруппировать для нового размера карты:
```rust
let obstacles = [
    // Группа 1 — скалы севернее старта
    (3, 0), (4, 0), (3, 1),
    // Группа 2 — центральный барьер
    (9, 3), (9, 4), (9, 5),
    // Группа 3 — восточный регион
    (14, 1), (14, 2), (15, 8), (15, 9),
    // Группа 4 — юг
    (6, 12), (7, 12), (7, 13),
    (11, 11), (12, 11),
];
let water = [
    (0, 13), (0, 14), (1, 14), (2, 14),
    (19, 0), (19, 1), (18, 0),
    (18, 14), (19, 14), (19, 13),
];
```

> **Примечание:** точные позиции препятствий подбираются так, чтобы оставить проходимые маршруты ко всем трём нейтральным отрядам.

### 6.2 Стартовое состояние (`adventure/render.rs`)

- [ ] Добавить стартовую армию герою — 5 Крестьян:
  ```rust
  let peasant_starter = UnitType {
      name: "Крестьянин".to_string(),
      damage_per_unit: 1,
      hp: 5,
      cost: ResourceBag::gold(25),
  };
  let mut hero = Hero { id: HeroId(0), name: "Aldric".to_string(), ... };
  hero.army.add_stack(UnitStack::new(peasant_starter, 5));
  ```
- [ ] Начальное золото: 500 (уже есть — без изменений).
- [ ] Начальный доход города: 250/день (уже есть — без изменений).
- [ ] Логировать состав начальной армии при инициализации.

### 6.3 Условие победы (`core/state.rs`)

- [ ] Добавить метод в `GameState`:
  ```rust
  pub fn count_neutral_armies(&self) -> usize {
      self.map.tiles.iter()
          .filter(|t| matches!(t.object, Some(MapObject::NeutralArmy(_))))
          .count()
  }
  ```
- [ ] Добавить unit-тест `victory_when_all_neutrals_dead`:
  - создать `GameState`, убрать все `NeutralArmy` с карты вручную
  - проверить что `count_neutral_armies() == 0`

### 6.4 Ресурс результата игры (`main.rs`)

- [ ] Добавить `GameScreen::GameOver` в enum:
  ```rust
  pub enum GameScreen {
      #[default]
      Adventure,
      Town,
      Battle,
      GameOver,
  }
  ```
- [ ] Объявить ресурс:
  ```rust
  #[derive(Resource)]
  pub struct GameOverResult {
      pub is_victory: bool,
  }
  ```
- [ ] Зарегистрировать `GameOverPlugin` в `main`.

### 6.5 Расширить `finish_battle` (`battle/mod.rs`)

- [ ] Добавить `next_state: ResMut<NextState<GameScreen>>` как параметр.
- [ ] **При победе атакующего** (после обновления карты и армии):
  ```rust
  if game_state.0.count_neutral_armies() == 0 {
      commands.insert_resource(GameOverResult { is_victory: true });
      // не устанавливать next_state здесь — Adventure установит его сам при OnEnter
  }
  ```
- [ ] **При победе защитника**:
  - Очистить армию героя: `hero.army.0.clear()`
  - Вставить `GameOverResult { is_victory: false }`
  - Логировать: `[BATTLE] Defeat! Hero army cleared. Game over.`
- [ ] Убрать `ShowBanner::Victory` / `ShowBanner::Defeat` если переходим в GameOver
  (баннер не нужен, когда сразу открывается экран финала).

### 6.6 Переход в GameOver из Adventure (`adventure/mod.rs`)

- [ ] Добавить систему `check_game_over` в `OnEnter(GameScreen::Adventure)`:
  ```rust
  fn check_game_over(
      game_over: Option<Res<GameOverResult>>,
      mut next_state: ResMut<NextState<GameScreen>>,
  ) {
      if game_over.is_some() {
          next_state.set(GameScreen::GameOver);
      }
  }
  ```
- [ ] Поставить `check_game_over` **перед** `show_result_banner` в `.chain()` в `OnEnter`.

### 6.7 Экран финала (`src/gameover.rs`)

**Структура UI:**
```
┌──────────────────────────────────────────────┐
│                                              │
│           ПОБЕДА! / ПОРАЖЕНИЕ                │
│                                              │
│          [ Играть снова ]                    │
│                                              │
└──────────────────────────────────────────────┘
```

- [ ] Создать файл `src/gameover.rs`, объявить `GameOverPlugin`.

**Компоненты:**
```rust
#[derive(Component)] struct GameOverRoot;
#[derive(Component)] struct PlayAgainButton;
```

**Система `spawn_gameover_ui`** (в `OnEnter(GameScreen::GameOver)`):
- [ ] Тёмный полноэкранный фон `rgba(0.0, 0.0, 0.0, 0.85)`, `GlobalZIndex(200)`.
- [ ] Текст: `"ПОБЕДА!"` зелёным или `"ПОРАЖЕНИЕ"` красным, размер 96px.
- [ ] Кнопка `"Играть снова"`: белый фон, тёмный текст, 200×60px.
- [ ] Читать `GameOverResult` для определения текста и цвета.

**Система `despawn_gameover_ui`** (в `OnExit(GameScreen::GameOver)`):
- [ ] `commands.entity(root).despawn_recursive()`

**Система `handle_gameover_input`** (в `Update`, `in_state(GameOver)`):
- [ ] При клике на кнопку `PlayAgainButton`:
  1. Вставить ресурс `NeedsMapReset` (см. 6.8).
  2. Сбросить `GameStateResource` → `GameStateResource(build_initial_game_state())`.
  3. Убрать ресурс `GameOverResult`.
  4. Убрать `ShowBanner` → `ShowBanner(None)`.
  5. Перейти в `GameScreen::Adventure`.

**Регистрация:**
```rust
impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScreen::GameOver), spawn_gameover_ui)
           .add_systems(OnExit(GameScreen::GameOver),  despawn_gameover_ui)
           .add_systems(Update, handle_gameover_input
               .run_if(in_state(GameScreen::GameOver)));
    }
}
```

### 6.8 Сброс карты при перезапуске (`adventure/mod.rs`)

После нажатия «Играть снова» нужно вернуть спрайты объектов карты в исходное состояние.

- [ ] Объявить ресурс:
  ```rust
  #[derive(Resource, Default)]
  pub struct NeedsMapReset;
  ```
- [ ] Инициализировать: `app.init_resource::<NeedsMapReset>()`... нет, **не добавлять** в `init_resource` — ресурс вставляется вручную при перезапуске.
- [ ] Добавить систему `reset_map_on_restart` (в `OnEnter(GameScreen::Adventure)`):
  ```rust
  fn reset_map_on_restart(
      mut commands: Commands,
      reset: Option<Res<NeedsMapReset>>,
      game_state: Res<GameStateResource>,
      pile_q: Query<Entity, With<ResourcePileMarker>>,
      neutral_q: Query<Entity, With<NeutralArmyMarker>>,
      hero_q: Query<Entity, With<HeroMarker>>,
  ) {
      let Some(_) = reset else { return };
      commands.remove_resource::<NeedsMapReset>();

      // Despawn старых спрайтов
      for e in &pile_q  { commands.entity(e).despawn(); }
      for e in &neutral_q { commands.entity(e).despawn(); }
      for e in &hero_q  { commands.entity(e).despawn(); }

      // Spawn новых из сброшенного GameState (по той же логике что в startup_setup)
      respawn_map_objects(&mut commands, &game_state.0);
  }
  ```
- [ ] Вспомогательная функция `respawn_map_objects` — выносит логику спавна ResourcePile / NeutralArmy / Hero из `startup_setup` в переиспользуемую форму (или дублирует inline — на усмотрение реализации).
- [ ] Поставить `reset_map_on_restart` **перед** `check_game_over` в `OnEnter`.

### 6.9 Полировка петли (6.5 в ROADMAP)

- [ ] Проверить переходы: Adventure → Town → Adventure → Battle → Adventure → GameOver → Adventure.
- [ ] Проверить что `EndTurn` (Space/Enter) корректно обновляет все системы: MP, золото, день, доступные рекруты.
- [ ] Проверить что герой не может атаковать нейтрала без армии (уже реализовано — проверить что лог появляется).
- [ ] Проверить что нельзя нанять больше существ, чем есть в городе.
- [ ] Проверить что нельзя нанять существ без нужного золота.
- [ ] Убедиться что `cargo build`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` — все зелёные.

---

## Новые тесты

### `core/state.rs`
- [ ] `victory_when_all_neutrals_dead` — убрать все `NeutralArmy` с тайлов, проверить `count_neutral_armies() == 0`
- [ ] `neutrals_counted_correctly` — карта с 3 нейтралами, `count_neutral_armies() == 3`, убрать 1 → `== 2`

### `adventure/render.rs`
- [ ] `hero_has_starter_army` — `build_initial_game_state()` возвращает героя с непустой армией
- [ ] `three_neutral_armies_on_map` — карта содержит ровно 3 `NeutralArmy` объекта
- [ ] `town_near_start` — город находится в позиции `(4, 2)`

---

## Структура файлов после этапа

```
src/
├── main.rs               — +GameScreen::GameOver, +GameOverResult, +GameOverPlugin
├── gameover.rs           — GameOverPlugin, spawn/despawn UI, PlayAgainButton
├── core/
│   └── state.rs          — +count_neutral_armies()
├── adventure/
│   ├── mod.rs            — +NeedsMapReset, +reset_map_on_restart, +check_game_over
│   └── render.rs         — карта 20×15, 3 нейтрала, стартовая армия
└── battle/
    └── mod.rs            — finish_battle: victory check, army clear, GameOverResult
```

---

## Ожидаемый игровой сценарий (5–10 минут)

1. **День 1:** Герой (5 крестьян, 500 золота) выходит на карту. Идёт в город (4,2), нанимает 10 крестьян (250 золота). По пути собирает золото (150 г). Завершает ход.
2. **День 2:** MP восстановлены. Идёт к гоблинам (7,5). Бой: 15 крестьян vs 5 гоблинов → победа. Баннер «Победа!». Возвращается к городу или золоту.
3. **День 3–4:** Нанимает мечников (75г/шт). Идёт к оркам (13,6). Бой: крестьяне + мечники vs 4 орка → победа.
4. **День 5–7:** Рекрутирует ещё войска. Идёт к троллям (17,10). Бой: армия vs 2 тролля → победа.
5. **Финал:** Все нейтральные побеждены. Экран «ПОБЕДА!». Кнопка «Играть снова».

---

## Критерий готовности

1. `cargo run` — открывается карта 20×15 с тремя нейтральными отрядами, героем и городом.
2. Герой имеет стартовую армию (5 крестьян).
3. После победы во всех трёх боях появляется экран «ПОБЕДА!».
4. После поражения в бою появляется экран «ПОРАЖЕНИЕ».
5. Кнопка «Играть снова» возвращает в начало с чистым состоянием.
6. `cargo test` — все тесты проходят, включая новые в `core/state.rs` и `adventure/render.rs`.
7. `cargo clippy -- -D warnings` — нет предупреждений.
8. `cargo fmt --check` — код отформатирован.
