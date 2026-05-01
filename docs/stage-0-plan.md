# Stage 0 Plan — Подготовка репозитория

> Этап выполнен до введения требования об обязательном плановом документе.
> Файл создан ретроспективно как историческая запись.

**Цель:** создать основу проекта, убедиться что `cargo run` открывает окно.

**Статус:** ✅ Завершён

---

## Что было в проекте до этапа

Пустая директория.

## Что добавлялось

### Инициализация

- `git init` — создан репозиторий
- `cargo new heroes_of_rust --bin` — создан Rust-проект
- `rust-toolchain.toml` — зафиксирован `channel = "stable"`
- `.gitignore` — исключены `target/`, `.env`, `*.pdb`
- `README.md` — описание проекта, команды разработки, ссылка на ROADMAP

### Зависимости (`Cargo.toml`)

- `bevy = { version = "0.18", features = ["dynamic_linking"] }`
- `[profile.dev] opt-level = 1`
- `[profile.dev.package."*"] opt-level = 3`

### Инструменты качества кода

- `rustfmt.toml` — `edition = "2024"`, `max_width = 100`
- `#![warn(clippy::all, clippy::pedantic)]` в `main.rs`

### Минимальное окно Bevy (`src/main.rs`)

- `App::new()` с `DefaultPlugins`
- Заголовок окна: `"Heroes of Rust"`
- Размер: 1280×720
- `ClearColor(Color::srgb(0.08, 0.08, 0.12))`

### Первый коммит

```
7f762b4 chore: init project
```

---

## Файлы созданные в этапе

```
.gitignore
Cargo.toml
Cargo.lock
README.md
rust-toolchain.toml
rustfmt.toml
src/main.rs
```

---

## Критерий готовности

`cargo run` открывает окно с тёмным фоном и заголовком «Heroes of Rust». ✅
