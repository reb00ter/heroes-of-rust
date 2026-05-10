# Stage 10 Plan — Выбор героя

**Цель:** перед началом игры дать выбор из нескольких заранее определённых героев.
У каждого героя — имя, портрет, уникальные характеристики: атака, защита и дальность обзора.
Выбранный герой влияет и на туман войны, и на урон в бою.

**Зависит от:** Этап 9 (туман войны — `Hero::sight_range` уже есть).

**Статус:** ⬜ Не начато

---

## Что уже есть

| Элемент | Файл | Состояние |
|---------|------|-----------|
| `Hero::sight_range: u32` | `src/core/hero.rs` | ✅ значение приходит из ростера, не из карты |
| `HeroDef::sight_range` с `#[serde(default)]` | `src/data/mod.rs` | ❌ убрать — не карточная характеристика |
| `load_map` вызывает `update_visibility` | `src/data/mod.rs` | ❌ перенести в `load_map_from_config` |
| `GameStartConfig { map_path, hero_name }` | `src/main.rs` | ✅ расширяем |
| `MenuState { hero_name, selected_map, maps }` | `src/menu/mod.rs` | ✅ добавляем `selected_hero` |
| SVG→PNG pipeline в `build.rs` | `build.rs` | ✅ добавляем портреты героев |
| `BattleState::attack()` — урон без бонусов | `src/battle/state.rs` | ✅ добавляем `hero_attack/defense` |

---

## Три героя ростера

| id | Имя | Атака | Защита | Обзор | Стиль |
|----|-----|-------|--------|-------|-------|
| `aldric` | Альдрик | 2 | 1 | 4 | Сбалансированный боец |
| `valeria` | Валерия | 1 | 0 | 6 | Разведчица — видит далеко, слабее в бою |
| `gorm` | Горм | 3 | 0 | 3 | Берсерк — максимальный урон, узкий обзор |

**Атака:** добавляется к совокупному урону армии за удар.
**Защита:** вычитается из совокупного входящего урона (минимум 1).

---

## Задачи

### 10.1 Исправление данных карты (`src/data/mod.rs`, `assets/maps/default.ron`)

`sight_range` — характеристика героя, не карты. Убрать из формата карты.

- [ ] Удалить поле `sight_range` и `default_sight_range()` из `HeroDef`:
  ```rust
  // было:
  #[serde(default = "default_sight_range")]
  pub sight_range: u32,
  // → убрать полностью
  ```
- [ ] Удалить вызов `update_visibility` из `load_map` (перенесётся в 10.5).
- [ ] Удалить тест `load_map_reveals_hero_start` из `src/data/mod.rs`
  (он тестировал поведение, которое переезжает в `load_map_from_config`).
- [ ] В `map_def_to_game_state` задавать `hero.sight_range = 0`
  (placeholder — реальное значение придёт из ростера).
- [ ] Удалить `sight_range` из `assets/maps/default.ron` если там есть.
- [ ] Добавить `sight_range` в `#[serde(deny_unknown_fields)]`-исключение (или убедиться что поле
  не было в файле карты).

---

### 10.2 Ростер героев (`assets/data/heroes.ron`, `src/data/mod.rs`)

- [ ] Создать `assets/data/heroes.ron`:
  ```ron
  [
      (id: "aldric",  name: "Альдрик", attack: 2, defense: 1, sight_range: 4, portrait: "heroes/aldric"),
      (id: "valeria", name: "Валерия", attack: 1, defense: 0, sight_range: 6, portrait: "heroes/valeria"),
      (id: "gorm",    name: "Горм",    attack: 3, defense: 0, sight_range: 3, portrait: "heroes/gorm"),
  ]
  ```
- [ ] Объявить `HeroRosterDef` в `src/data/mod.rs`:
  ```rust
  #[derive(serde::Deserialize, Clone, Debug)]
  pub struct HeroRosterDef {
      pub id:          String,
      pub name:        String,
      pub attack:      u32,
      pub defense:     u32,
      pub sight_range: u32,
      pub portrait:    String,   // путь без расширения, напр. "heroes/aldric"
  }
  ```
- [ ] Реализовать `pub fn load_heroes(path: &str) -> Vec<HeroRosterDef>`.

---

### 10.3 Модель героя (`src/core/hero.rs`)

- [ ] Добавить поля в `Hero`:
  ```rust
  pub attack:  u32,   // бонус к урону армии
  pub defense: u32,   // снижение входящего урона
  ```
- [ ] В `map_def_to_game_state` инициализировать `attack: 0, defense: 0`
  (перезаписываются в `load_map_from_config`).

---

### 10.4 Портреты героев (`assets/sprites/heroes/`, `build.rs`)

- [ ] Создать директорию `assets/sprites/heroes/src/`.
- [ ] Нарисовать 3 SVG-портрета (~64×80 px, простая геометрия):
  - `aldric.svg` — синий щит
  - `valeria.svg` — зелёная стрела
  - `gorm.svg` — красный кулак
- [ ] Добавить рендеринг в `build.rs`:
  ```rust
  for (svg, png) in [
      ("assets/sprites/heroes/src/aldric.svg",  "assets/sprites/heroes/aldric.png"),
      ("assets/sprites/heroes/src/valeria.svg", "assets/sprites/heroes/valeria.png"),
      ("assets/sprites/heroes/src/gorm.svg",    "assets/sprites/heroes/gorm.png"),
  ] {
      render_svg(svg, png, 2.0);
  }
  ```

---

### 10.5 Расширение `GameStartConfig` (`src/main.rs`)

- [ ] Добавить поля выбранного героя:
  ```rust
  #[derive(Resource)]
  pub struct GameStartConfig {
      pub map_path:      String,
      pub hero_name:     String,
      pub hero_attack:   u32,
      pub hero_defense:  u32,
      pub hero_sight:    u32,
      pub hero_portrait: String,   // путь к PNG, напр. "sprites/heroes/aldric.png"
  }
  ```

---

### 10.6 Применение характеристик героя (`adventure/mod.rs`)

В `load_map_from_config`, после вызова `load_map(...)`:

- [ ] Применить поля из `GameStartConfig`:
  ```rust
  if let Some(hero) = gs.heroes.first_mut() {
      hero.name.clone_from(&config.hero_name);
      hero.attack      = config.hero_attack;
      hero.defense     = config.hero_defense;
      hero.sight_range = config.hero_sight;
  }
  ```
- [ ] Вызвать `update_visibility(&mut gs.map, hero.position, hero.sight_range)`
  **здесь** (перенесено из `load_map`).
- [ ] Добавить тест на интеграцию видимости переместился в `adventure` (опционально,
  тестируется через существующий сценарий запуска).

---

### 10.7 Экран выбора героя (`src/menu/mod.rs`)

**Новый компонент:**
```rust
#[derive(Component)]
struct HeroCard { hero_id: String }
```

**`MenuState` расширяется:**
```rust
#[derive(Resource, Default)]
struct MenuState {
    hero_name:     String,
    selected_map:  Option<String>,
    selected_hero: Option<String>,   // hero id
    maps:          Vec<MapInfo>,
    heroes:        Vec<HeroRosterDef>,
}
```

**`setup_menu`:**
- [ ] Загрузить ростер через `data::load_heroes("assets/data/heroes.ron")`.
- [ ] Отрисовать горизонтальный ряд карточек:
  ```
  ┌──────────┐  ┌──────────┐  ┌──────────┐
  │ [портрет]│  │ [портрет]│  │ [портрет]│
  │ Альдрик  │  │ Валерия  │  │ Горм     │
  │ ⚔2 🛡1 👁4│  │ ⚔1 🛡0 👁6│  │ ⚔3 🛡0 👁3│
  └──────────┘  └──────────┘  └──────────┘
  ```
- [ ] Клик на карточку → `selected_hero = Some(id)`, поле имени предзаполняется именем героя
  (если поле пустое или содержит имя другого героя из ростера).

**`handle_hero_selection` (`Update`, `in_state(MainMenu)`):**
- [ ] Обновить `selected_hero`, предзаполнить имя, выделить карточку визуально.

**`update_start_button`:**
- [ ] Активна если `!hero_name.is_empty() && selected_map.is_some() && selected_hero.is_some()`.

**`handle_start_button`:**
- [ ] Найти `HeroRosterDef` по `selected_hero`, собрать `GameStartConfig` со всеми полями.

---

### 10.8 Бонусы героя в бою (`src/battle/state.rs`, `src/battle/mod.rs`)

- [ ] Добавить поля в `BattleState`:
  ```rust
  pub attacker_attack_bonus:  u32,
  pub attacker_defense_bonus: u32,
  ```
- [ ] Изменить `BattleState::attack(target_id)`:
  - Атакующий стек (сторона `Attacker`): `damage = count * unit_base_damage + attacker_attack_bonus`
  - Цель — сторона `Attacker`: `damage = max(1, raw_damage.saturating_sub(attacker_defense_bonus))`
- [ ] Передавать бонусы при создании `BattleState` из `hero.attack` / `hero.defense`.

---

### 10.9 Портрет героя в UI приключения

- [ ] Компонент `HeroPortraitUI` + `HeroStatsText`.
- [ ] Спавнить в `spawn_adventure_ui`: изображение портрета + строка `"⚔ X  🛡 Y"`.
- [ ] При рестарте (`load_map_from_config`) деспавнить и пересоздавать.

---

### 10.10 Тесты

**`src/data/mod.rs`:**
- [ ] `load_heroes_parses` — 3 записи; `aldric` имеет `attack=2`, `sight_range=4`.
- [ ] `all_hero_ids_unique` — нет дублирующихся `id`.

**`src/battle/state.rs`:**
- [ ] `attack_bonus_increases_damage` — `bonus=2, count=3, base=1` → урон=5.
- [ ] `defense_bonus_reduces_damage` — `defense=3, raw=4` → урон=1.
- [ ] `defense_never_below_one` — `defense > damage` → урон=1.

---

## Новые файлы

```
assets/data/heroes.ron
assets/sprites/heroes/src/aldric.svg
assets/sprites/heroes/src/valeria.svg
assets/sprites/heroes/src/gorm.svg
assets/sprites/heroes/aldric.png   (генерируется build.rs)
assets/sprites/heroes/valeria.png  (генерируется build.rs)
assets/sprites/heroes/gorm.png     (генерируется build.rs)
```

## Изменяемые файлы

```
src/core/hero.rs         — +attack: u32, +defense: u32
src/data/mod.rs          — убрать sight_range из HeroDef и update_visibility из load_map;
                           +HeroRosterDef, +load_heroes(); убрать тест load_map_reveals_hero_start
src/main.rs              — +hero_attack/defense/sight/portrait в GameStartConfig
src/adventure/mod.rs     — применить характеристики + вызов update_visibility после load_map
src/adventure/render.rs  — +HeroPortraitUI в spawn_adventure_ui
src/battle/state.rs      — +attacker_attack/defense_bonus, изменить attack()
src/battle/mod.rs        — передавать hero.attack/defense при создании BattleState
src/menu/mod.rs          — +HeroCard, панель выбора героя, MenuState, StartButton
build.rs                 — +рендеринг 3 портретов
assets/maps/default.ron  — убрать sight_range из секции hero (если было)
```

---

## Порядок реализации

1. `src/data/mod.rs` — убрать `sight_range` из `HeroDef`, убрать `update_visibility` из `load_map`
2. `assets/maps/default.ron` — убрать `sight_range` из `hero`
3. `assets/data/heroes.ron` + `HeroRosterDef` + `load_heroes` + тесты
4. `src/core/hero.rs` — `+attack`, `+defense`
5. `src/main.rs` — расширить `GameStartConfig`
6. SVG-портреты + `build.rs`
7. `src/menu/mod.rs` — панель выбора героя
8. `src/adventure/mod.rs` — применение характеристик + `update_visibility`
9. `src/adventure/render.rs` — `HeroPortraitUI`
10. `src/battle/state.rs` + `src/battle/mod.rs` — бонусы + тесты
11. `cargo test` — все тесты зелёные
12. `cargo clippy -- -D warnings` + `cargo fmt --check`
13. Коммит

---

## Критерий готовности

1. В меню отображаются 3 героя с портретами и характеристиками.
2. Выбор героя предзаполняет поле имени (редактируемое).
3. Кнопка «Начать игру» активна только при выборе карты, героя и непустом имени.
4. Валерия при старте видит значительно больше карты (sight=6), Горм — меньше (sight=3).
5. В приключении видны портрет и характеристики выбранного героя.
6. Урон в бою отличается у разных героев.
7. `cargo test` — все тесты зелёные, включая 5 новых.
8. `cargo clippy -- -D warnings` и `cargo fmt --check` — чисто.
