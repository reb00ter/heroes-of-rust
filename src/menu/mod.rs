mod name_gen;

use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use rand::thread_rng;

use crate::GameScreen;
use crate::data::{HeroRosterDef, MapInfo, discover_maps, load_heroes, load_units};

// ---------------------------------------------------------------------------
// Компоненты
// ---------------------------------------------------------------------------

#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
struct HeroNameDisplay;

#[derive(Component)]
struct MapListItem {
    index: usize,
}

#[derive(Component)]
struct StartButton;

#[derive(Component)]
struct StartButtonText;

#[derive(Component)]
struct RandomNameButton;

#[derive(Component)]
struct HeroCard {
    hero_id: String,
}

#[derive(Component)]
struct HeroCardBg {
    hero_id: String,
}

// ---------------------------------------------------------------------------
// Ресурс состояния меню
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
struct MenuState {
    hero_name: String,
    selected_map_index: Option<usize>,
    selected_hero_id: Option<String>,
    maps: Vec<MapInfo>,
    heroes: Vec<HeroRosterDef>,
}

// ---------------------------------------------------------------------------
// Плагин
// ---------------------------------------------------------------------------

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>()
            .add_systems(OnEnter(GameScreen::MainMenu), setup_menu)
            .add_systems(OnExit(GameScreen::MainMenu), despawn_menu)
            .add_systems(
                Update,
                (
                    handle_hero_name_input,
                    handle_map_selection,
                    handle_hero_selection,
                    update_start_button,
                    handle_start_button,
                    handle_random_name_button,
                )
                    .chain()
                    .run_if(in_state(GameScreen::MainMenu)),
            );
    }
}

// ---------------------------------------------------------------------------
// Системы
// ---------------------------------------------------------------------------

#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut menu_state: ResMut<MenuState>,
) {
    let units = load_units("assets/data/units.ron");
    menu_state.maps = discover_maps("assets/maps/", &units);
    menu_state.heroes = load_heroes("assets/data/heroes.ron");
    menu_state.hero_name = String::new();
    menu_state.selected_hero_id = None;
    menu_state.selected_map_index = if menu_state.maps.len() == 1 {
        Some(0)
    } else {
        None
    };

    info!(
        "[MENU] Setup. Found {} valid map(s), {} heroes.",
        menu_state.maps.len(),
        menu_state.heroes.len()
    );

    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");

    let maps_snapshot: Vec<(String, String, u32, u32, usize, usize)> = menu_state
        .maps
        .iter()
        .map(|m| {
            (
                m.name.clone(),
                m.description.clone(),
                m.width,
                m.height,
                m.neutral_count,
                m.town_count,
            )
        })
        .collect();
    let initial_selected = menu_state.selected_map_index;
    let heroes_snapshot: Vec<HeroRosterDef> = menu_state.heroes.clone();

    // Корневой контейнер — полноэкранный фон
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.06, 0.06, 0.10)),
            GlobalZIndex(100),
            MenuRoot,
        ))
        .id();

    // Заголовок
    let title = commands
        .spawn((
            Text::new("Heroes of Rust"),
            TextFont {
                font: font.clone(),
                font_size: 56.0,
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.80, 0.20)),
        ))
        .id();

    // === Панель выбора героя ===
    let hero_label = commands
        .spawn((
            Text::new("Выберите героя:"),
            TextFont {
                font: font.clone(),
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.75, 0.75, 0.80)),
        ))
        .id();

    let hero_row = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            ..default()
        })
        .id();

    let mut hero_card_entities: Vec<Entity> = Vec::new();
    for hero_def in &heroes_snapshot {
        let card = commands
            .spawn((
                Button,
                Node {
                    width: Val::Px(148.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexStart,
                    padding: UiRect::all(Val::Px(8.0)),
                    row_gap: Val::Px(4.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.35, 0.35, 0.50)),
                BackgroundColor(Color::srgb(0.10, 0.10, 0.18)),
                HeroCard {
                    hero_id: hero_def.id.clone(),
                },
                HeroCardBg {
                    hero_id: hero_def.id.clone(),
                },
            ))
            .id();

        // Портрет
        let portrait_path = format!("sprites/{}.png", hero_def.portrait);
        let portrait_tex: Handle<Image> = asset_server.load(portrait_path);
        let portrait = commands
            .spawn((
                ImageNode::new(portrait_tex),
                Node {
                    width: Val::Px(128.0),
                    height: Val::Px(100.0),
                    ..default()
                },
            ))
            .id();

        // Имя героя
        let hero_name_text = commands
            .spawn((
                Text::new(hero_def.name.clone()),
                TextFont {
                    font: font.clone(),
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ))
            .id();

        // Характеристики — без эмодзи, только ASCII
        let stats_text = commands
            .spawn((
                Text::new(format!(
                    "Атк:{} Защ:{} Обз:{}",
                    hero_def.attack, hero_def.defense, hero_def.sight_range
                )),
                TextFont {
                    font: font.clone(),
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.60)),
            ))
            .id();

        commands
            .entity(card)
            .add_children(&[portrait, hero_name_text, stats_text]);
        hero_card_entities.push(card);
    }

    // === Поле имени ===
    let name_row = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        })
        .id();

    let name_box = commands
        .spawn((
            Node {
                width: Val::Px(300.0),
                height: Val::Px(44.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::axes(Val::Px(12.0), Val::Px(0.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.35, 0.35, 0.50)),
            BackgroundColor(Color::srgb(0.12, 0.12, 0.18)),
        ))
        .id();

    let name_display = commands
        .spawn((
            Text::new("Введите имя героя"),
            TextFont {
                font: font.clone(),
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.45, 0.45, 0.55)),
            HeroNameDisplay,
        ))
        .id();

    let random_btn = commands
        .spawn((
            Button,
            Node {
                width: Val::Px(44.0),
                height: Val::Px(44.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.35, 0.35, 0.50)),
            BackgroundColor(Color::srgb(0.20, 0.20, 0.30)),
            RandomNameButton,
        ))
        .id();

    let random_btn_text = commands
        .spawn((
            Text::new("Сл."),
            TextFont {
                font: font.clone(),
                font_size: 13.0,
                ..default()
            },
            TextColor(Color::srgb(0.85, 0.85, 0.95)),
        ))
        .id();

    // === Список карт ===
    let map_label = commands
        .spawn((
            Text::new("Выберите карту:"),
            TextFont {
                font: font.clone(),
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.75, 0.75, 0.80)),
        ))
        .id();

    let map_list = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(5.0),
            width: Val::Px(520.0),
            ..default()
        })
        .id();

    let mut map_btn_entities: Vec<Entity> = Vec::new();
    for (i, (name, description, width, height, neutral_count, town_count)) in
        maps_snapshot.iter().enumerate()
    {
        let is_selected = initial_selected == Some(i);
        let bg = if is_selected {
            Color::srgb(0.18, 0.32, 0.52)
        } else {
            Color::srgb(0.10, 0.10, 0.16)
        };

        let map_btn = commands
            .spawn((
                Button,
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(8.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.30, 0.30, 0.45)),
                BackgroundColor(bg),
                MapListItem { index: i },
            ))
            .id();

        let line1 = commands
            .spawn((
                Text::new(format!(
                    "{name}  |  {width}×{height}  |  нейтралов: {neutral_count}  |  замков: {town_count}"
                )),
                TextFont {
                    font: font.clone(),
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ))
            .id();

        commands.entity(map_btn).add_child(line1);

        if !description.is_empty() {
            let line2 = commands
                .spawn((
                    Text::new(description.clone()),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.65, 0.65, 0.70)),
                ))
                .id();
            commands.entity(map_btn).add_child(line2);
        }

        map_btn_entities.push(map_btn);
    }

    if maps_snapshot.is_empty() {
        let no_maps = commands
            .spawn((
                Text::new("Карты не найдены"),
                TextFont {
                    font: font.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.80, 0.30, 0.30)),
            ))
            .id();
        commands.entity(map_list).add_child(no_maps);
    }

    // === Кнопка «Начать игру» ===
    let start_btn = commands
        .spawn((
            Button,
            Node {
                width: Val::Px(220.0),
                height: Val::Px(52.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.30, 0.30, 0.40)),
            BackgroundColor(Color::srgb(0.22, 0.22, 0.28)),
            StartButton,
        ))
        .id();

    let start_btn_text = commands
        .spawn((
            Text::new("Начать игру"),
            TextFont {
                font,
                font_size: 22.0,
                ..default()
            },
            TextColor(Color::srgb(0.45, 0.45, 0.50)),
            StartButtonText,
        ))
        .id();

    // Сборка иерархии
    commands.entity(name_box).add_child(name_display);
    commands.entity(random_btn).add_child(random_btn_text);
    commands
        .entity(name_row)
        .add_children(&[name_box, random_btn]);
    for e in &hero_card_entities {
        commands.entity(hero_row).add_child(*e);
    }
    for e in &map_btn_entities {
        commands.entity(map_list).add_child(*e);
    }
    commands.entity(start_btn).add_child(start_btn_text);
    commands.entity(root).add_children(&[
        title, hero_label, hero_row, name_row, map_label, map_list, start_btn,
    ]);
}

#[allow(clippy::needless_pass_by_value)]
fn despawn_menu(mut commands: Commands, root_q: Query<Entity, With<MenuRoot>>) {
    for entity in &root_q {
        commands.entity(entity).despawn();
    }
}

/// Считывает вводимые символы и обновляет имя героя в `MenuState`.
#[allow(clippy::needless_pass_by_value)]
fn handle_hero_name_input(
    mut key_events: MessageReader<KeyboardInput>,
    mut menu_state: ResMut<MenuState>,
    mut name_q: Query<(&mut Text, &mut TextColor), With<HeroNameDisplay>>,
) {
    let mut changed = false;
    for event in key_events.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        if event.key_code == KeyCode::Backspace {
            menu_state.hero_name.pop();
            changed = true;
            continue;
        }
        if let Some(text) = &event.text {
            let s = text.as_str();
            if s.chars().all(|c| !c.is_control()) && menu_state.hero_name.len() + s.len() <= 20 {
                menu_state.hero_name.push_str(s);
                changed = true;
            }
        }
    }

    if !changed {
        return;
    }

    let Ok((mut text, mut color)) = name_q.single_mut() else {
        return;
    };
    if menu_state.hero_name.is_empty() {
        **text = "Введите имя героя".to_string();
        color.0 = Color::srgb(0.45, 0.45, 0.55);
    } else {
        (**text).clone_from(&menu_state.hero_name);
        color.0 = Color::WHITE;
    }
}

/// Обрабатывает клик по карте в списке.
#[allow(clippy::needless_pass_by_value)]
fn handle_map_selection(
    mut menu_state: ResMut<MenuState>,
    interaction_q: Query<(&Interaction, &MapListItem), Changed<Interaction>>,
    mut map_items_q: Query<(&MapListItem, &mut BackgroundColor)>,
) {
    let mut newly_selected: Option<usize> = None;
    for (interaction, item) in &interaction_q {
        if *interaction == Interaction::Pressed {
            newly_selected = Some(item.index);
        }
    }

    let Some(new_idx) = newly_selected else {
        return;
    };

    menu_state.selected_map_index = Some(new_idx);
    info!("[MENU] Map selected: index {new_idx}");

    for (item, mut bg) in &mut map_items_q {
        bg.0 = if item.index == new_idx {
            Color::srgb(0.18, 0.32, 0.52)
        } else {
            Color::srgb(0.10, 0.10, 0.16)
        };
    }
}

/// Обрабатывает клик по карточке героя.
#[allow(clippy::needless_pass_by_value, clippy::collapsible_if)]
fn handle_hero_selection(
    mut menu_state: ResMut<MenuState>,
    interaction_q: Query<(&Interaction, &HeroCard), Changed<Interaction>>,
    mut card_bg_q: Query<(&HeroCardBg, &mut BackgroundColor)>,
    mut name_q: Query<(&mut Text, &mut TextColor), With<HeroNameDisplay>>,
) {
    let mut newly_selected: Option<String> = None;
    for (interaction, card) in &interaction_q {
        if *interaction == Interaction::Pressed {
            newly_selected = Some(card.hero_id.clone());
        }
    }

    let Some(new_id) = newly_selected else {
        return;
    };

    // Предзаполнить имя, если поле пустое или совпадает с именем другого героя из ростера
    let new_hero_name = menu_state
        .heroes
        .iter()
        .find(|h| h.id == new_id)
        .map(|h| h.name.clone());

    let current_is_roster_name = menu_state
        .heroes
        .iter()
        .any(|h| h.name == menu_state.hero_name);

    if menu_state.hero_name.is_empty() || current_is_roster_name {
        if let Some(name) = new_hero_name {
            menu_state.hero_name.clone_from(&name);
            if let Ok((mut text, mut color)) = name_q.single_mut() {
                (**text).clone_from(&menu_state.hero_name);
                color.0 = Color::WHITE;
            }
        }
    }

    menu_state.selected_hero_id = Some(new_id.clone());
    info!("[MENU] Hero selected: {new_id}");

    for (card_bg, mut bg) in &mut card_bg_q {
        bg.0 = if card_bg.hero_id == new_id {
            Color::srgb(0.22, 0.40, 0.22)
        } else {
            Color::srgb(0.10, 0.10, 0.18)
        };
    }
}

/// Обновляет вид кнопки «Начать игру» в зависимости от состояния ввода.
#[allow(clippy::needless_pass_by_value)]
fn update_start_button(
    menu_state: Res<MenuState>,
    mut btn_q: Query<&mut BackgroundColor, With<StartButton>>,
    mut txt_q: Query<&mut TextColor, With<StartButtonText>>,
) {
    if !menu_state.is_changed() {
        return;
    }

    let active = !menu_state.hero_name.is_empty()
        && menu_state.selected_map_index.is_some()
        && menu_state.selected_hero_id.is_some();

    for mut bg in &mut btn_q {
        bg.0 = if active {
            Color::srgb(0.12, 0.50, 0.22)
        } else {
            Color::srgb(0.22, 0.22, 0.28)
        };
    }
    for mut tc in &mut txt_q {
        tc.0 = if active {
            Color::WHITE
        } else {
            Color::srgb(0.45, 0.45, 0.50)
        };
    }
}

/// Обрабатывает клик по кнопке «Начать игру».
#[allow(clippy::needless_pass_by_value)]
fn handle_start_button(
    mut commands: Commands,
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<StartButton>)>,
    menu_state: Res<MenuState>,
    mut next_state: ResMut<NextState<GameScreen>>,
) {
    for interaction in &interaction_q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(idx) = menu_state.selected_map_index else {
            continue;
        };
        if menu_state.hero_name.is_empty() {
            continue;
        }
        let Some(ref hero_id) = menu_state.selected_hero_id else {
            continue;
        };
        let Some(map_info) = menu_state.maps.get(idx) else {
            continue;
        };
        let Some(hero_def) = menu_state.heroes.iter().find(|h| &h.id == hero_id) else {
            continue;
        };

        info!(
            "[MENU] Starting game. Hero: '{}' ({}), Map: '{}'",
            menu_state.hero_name, hero_id, map_info.path
        );

        commands.insert_resource(crate::GameStartConfig {
            map_path: map_info.path.clone(),
            hero_name: menu_state.hero_name.clone(),
            hero_attack: hero_def.attack,
            hero_defense: hero_def.defense,
            hero_sight: hero_def.sight_range,
            hero_portrait: format!("sprites/{}.png", hero_def.portrait),
        });
        next_state.set(GameScreen::Adventure);
    }
}

/// Обрабатывает клик по кнопке случайного имени.
#[allow(clippy::needless_pass_by_value)]
fn handle_random_name_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<RandomNameButton>)>,
    mut menu_state: ResMut<MenuState>,
    mut name_q: Query<(&mut Text, &mut TextColor), With<HeroNameDisplay>>,
) {
    for interaction in &interaction_q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let name = name_gen::generate(&mut thread_rng());
        info!("[MENU] Random name generated: '{name}'");
        menu_state.hero_name.clone_from(&name);

        let Ok((mut text, mut color)) = name_q.single_mut() else {
            continue;
        };
        **text = name;
        color.0 = Color::WHITE;
    }
}
