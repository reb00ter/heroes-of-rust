# Stage 11 Plan — Фракции

**Цель:** разделить контент по фракциям. Каждая фракция определяет своего героя, набор юнитов
и стартовую армию. Выбор героя в меню = выбор фракции. Стартовый город автоматически
заполняется юнитами фракции выбранного героя.

**Зависит от:** Этап 10 (выбор героя).

**Статус:** ⬜ Не начато

---

## Две фракции

### Японская деревня (`japanese_village`)

**Герой Наруто:** атака=2, защита=1, обзор=5

| Юнит | id | Тир | Тип | HP | Урон | Стоимость | Рост/день |
|------|----|-----|-----|----|------|-----------|-----------|
| Клон Наруто | `naruto_clone` | 1 | ближний | 6 | 2 | 25 | 7 |
| Ученик ниндзя | `ninja_student` | 2 | дальний* | 12 | 4 | 60 | 3 |
| Ниндзя | `ninja` | 3 | ближний | 22 | 7 | 110 | 1 |
| Какаши | `kakashi` | 4 | ближний | 38 | 12 | 200 | 1 |

Стартовая армия: 5× Клон Наруто

---

### Самодельный штаб (`diy_hq`)

**Герой Егор:** атака=3, защита=0, обзор=5

| Юнит | id | Тир | Тип | HP | Урон | Стоимость | Рост/день |
|------|----|-----|-----|----|------|-----------|-----------|
| Мальчик с кулаками | `fist_boy` | 1 | ближний | 5 | 1 | 20 | 8 |
| Мальчик с камнем | `rock_boy` | 2 | дальний* | 8 | 3 | 45 | 4 |
| Мальчик с обувной ложкой | `shoehorn_boy` | 3 | ближний | 18 | 6 | 90 | 2 |
| Мальчик с закидушкой | `zakidushka_boy` | 4 | ближний | 28 | 10 | 170 | 1 |

Стартовая армия: 5× Мальчик с кулаками

*`ranged: true` зарезервирован в данных; механика дальнего боя — Stage 13.*

---

### Нейтралы (без фракции)

Остаются в `assets/data/units.ron`: гоблин, орк, тролль.
Крестьянин и Мечник **удаляются** из `units.ron` — они были фракционными юнитами-заглушками.

---

## Изменения формата карты

### `HeroDef` — убрать `name` и `army`

```ron
// было:
hero: (pos: (1,1), name: "Aldric", movement_points: 10,
       army: [(id: "peasant", count: 5)], starting_gold: 500)

// стало:
hero: (pos: (1,1), movement_points: 10, starting_gold: 500)
```

Имя и стартовая армия приходят из фракции.

### `TownDef` — убрать `recruits`

```ron
// было:
towns: [(id: 0, pos: (4,2), daily_income: 250,
         recruits: [(id: "peasant", count: 10), (id: "swordsman", count: 5)])]

// стало:
towns: [(id: 0, pos: (4,2), daily_income: 250)]
```

Рекруты города заполняются из фракции при старте игры.

---

## Задачи

### 10.1 Файл фракций (`assets/data/factions.ron`)

- [ ] Создать `assets/data/factions.ron`:
  ```ron
  [
      (
          id: "japanese_village",
          name: "Японская деревня",
          heroes: [
              (
                  id: "naruto", name: "Наруто",
                  attack: 2, defense: 1, sight_range: 5,
                  portrait: "heroes/naruto",
              ),
          ],
          units: [
              (id: "naruto_clone",  name: "Клон Наруто",            tier: 1, damage: 2, hp: 6,  cost: 25,  daily_growth: 7, ranged: false),
              (id: "ninja_student", name: "Ученик ниндзя",          tier: 2, damage: 4, hp: 12, cost: 60,  daily_growth: 3, ranged: true),
              (id: "ninja",         name: "Ниндзя",                 tier: 3, damage: 7, hp: 22, cost: 110, daily_growth: 1, ranged: false),
              (id: "kakashi",       name: "Какаши",                 tier: 4, damage: 12, hp: 38, cost: 200, daily_growth: 1, ranged: false),
          ],
          starting_army: [(id: "naruto_clone", count: 5)],
      ),
      (
          id: "diy_hq",
          name: "Самодельный штаб",
          heroes: [
              (
                  id: "egor", name: "Егор",
                  attack: 3, defense: 0, sight_range: 5,
                  portrait: "heroes/Yegor",
              ),
          ],
          units: [
              (id: "fist_boy",       name: "Мальчик с кулаками",        tier: 1, damage: 1,  hp: 5,  cost: 20,  daily_growth: 8, ranged: false),
              (id: "rock_boy",       name: "Мальчик с камнем",           tier: 2, damage: 3,  hp: 8,  cost: 45,  daily_growth: 4, ranged: true),
              (id: "shoehorn_boy",   name: "Мальчик с обувной ложкой",   tier: 3, damage: 6,  hp: 18, cost: 90,  daily_growth: 2, ranged: false),
              (id: "zakidushka_boy", name: "Мальчик с закидушкой",       tier: 4, damage: 10, hp: 28, cost: 170, daily_growth: 1, ranged: false),
          ],
          starting_army: [(id: "fist_boy", count: 5)],
      ),
  ]
  ```

---

### 10.2 Типы данных (`src/data/mod.rs`)

- [ ] Объявить:
  ```rust
  #[derive(serde::Deserialize, Clone, Debug)]
  pub struct FactionHeroDef {
      pub id:          String,
      pub name:        String,
      pub attack:      u32,
      pub defense:     u32,
      pub sight_range: u32,
      pub portrait:    String,
  }

  #[derive(serde::Deserialize, Clone, Debug)]
  pub struct FactionUnitDef {
      pub id:           String,
      pub name:         String,
      pub tier:         u32,
      pub damage:       u32,
      pub hp:           u32,
      pub cost:         u32,
      pub daily_growth: u32,
      #[serde(default)]
      pub ranged:       bool,
  }

  #[derive(serde::Deserialize, Clone, Debug)]
  pub struct FactionDef {
      pub id:            String,
      pub name:          String,
      pub heroes:        Vec<FactionHeroDef>,
      pub units:         Vec<FactionUnitDef>,
      pub starting_army: Vec<StackRef>,
  }
  ```
- [ ] Реализовать `pub fn load_factions(path: &str) -> Vec<FactionDef>`.
- [ ] Обновить `HeroDef` — удалить `name` и `army`.
- [ ] Обновить `TownDef` — удалить `recruits` и `daily_growth` (приходят из фракции).
- [ ] Обновить `map_def_to_game_state` — создавать `Town` без рекрутов.
- [ ] Обновить `load_map` — убрать зависимость от `load_units` для фракционных юнитов
  (только нейтральные армии остаются в `units.ron`).

---

### 10.3 Модель героя (`src/core/hero.rs`)

- [ ] Добавить поля в `Hero`:
  ```rust
  pub attack:  u32,
  pub defense: u32,
  // sight_range уже есть
  ```
- [ ] В `map_def_to_game_state`: `attack: 0, defense: 0`, имя = `""` — заполнится из фракции.

---

### 10.4 Обновление данных карты и юнитов

- [ ] `assets/maps/default.ron` — убрать `name` и `army` из `hero`, убрать `recruits` из `towns`.
- [ ] `assets/data/units.ron` — оставить только нейтралов: гоблин, орк, тролль.
  Удалить крестьянина и мечника.

---

### 10.5 Расширение `GameStartConfig` (`src/main.rs`)

- [ ] Добавить `faction_id: String`:
  ```rust
  #[derive(Resource)]
  pub struct GameStartConfig {
      pub map_path:   String,
      pub hero_name:  String,
      pub faction_id: String,
  }
  ```

---

### 10.6 Применение фракции (`adventure/mod.rs`)

В `load_map_from_config` после `load_map(...)`:

- [ ] Загрузить фракции: `load_factions("assets/data/factions.ron")`.
- [ ] Найти выбранную фракцию по `config.faction_id`.
- [ ] Применить к герою:
  ```rust
  hero.name.clone_from(&config.hero_name);
  hero.attack      = faction.hero.attack;
  hero.defense     = faction.hero.defense;
  hero.sight_range = faction.hero.sight_range;
  // стартовая армия
  for stack_ref in &faction.starting_army { ... }
  ```
- [ ] Заполнить рекрутов первого города юнитами фракции (тир 1→2→3→4).
- [ ] Вызвать `update_visibility` с новым `sight_range` (перенесено из `load_map`).

---

### 10.7 Портреты (`assets/sprites/heroes/`, `build.rs`)

- [ ] `assets/sprites/heroes/src/naruto.svg` — портрет Наруто.
- [ ] Добавить рендеринг `naruto.svg → naruto.png` в `build.rs`.
- [ ] `assets/sprites/heroes/Yegor.png` — уже добавлен вручную, SVG pipeline не нужен.

---

### 10.8 Экран выбора фракции/героя (`src/menu/mod.rs`)

**Компонент:**
```rust
#[derive(Component)]
struct FactionCard { faction_id: String }
```

**`MenuState`:**
```rust
struct MenuState {
    hero_name:        String,
    selected_map:     Option<String>,
    selected_faction: Option<String>,
    maps:             Vec<MapInfo>,
    factions:         Vec<FactionDef>,
}
```

**`setup_menu`:**
- [ ] Загрузить `load_factions("assets/data/factions.ron")`.
- [ ] Горизонтальный ряд карточек фракций:
  ```
  ┌─────────────────┐  ┌─────────────────┐
  │    [портрет]    │  │    [портрет]    │
  │ Японская деревня│  │Самодельный штаб │
  │ Герой: Наруто   │  │ Герой: Егор     │
  │ ⚔2  🛡1  👁5    │  │ ⚔3  🛡0  👁5    │
  │ Клон·Ниндзя·... │  │ Кулак·Камень·..│
  └─────────────────┘  └─────────────────┘
  ```
- [ ] Клик → `selected_faction = Some(id)`, имя предзаполняется именем героя фракции.

**`update_start_button`:**
- [ ] Активна если `!hero_name.is_empty() && selected_map.is_some() && selected_faction.is_some()`.

---

### 10.9 Бонусы героя в бою (`src/battle/state.rs`, `src/battle/mod.rs`)

- [ ] Добавить в `BattleState`:
  ```rust
  pub attacker_attack_bonus:  u32,
  pub attacker_defense_bonus: u32,
  ```
- [ ] `attack(target_id)`:
  - атакующий стек (сторона Attacker): `damage = count * base_damage + attacker_attack_bonus`
  - цель (сторона Attacker): `damage = max(1, raw.saturating_sub(attacker_defense_bonus))`
- [ ] Передавать `hero.attack` / `hero.defense` при создании `BattleState`.

---

### 10.10 Портрет и характеристики в UI приключения

- [ ] Компоненты `HeroPortraitUI`, `HeroStatsText`.
- [ ] `spawn_adventure_ui`: спавнить портрет + `"⚔ X  🛡 Y"`.
- [ ] Деспавнить и пересоздавать при рестарте.

---

### 10.11 Тесты

**`src/data/mod.rs`:**
- [ ] `load_factions_parses` — 2 фракции, ids `"japanese_village"` и `"diy_hq"`.
- [ ] `japanese_village_hero_stats` — Наруто: `attack=2`, `sight_range=5`.
- [ ] `diy_hq_starting_army` — стартовая армия содержит `"fist_boy"`, count=5.
- [ ] `faction_unit_ids_match_starting_army` — все id в `starting_army` есть в `units`.
- [ ] `neutrals_still_load` — `load_units("assets/data/units.ron")` даёт 3 записи (гоблин, орк, тролль).

**`src/battle/state.rs`:**
- [ ] `attack_bonus_increases_damage` — `bonus=3, count=5, base=1` → `damage=8`.
- [ ] `defense_bonus_reduces_damage` — `defense=2, raw=5` → `damage=3`.
- [ ] `defense_never_below_one` — `defense ≥ raw` → `damage=1`.

---

## Новые файлы

```
assets/data/factions.ron
assets/sprites/heroes/src/naruto.svg
assets/sprites/heroes/naruto.png        (генерируется build.rs)
assets/sprites/heroes/Yegor.png         (уже добавлен вручную)
```

## Изменяемые файлы

```
src/core/hero.rs         — +attack, +defense
src/data/mod.rs          — +FactionDef/FactionHeroDef/FactionUnitDef, +load_factions();
                           обновить HeroDef (–name, –army), TownDef (–recruits),
                           map_def_to_game_state; обновить тесты
src/main.rs              — +faction_id в GameStartConfig
src/adventure/mod.rs     — применение фракции в load_map_from_config; update_visibility сюда
src/adventure/render.rs  — +HeroPortraitUI
src/battle/state.rs      — +attack/defense_bonus, изменить attack()
src/battle/mod.rs        — передавать hero bonuses в BattleState
src/menu/mod.rs          — FactionCard вместо HeroCard; обновить MenuState и StartButton
build.rs                 — +рендеринг naruto.svg
assets/data/units.ron    — оставить только гоблин/орк/тролль
assets/maps/default.ron  — убрать name/army из hero, убрать recruits из towns
```

---

## Порядок реализации

1. `assets/data/factions.ron` — файл фракций
2. `assets/data/units.ron` — убрать фракционных юнитов
3. `assets/maps/default.ron` — обновить формат
4. `src/data/mod.rs` — новые типы, `load_factions`, обновить `HeroDef`/`TownDef`/`map_def_to_game_state`; обновить тесты
5. `src/core/hero.rs` — `+attack`, `+defense`
6. `src/main.rs` — `+faction_id` в `GameStartConfig`
7. SVG-портрет Наруто + `build.rs`
8. `src/menu/mod.rs` — `FactionCard`, панель выбора фракции
9. `src/adventure/mod.rs` — применение фракции + `update_visibility`
10. `src/adventure/render.rs` — `HeroPortraitUI`
11. `src/battle/state.rs` + `src/battle/mod.rs` — бонусы атаки/защиты
12. `cargo test` — все тесты зелёные
13. `cargo clippy -- -D warnings` + `cargo fmt --check`
14. Коммит

---

## Критерий готовности

1. В меню — 2 карточки фракций с портретами героев и списком юнитов.
2. Выбор фракции предзаполняет имя героя (редактируемое).
3. Стартовый город содержит юнитов выбранной фракции, не захардкоженных в карте.
4. Герой стартует с армией своей фракции (не из карты).
5. Наруто и Егор дают разный урон и видимость.
6. `cargo test` — все тесты зелёные, включая 8 новых.
7. `cargo clippy -- -D warnings` и `cargo fmt --check` — чисто.
