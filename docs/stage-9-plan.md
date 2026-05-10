# Stage 9 Plan — Туман войны

**Цель:** скрыть неисследованную карту; герой открывает область видимости по мере движения.
Непосещённые тайлы — чёрные, посещённые — тёмные (объекты сквозь туман видны), видимые (в зоне обзора) — нормальные.

**Зависит от:** Этап 8.

**Статус:** ⬜ Не начато

---

## Что уже есть в проекте

| Элемент | Файл | Состояние |
|---------|------|-----------|
| `AdventureMap { width, height, tiles }` | `src/core/map.rs` | ✅ расширяем массивом видимости |
| `TileMarker { pos }` на каждом тайле | `adventure/render.rs` | ✅ добавим компонент тумана |
| `respawn_map_objects` | `adventure/render.rs` | ✅ спавним туман здесь же |
| `sync_map_objects` | `adventure/sync.rs` | ✅ добавим `sync_fog_overlay` рядом |
| Z-порядок: тайл=0, объекты=1, герой=4 | `adventure/mod.rs` | ✅ туман займёт Z=2 |

---

## Концепция видимости

```
Unexplored  — чёрный непрозрачный оверлей (объекты скрыты)
Visited     — тёмный полупрозрачный оверлей (α≈0.6, объекты видны)
Visible     — оверлей невидим (нормальная отрисовка)
```

Туманный оверлей на Z=2 — выше объектов (Z=1), ниже подсветки хода (Z=3) и героя (Z=4).
При `Unexplored` непрозрачный туман перекрывает объекты без дополнительной логики скрытия.

**Зона обзора** считается по расстоянию Чебышёва (квадрат `sight_range×sight_range`):
```
расстояние_чебышёва(a, b) = max(|a.x - b.x|, |a.y - b.y|)
```
Значение по умолчанию: `sight_range = 4`.

---

## Задачи

### 9.1 Модель данных (`src/core/map.rs`)

- [ ] Добавить `VisibilityState`:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum VisibilityState {
      Unexplored,
      Visited,
      Visible,
  }
  ```
- [ ] Добавить поле в `AdventureMap`:
  ```rust
  pub struct AdventureMap {
      pub width:      u32,
      pub height:     u32,
      pub tiles:      Vec<Tile>,
      pub visibility: Vec<VisibilityState>,   // параллельный массив
  }
  ```
- [ ] Обновить `AdventureMap::new` — заполнять `visibility` значением `Unexplored`.
- [ ] Добавить методы:
  ```rust
  pub fn get_visibility(&self, pos: Position) -> Option<VisibilityState>
  pub fn set_visibility(&mut self, pos: Position, state: VisibilityState)
  ```

---

### 9.2 Логика обновления видимости (`src/core/map.rs`)

- [ ] Добавить `pub fn update_visibility(map: &mut AdventureMap, hero_pos: Position, sight_range: u32)`:
  1. Все `Visible` → `Visited` (герой покинул ту зону).
  2. Для всех позиций с чебышёвым расстоянием ≤ `sight_range` от `hero_pos` → `Visible`.

> Простой квадратный радиус без учёта препятствий — достаточно для этого этапа.

---

### 9.3 Поле `sight_range` у героя (`src/core/hero.rs`, `src/data/mod.rs`)

- [ ] Добавить `pub sight_range: u32` в `Hero` (default `4`).
- [ ] Добавить `#[serde(default = "default_sight_range")]` в `HeroDef`:
  ```rust
  fn default_sight_range() -> u32 { 4 }
  ```
- [ ] В `map_def_to_game_state` передавать `hero_def.sight_range` при создании `Hero`.
- [ ] После построения `GameState` в `load_map` вызвать `update_visibility` для стартовой позиции героя — карта изначально частично открыта.

---

### 9.4 Рендер тумана (`src/adventure/render.rs`, `src/adventure/mod.rs`)

**Компонент:**
```rust
#[derive(Component)]
pub struct FogOverlay {
    pub pos: Position,
}
```

**Спавн в `respawn_map_objects`:**
- [ ] Для каждого тайла спавнить спрайт тумана поверх тайла (Z=2):
  ```rust
  commands.spawn((
      Sprite {
          color: fog_color(visibility),
          custom_size: Some(Vec2::splat(TILE_SIZE)),
          ..default()
      },
      Transform::from_xyz(world.x, world.y, 2.0),
      FogOverlay { pos },
  ));
  ```
  где `fog_color`:
  ```rust
  fn fog_color(state: VisibilityState) -> Color {
      match state {
          VisibilityState::Unexplored => Color::srgba(0.0, 0.0, 0.0, 1.0),
          VisibilityState::Visited    => Color::srgba(0.0, 0.0, 0.0, 0.55),
          VisibilityState::Visible    => Color::srgba(0.0, 0.0, 0.0, 0.0),
      }
  }
  ```

**Система `sync_fog_overlay` (в `adventure/sync.rs`, `in_state(Adventure)`):**
- [ ] Запрашивать `Query<(&FogOverlay, &mut Sprite)>` + `Res<GameStateResource>`.
- [ ] Для каждого тайла обновлять `sprite.color` по текущему `visibility[pos]`.
- [ ] Запускать после `sync_map_objects` (в `.chain()`).

---

### 9.5 Интеграция с вводом (`src/adventure/input.rs`)

- [ ] После каждого успешного `MoveHero` вызывать `update_visibility`:
  ```rust
  let hero = gs.get_hero(hero_id)?;
  update_visibility(&mut gs.map, hero.position, hero.sight_range);
  ```
- [ ] Добавить `sync_fog_overlay` в цепочку систем `Update` в `AdventurePlugin`.

---

### 9.6 Тесты (`src/core/map.rs`)

- [ ] `initial_fog_all_unexplored` — свежая `AdventureMap::new` имеет все тайлы `Unexplored`.
- [ ] `update_visibility_reveals_area` — после `update_visibility(pos=(1,1), range=2)` тайлы в радиусе 2 — `Visible`, остальные — `Unexplored`.
- [ ] `moving_hero_marks_old_area_visited` — второй вызов `update_visibility` с другой позицией переводит первую зону в `Visited`, новую — в `Visible`.
- [ ] `visited_tiles_not_reset` — тайл, ставший `Visited`, не переходит обратно в `Unexplored` при последующих обновлениях.
- [ ] `load_map_reveals_hero_start` — `load_map("assets/maps/default.ron", ...)` возвращает `GameState`, в котором тайл (1,1) — `Visible`.

---

## Новые файлы

Нет — только изменения в существующих.

## Изменяемые файлы

```
src/core/map.rs          — +VisibilityState, +visibility в AdventureMap, +update_visibility
src/core/hero.rs         — +sight_range: u32
src/data/mod.rs          — +sight_range в HeroDef (#[serde(default)]), вызов update_visibility в load_map
src/adventure/mod.rs     — +FogOverlay компонент
src/adventure/render.rs  — спавн FogOverlay в respawn_map_objects
src/adventure/sync.rs    — +sync_fog_overlay система
src/adventure/input.rs   — вызов update_visibility после MoveHero
```

---

## Порядок реализации

1. `src/core/map.rs` — `VisibilityState`, поле `visibility`, методы, `update_visibility` + тесты
2. `src/core/hero.rs` — поле `sight_range`
3. `src/data/mod.rs` — `sight_range` в `HeroDef`, вызов `update_visibility` в `load_map`; тест `load_map_reveals_hero_start`
4. `src/adventure/mod.rs` — `FogOverlay`
5. `src/adventure/render.rs` — спавн оверлея в `respawn_map_objects`
6. `src/adventure/sync.rs` — `sync_fog_overlay`
7. `src/adventure/input.rs` — `update_visibility` после хода
8. `cargo test` — все тесты зелёные
9. `cargo clippy -- -D warnings` + `cargo fmt --check`
10. Коммит

---

## Критерий готовности

1. При запуске игры большая часть карты покрыта чёрным туманом; видна только стартовая область героя.
2. По мере движения героя туман рассеивается вокруг него.
3. Посещённые тайлы остаются тёмными (но различимыми) после ухода героя.
4. Объекты (нейтралы, золото) на непосещённых тайлах не видны сквозь непрозрачный туман.
5. `cargo test` — все тесты зелёные, включая 5 новых в `src/core/map.rs`.
6. `cargo clippy -- -D warnings` и `cargo fmt --check` — чисто.
