# Stage 5 Plan — Простой бой

**Цель:** герой атакует нейтральный отряд, проходит пошаговый бой, побеждает и возвращается на карту.

**Статус:** ⬜ Не начато

---

## Что уже есть в проекте

| Элемент | Файл | Состояние |
|---------|------|-----------|
| `MapObject::NeutralArmy(Army)` | `core/map.rs:43` | ✅ определён |
| `GameEvent::BattleStarted { attacker, defender_pos }` | `core/commands.rs:48` | ✅ определён |
| Генерация `BattleStarted` при входе на клетку с `NeutralArmy` | `core/commands.rs:168` | ✅ реализовано |
| `is_passable` блокирует тайл с `NeutralArmy` | `core/map.rs:108` | ✅ реализовано |
| `GameScreen` state-машина (`Adventure`, `Town`) | `main.rs:13` | ✅ есть, нужно расширить |
| `Army`, `UnitStack`, `UnitType` | `core/hero.rs` | ✅ полностью готовы |

### Итог: событие боя уже генерируется ядром. Нужны: переход состояний, модуль боя, UI, AI, завершение.

---

## Что добавляется в этом этапе

### Новый модуль

```
src/battle/mod.rs    — BattlePlugin, компоненты
src/battle/state.rs  — BattleState, BattleStack, Side, StackId
src/battle/render.rs — отрисовка поля боя
src/battle/input.rs  — обработка ввода игрока в бою
```

### Изменяемые файлы

```
src/main.rs              — добавить GameScreen::Battle, BattlePlugin, PendingBattle
src/adventure/input.rs   — обрабатывать BattleStarted, переходить в GameScreen::Battle
src/adventure/render.rs  — маркеры нейтральных отрядов, удаление после победы
src/core/commands.rs     — новый GameCommand::RemoveNeutralArmy, GameEvent::BattleEnded
```

---

## Схема переходов состояний

```
GameScreen::Adventure
    │
    │  герой входит на тайл с NeutralArmy
    │  → BattleStarted event
    │  → записать PendingBattle в ресурс
    ▼
GameScreen::Battle
    │
    │  одна из сторон уничтожена → BattleEnded
    ├── Победа атакующего → удалить NeutralArmy с карты,
    │                        обновить армию героя (потери)
    │                        → GameScreen::Adventure + баннер "Победа!"
    └── Победа защитника → заморозить состояние + баннер "Поражение"
```

---

## Задачи

### 5.1 Нейтральные отряды на карте (`adventure/render.rs`)

- [ ] В `build_initial_game_state` разместить 2 нейтральных отряда:

| Позиция | Состав отряда |
|---------|---------------|
| `(5, 3)` | `Army` с 1 стеком: Гоблин ×5 (урон 2, HP 5, стоимость 0) |
| `(10, 7)` | `Army` с 1 стеком: Орк ×3 (урон 4, HP 10, стоимость 0) |

```rust
// Стоимость 0 — нейтральные не нанимаются
let goblin = UnitType { name: "Гоблин".to_string(), damage_per_unit: 2, hp: 5, cost: ResourceBag::gold(0) };
map.get_mut(Position::new(5, 3)).unwrap().object =
    Some(MapObject::NeutralArmy(Army(vec![UnitStack::new(goblin, 5)])));
```

- [ ] Объявить компонент `NeutralArmyMarker(Position)` в `adventure/mod.rs`.
- [ ] В `startup_setup` обойти тайлы, найти `MapObject::NeutralArmy` и заспавнить спрайт:
  - цвет: `srgb(0.85, 0.15, 0.15)` — красный
  - размер: `Vec2::splat(TILE_SIZE * 0.75)`
  - Z-слой: `1.0`
  - компонент: `NeutralArmyMarker(pos)`

### 5.2 Ресурс передачи данных боя (`src/main.rs`)

- [ ] Объявить ресурс `PendingBattle`:
  ```rust
  #[derive(Resource)]
  pub struct PendingBattle {
      pub attacker_hero_id: HeroId,
      pub defender_army: Army,
      pub defender_pos: Position,
  }
  ```
- [ ] Добавить `GameScreen::Battle` в enum.

### 5.3 Переход в режим боя (`adventure/input.rs`)

- [ ] После успешного `apply(MoveHero)` проверить события на `BattleStarted`.
- [ ] При обнаружении:
  - извлечь `Army` из `MapObject::NeutralArmy` в `game_state.map.get(defender_pos)`
  - записать `commands.insert_resource(PendingBattle { ... })`
  - перейти в `next_state.set(GameScreen::Battle)`
- [ ] Блокировать `is_passable` уже работает — герой не может просто пройти сквозь отряд.

> **Примечание:** `NeutralArmy` не удаляется с карты при входе — удаление происходит только после победы.

### 5.4 Модуль боя: типы (`src/battle/state.rs`)

- [ ] Объявить `StackId(u32)`.
- [ ] Объявить `Side { Attacker, Defender }`.
- [ ] Объявить `BattleStack`:
  ```rust
  pub struct BattleStack {
      pub id: StackId,
      pub unit_type: UnitType,
      pub count: u32,
      pub hp_remaining: u32,  // HP текущего переднего юнита в стеке
      pub side: Side,
  }
  ```
- [ ] Объявить `BattleEvent`:
  ```rust
  pub enum BattleEvent {
      Attacked { attacker_id: StackId, target_id: StackId, damage: u32, killed: u32 },
      StackDied { id: StackId },
      TurnPassed { next_id: StackId },
      BattleOver { winner: Side },
  }
  ```
- [ ] Объявить `BattleState` как Bevy `Resource`:
  ```rust
  #[derive(Resource)]
  pub struct BattleState {
      pub stacks: Vec<BattleStack>,
      pub turn_order: VecDeque<StackId>,
      pub current_stack_id: StackId,
      pub log: Vec<BattleEvent>,
  }
  ```

### 5.5 Механика боя (`src/battle/state.rs`)

Все методы — чистая логика, без Bevy.

- [ ] `BattleState::from_pending(pending: &PendingBattle, hero: &Hero) -> BattleState`:
  - создать `BattleStack` для каждого стека атакующей армии (`Side::Attacker`)
  - создать `BattleStack` для каждого стека защищающейся армии (`Side::Defender`)
  - построить `turn_order`: чередовать атакующих и защитников по порядку
  - установить `current_stack_id` на первый в очереди

- [ ] `BattleState::current_stack(&self) -> &BattleStack`:
  - вернуть стек по `current_stack_id`

- [ ] `BattleState::attack(&mut self, target_id: StackId) -> Vec<BattleEvent>`:
  - найти атакующий стек по `current_stack_id`, найти цель по `target_id`
  - проверить что цель принадлежит противоположной стороне
  - вычислить урон: `damage = attacker.count * attacker.unit_type.damage_per_unit`
  - вычислить убитых: `killed = damage / target.unit_type.hp`
  - вычислить остаток HP у переднего юнита: `hp_remaining = target.hp_remaining - (damage % target.unit_type.hp)`
    - если `hp_remaining == 0` — добить ещё одного, восстановить `hp_remaining = target.unit_type.hp`
  - обновить `target.count -= killed`, `target.hp_remaining`
  - если `target.count == 0` — убрать из `turn_order`, добавить `StackDied` в события
  - добавить событие `Attacked { ... }`
  - вызвать `next_turn()`
  - вернуть все события

- [ ] `BattleState::next_turn(&mut self)`:
  - снять `current_stack_id` с головы `turn_order`, положить в хвост (если стек жив)
  - обновить `current_stack_id` на новую голову

- [ ] `BattleState::is_over(&self) -> Option<Side>`:
  - если все стеки `Side::Attacker` мертвы → `Some(Side::Defender)`
  - если все стеки `Side::Defender` мертвы → `Some(Side::Attacker)`
  - иначе `None`

### 5.6 Инициализация боя (`src/battle/mod.rs`)

- [ ] Система `setup_battle` (в `OnEnter(GameScreen::Battle)`):
  - читать `PendingBattle`
  - найти героя по `attacker_hero_id` в `GameStateResource`
  - создать `BattleState::from_pending(...)`
  - вставить как ресурс: `commands.insert_resource(battle_state)`

### 5.7 Отрисовка боя (`src/battle/render.rs`)

Экран появляется вместо карты — карта не деспавнится, но скрыта состоянием.

**Компоненты:**
- [ ] `BattleStackWidget(StackId)` — маркер UI-элемента стека
- [ ] `BattleScreenRoot` — маркер корневого узла

**Система `spawn_battle_ui`** (в `OnEnter(GameScreen::Battle)`):

```
┌──────────────────────────────────────────────────┐
│                      БОЙ                         │
├──────────────────────┬───────────────────────────┤
│   АТАКУЮЩИЕ          │   ЗАЩИТНИКИ               │
│                      │                           │
│  [Мечник ×3]         │  [Гоблин ×5]              │
│   HP: 10  Dmg: 3     │   HP: 5   Dmg: 2          │
│                      │                           │
│  ► Крестьянин ×10    │                           │
│   HP: 5   Dmg: 1     │                           │
│   [Атаковать]        │                           │
├──────────────────────┴───────────────────────────┤
│  Ход: Крестьянин (Атакующий)                     │
└──────────────────────────────────────────────────┘
```

- [ ] Тёмный фон `rgba(0.05, 0.05, 0.1, 0.98)`, заголовок «БОЙ».
- [ ] Левая колонка: стеки атакующего. Правая: стеки защитника.
- [ ] Для каждого стека: название, `×count`, `HP: X`, `Dmg: Y`.
- [ ] Активный стек выделен рамкой или иным цветом (`srgb(1.0, 0.85, 0.1)` — жёлтый).
- [ ] Кнопка `Атаковать` под каждым стеком защитника (кликабельна только когда ход атакующего).
- [ ] Строка внизу: «Ход: [Название] ([Сторона])».

**Система `despawn_battle_ui`** (в `OnExit(GameScreen::Battle)`):
- [ ] `commands.entity(root).despawn_recursive()`

**Система `update_battle_ui`** (в `Update`, `in_state(Battle)`):
- [ ] При изменении `BattleState` обновить счётчики `count` и `hp_remaining` в виджетах.
- [ ] Убрать виджет мёртвого стека.
- [ ] Обновить строку «Ход».

### 5.8 Обработка ввода в бою (`src/battle/input.rs`)

- [ ] Компонент `AttackButton(StackId)` — хранит id целевого стека.
- [ ] Система `handle_battle_input` (в `Update`, `in_state(Battle)`):
  - Если ход текущего стека принадлежит `Side::Attacker`:
    - при клике на `AttackButton(target_id)` вызвать `battle_state.attack(target_id)`
    - обработать возвращённые `BattleEvent`
    - если `is_over()` → записать результат в ресурс `BattleResult`, перейти в нужное состояние
  - Иначе (ход защитника) — запустить AI-шаг (см. 5.9)

### 5.9 AI защитника (`src/battle/input.rs`)

- [ ] Функция `run_defender_turn(battle_state: &mut BattleState)`:
  - найти первый живой стек `Side::Attacker`
  - вызвать `battle_state.attack(target_id)`
- [ ] Вызывать сразу после `next_turn()`, если новый текущий стек — `Side::Defender`.
- [ ] Цикл: AI ходит немедленно, без анимации — до тех пор, пока ход не перейдёт обратно к атакующему или бой не закончится.

### 5.10 Завершение боя (`src/battle/mod.rs`)

- [ ] Объявить ресурс `BattleResult { winner: Side }`.
- [ ] Система `finish_battle` (запускается когда `is_over()` вернул `Some`):
  - **Победа атакующего (`Side::Attacker`):**
    - удалить `MapObject::NeutralArmy` с тайла `defender_pos`: `tile.object = None`
    - обновить армию героя: заменить `hero.army` на живые стеки из `battle_state` со стороны `Attacker`, конвертировав `BattleStack` обратно в `UnitStack`
    - despawn спрайта `NeutralArmyMarker(defender_pos)` на карте
    - перейти в `next_state.set(GameScreen::Adventure)`
    - установить `ShowBanner::Victory`
  - **Поражение атакующего (`Side::Defender`):**
    - перейти в `next_state.set(GameScreen::Adventure)`
    - установить `ShowBanner::Defeat`
  - Убрать ресурс `PendingBattle`, убрать ресурс `BattleState`

### 5.11 Баннер результата (`adventure/render.rs` или `src/battle/render.rs`)

- [ ] Объявить ресурс `ShowBanner(Option<BannerKind>)` с вариантами `Victory`, `Defeat`.
- [ ] Система `show_result_banner` (в `OnEnter(GameScreen::Adventure)`):
  - если `ShowBanner` содержит значение — заспавнить временный текстовый оверлей
  - текст «Победа!» (зелёный) или «Поражение» (красный), по центру экрана
  - убрать баннер через 2 секунды (таймер через `Timer` компонент) или по клику
- [ ] После скрытия сбросить `ShowBanner` в `None`.

### 5.12 Регистрация систем (`src/battle/mod.rs`)

```rust
impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameScreen::Battle), (setup_battle, render::spawn_battle_ui))
            .add_systems(OnExit(GameScreen::Battle),  render::despawn_battle_ui)
            .add_systems(Update, (
                render::update_battle_ui,
                input::handle_battle_input,
            ).run_if(in_state(GameScreen::Battle)));
    }
}
```

---

## Новые тесты (`src/battle/state.rs`)

- [ ] `battle_attack_kills_units` — атака наносит урон, убивает правильное количество юнитов
- [ ] `battle_stack_removed_when_dead` — мёртвый стек удаляется из `turn_order`
- [ ] `battle_over_when_all_defenders_dead` — `is_over()` возвращает `Some(Attacker)` когда все защитники мертвы
- [ ] `battle_ai_attacks_attacker` — при ходе `Defender` AI атакует первый живой стек `Attacker`
- [ ] `battle_turn_order_alternates` — ход чередуется между атакующими и защитниками

---

## Структура файлов после этапа

```
src/
├── main.rs                  — GameScreen::Battle, BattlePlugin, PendingBattle, BattleResult
├── core/
│   └── commands.rs          — без изменений (BattleStarted уже есть)
├── adventure/
│   ├── mod.rs               — NeutralArmyMarker
│   ├── render.rs            — маркеры NeutralArmy, show_result_banner
│   └── input.rs             — обработка BattleStarted → PendingBattle → GameScreen::Battle
└── battle/
    ├── mod.rs               — BattlePlugin, setup_battle, finish_battle
    ├── state.rs             — BattleState, BattleStack, Side, StackId, BattleEvent
    ├── render.rs            — spawn/despawn/update battle UI
    └── input.rs             — handle_battle_input, run_defender_turn
```

---

## Что остаётся в ROADMAP без изменений

Этапы 6–7 не меняются. После завершения обновить в `ROADMAP.md`:
- задачи 5.1–5.7 пометить `[x]`
- заголовок: `## Этап 5 — Простой бой ✅`

---

## Критерий готовности

1. На карте видны 2 красных маркера нейтральных отрядов.
2. Герой заходит на клетку с отрядом — открывается экран боя.
3. Игрок видит стеки обеих сторон, выделен активный.
4. Клик «Атаковать» → юниты умирают, счётчики обновляются.
5. После хода игрока AI автоматически отвечает.
6. После победы: экран боя закрывается, баннер «Победа!», красный маркер исчезает с карты, армия героя отражает потери.
7. При поражении: баннер «Поражение», игра не крашится.
8. `cargo test` — все тесты проходят, включая 5 новых в `battle/state.rs`.
9. `cargo clippy -- -D warnings` — нет предупреждений.
