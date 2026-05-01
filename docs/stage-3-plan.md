# Stage 3 Plan — Ресурсы и экономика

**Цель:** первая полезная интеракция — собрать золото, завершить ход, получить доход.

**Статус:** ⬜ Не начато

---

## Что уже есть в проекте

### Готово в core (Этап 1)

| Элемент | Файл | Состояние |
|---------|------|-----------|
| `MapObject::ResourcePile(ResourceBag)` | `core/map.rs` | ✅ определён |
| `GameCommand::CollectResource { hero_id }` | `core/commands.rs` | ✅ реализован с логом |
| `GameCommand::EndTurn` | `core/commands.rs` | ✅ реализован с логом |
| `GameEvent::ResourceCollected { hero_id, amount }` | `core/commands.rs` | ✅ определён |
| `GameEvent::DayIncome { player_id, amount }` | `core/commands.rs` | ✅ определён |
| `GameEvent::TurnEnded { day }` | `core/commands.rs` | ✅ определён |
| `Town::income: ResourceBag` | `core/player.rs` | ✅ доход `250 gold` задан в render.rs |

### Частично готово в adventure (Этап 2)

| Элемент | Файл | Состояние |
|---------|------|-----------|
| `build_initial_game_state()` | `adventure/render.rs` | ⚠️ кучки золота не расставлены |
| `apply_move()` | `adventure/input.rs` | ⚠️ игнорирует `Ok(events)`, не запускает автосбор |
| Компоненты и UI-тексты | `adventure/mod.rs` | ⚠️ нет `ResourcePileMarker`, `GoldText`, `DayText` |

### Итог: вся бизнес-логика готова, нужно только подключить к визуалу.

---

## Что добавляется в этом этапе

### Новые компоненты (`adventure/mod.rs`)

```
ResourcePileMarker(Position)  — маркер спрайта кучки золота
GoldText                      — метка UI с текущим золотом
DayText                       — метка UI с текущим днём
```

### Изменения по файлам

```
adventure/mod.rs     — новые компоненты, регистрация новых систем в плагине
adventure/render.rs  — расстановка кучек на карте, их спрайты, новый UI
adventure/input.rs   — автосбор после MoveHero, обработка EndTurn (Space/Enter)
adventure/sync.rs    — sync_resource_piles, update_resource_ui, update_day_ui
```

---

## Задачи

### 3.1 Расстановка кучек золота на карте

- [ ] В `build_initial_game_state()` (`render.rs`) разместить 5 кучек `ResourcePile` на карте:

| Позиция | Количество золота |
|---------|------------------|
| `(5, 2)` | 100 |
| `(2, 6)` | 150 |
| `(8, 1)` | 200 |
| `(11, 6)` | 250 |
| `(4, 10)` | 100 |

  Использовать `map.get_mut(pos).unwrap().object = Some(MapObject::ResourcePile(ResourceBag::gold(N)))`.
- [ ] Обновить лог инициализации — добавить количество кучек к сообщению `[ADVENTURE] Game initialized`.

### 3.2 Новые компоненты (`adventure/mod.rs`)

- [ ] Объявить `ResourcePileMarker(pub Position)` — маркер спрайта кучки, хранит позицию для sync-системы.
- [ ] Объявить `GoldText` — маркер UI-текста с золотом.
- [ ] Объявить `DayText` — маркер UI-текста с днём.

### 3.3 Спавн спрайтов кучек (`adventure/render.rs`)

- [ ] В `startup_setup` после спавна тайлов пройти по всем тайлам карты.
- [ ] Для каждого тайла с `MapObject::ResourcePile` заспавнить спрайт:
  - цвет: `srgb(0.95, 0.75, 0.10)` — золотисто-жёлтый
  - размер: `Vec2::splat(TILE_SIZE * 0.5)` — меньше тайла
  - Z-слой: `1.0` (между тайлом и подсветкой ходов)
  - компонент: `ResourcePileMarker(pos)`

### 3.4 Синхронизация кучек: удаление собранных (`adventure/sync.rs`)

- [ ] Добавить систему `sync_resource_piles`:
  - запускается только когда `game_state.is_changed()`
  - итерировать сущности с `ResourcePileMarker`
  - для каждой проверить: `game_state.map.get(marker.0)` содержит ли ещё `ResourcePile`
  - если нет — `commands.entity(entity).despawn()`

### 3.5 Автосбор ресурса при входе на клетку (`adventure/input.rs`)

- [ ] Изменить `apply_move`: после успешного `MoveHero` проверить тайл на `target`.
- [ ] Если `map.get(target)` содержит `ResourcePile` — сразу применить `GameCommand::CollectResource { hero_id }`.
- [ ] Логировать ошибки сбора через `warn!` (не должны происходить, но на всякий случай).

> **Примечание:** `CollectResource` уже полностью реализован в core и пишет лог `[ADVENTURE] Hero collected...` самостоятельно.

### 3.6 Обработка завершения хода (`adventure/input.rs`)

- [ ] Добавить систему `handle_end_turn`:
  - реагирует на `just_pressed(KeyCode::Space)` или `just_pressed(KeyCode::Enter)`
  - применяет `GameCommand::EndTurn` через `game_state.0.apply(...)`
  - логирует ошибку через `warn!`, если `apply` вернул `Err`

### 3.7 UI золота (`adventure/render.rs` + `adventure/sync.rs`)

- [ ] В `startup_setup` заспавнить текст с компонентом `GoldText`:
  - начальный текст: `"Золото: 500"`
  - позиция: `top: 40px, left: 12px` (под строкой MP)
  - размер шрифта: `22.0`
- [ ] Добавить систему `update_resource_ui`:
  - запускается только когда `game_state.is_changed()`
  - читает `player.resources.gold` активного игрока
  - обновляет текст: `format!("Золото: {}", gold)`

### 3.8 UI текущего дня (`adventure/render.rs` + `adventure/sync.rs`)

- [ ] В `startup_setup` заспавнить текст с компонентом `DayText`:
  - начальный текст: `"День 1"`
  - позиция: `top: 12px, right: 12px`
  - размер шрифта: `22.0`
- [ ] Добавить систему `update_day_ui`:
  - запускается только когда `game_state.is_changed()`
  - читает `game_state.current_day`
  - обновляет текст: `format!("День {}", day)`

### 3.9 Обновить подсказку управления (`adventure/render.rs`)

- [ ] Дополнить текст подсказки внизу экрана:
  - было: `"WASD / стрелки — ход | ЛКМ — кликнуть клетку"`
  - стало: `"WASD / стрелки — ход | ЛКМ — кликнуть клетку | Space — завершить ход"`

### 3.10 Регистрация новых систем (`adventure/mod.rs`)

- [ ] Добавить `handle_end_turn` в цепочку систем `Update`:

```
keyboard_input
mouse_click_input
handle_end_turn          ← новая
update_hover_highlight
sync_hero_transform
sync_resource_piles      ← новая
update_available_moves
update_movement_ui
update_resource_ui       ← новая
update_day_ui            ← новая
```

### 3.11 Тесты

Бизнес-логика уже покрыта тестами в `core/commands.rs` (Этап 1). В этом этапе добавляем тест на coordinate round-trip уровня adventure:

- [ ] Убедиться что `cargo test` проходит без изменений (ничего неломаем).

---

## Z-слои спрайтов (итоговая таблица)

| Z | Содержимое |
|---|-----------|
| `0.0` | Тайлы карты |
| `1.0` | Кучки золота (новые) |
| `2.0` | Подсветка доступных ходов |
| `3.0` | Hover-подсветка |
| `4.0` | Герой |

---

## Что остаётся в ROADMAP без изменений

Этапы 4–7 не меняются. После завершения этапа обновить в `ROADMAP.md`:
- все задачи 3.1–3.5 пометить `[x]`
- заголовок этапа: `## Этап 3 — Ресурсы и экономика ✅`

---

## Критерий готовности

1. На карте видны 5 кучек золота.
2. Герой заходит на клетку с кучкой — она исчезает, баланс золота в UI увеличивается.
3. Нажатие `Space` завершает ход: MP восстанавливается, день увеличивается, золото прибавляется на 250.
4. `cargo test` — все тесты проходят.
5. `cargo clippy -- -D warnings` — нет предупреждений.
