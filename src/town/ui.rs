use bevy::prelude::*;

use crate::GameScreen;
use crate::adventure::GameStateResource;
use crate::core::commands::GameCommand;
use crate::core::player::TownId;
use crate::core::state::GameState;

use super::{CurrentTownId, HireButton, LeaveButton};

/// Маркер корневого узла UI экрана города — используется для despawn при выходе.
#[derive(Component)]
pub struct TownScreenRoot;

// ---------------------------------------------------------------------------
// Вспомогательная функция спавна UI (используется и при первом входе, и при перестройке)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_lines)]
fn spawn_ui(commands: &mut Commands, gs: &GameState, town_id: TownId, font: Handle<Font>) {
    let Some(town) = gs.towns.iter().find(|t| t.id == town_id) else {
        return;
    };
    let gold = gs.players.first().map_or(0, |p| p.resources.gold);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(12.0),
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.95)),
            GlobalZIndex(100),
            TownScreenRoot,
        ))
        .with_children(|parent| {
            // Заголовок + кнопка выхода
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    width: Val::Px(600.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("Замок"),
                        TextFont {
                            font: font.clone(),
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                    row.spawn((
                        Button,
                        Node {
                            padding: UiRect::all(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.40, 0.10, 0.10)),
                        LeaveButton,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Покинуть город"),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                });

            // Золото
            parent.spawn((
                Text::new(format!("Золото: {gold}")),
                TextFont {
                    font: font.clone(),
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.0)),
            ));

            // Раздел найма
            parent.spawn((
                Text::new("Найм существ:"),
                TextFont {
                    font: font.clone(),
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            for (idx, (unit_type, available)) in town.available_recruits.iter().enumerate() {
                let info = format!(
                    "{}   {}g   HP:{}   Dmg:{}   ×{}",
                    unit_type.name,
                    unit_type.cost.gold,
                    unit_type.hp,
                    unit_type.damage_per_unit,
                    available,
                );
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(16.0),
                        ..default()
                    })
                    .with_children(|row| {
                        row.spawn((
                            Text::new(info),
                            TextFont {
                                font: font.clone(),
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                        row.spawn((
                            Button,
                            Node {
                                padding: UiRect::all(Val::Px(6.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.10, 0.35, 0.10)),
                            HireButton(idx),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Нанять 1"),
                                TextFont {
                                    font: font.clone(),
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                    });
            }

            // Армия героя
            parent.spawn((
                Text::new("Армия героя:"),
                TextFont {
                    font: font.clone(),
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            let army_text = gs.heroes.first().map_or_else(
                || "(пусто)".to_string(),
                |h| {
                    if h.army.0.is_empty() {
                        "(пусто)".to_string()
                    } else {
                        h.army
                            .0
                            .iter()
                            .map(|s| format!("{} ×{}", s.unit_type.name, s.count))
                            .collect::<Vec<_>>()
                            .join(", ")
                    }
                },
            );
            parent.spawn((
                Text::new(army_text),
                TextFont {
                    font,
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));
        });
}

// ---------------------------------------------------------------------------
// Системы
// ---------------------------------------------------------------------------

#[allow(clippy::needless_pass_by_value)]
pub fn spawn_town_ui(
    mut commands: Commands,
    game_state: Res<GameStateResource>,
    current_town: Res<CurrentTownId>,
    asset_server: Res<AssetServer>,
) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    spawn_ui(&mut commands, &game_state.0, current_town.0, font);
}

#[allow(clippy::needless_pass_by_value)]
pub fn despawn_town_ui(mut commands: Commands, root_q: Query<Entity, With<TownScreenRoot>>) {
    for entity in &root_q {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::needless_pass_by_value, clippy::too_many_arguments)]
pub fn handle_town_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    hire_q: Query<(&Interaction, &HireButton), Changed<Interaction>>,
    leave_q: Query<&Interaction, (Changed<Interaction>, With<LeaveButton>)>,
    mut next_state: ResMut<NextState<GameScreen>>,
    mut game_state: ResMut<GameStateResource>,
    current_town: Res<CurrentTownId>,
    mut commands: Commands,
    root_q: Query<Entity, With<TownScreenRoot>>,
    asset_server: Res<AssetServer>,
) {
    // Escape → выход
    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(GameScreen::Adventure);
        return;
    }

    // Кнопка «Покинуть город»
    for interaction in &leave_q {
        if *interaction == Interaction::Pressed {
            next_state.set(GameScreen::Adventure);
            return;
        }
    }

    // Кнопки найма
    let mut hired = false;
    for (interaction, hire_btn) in &hire_q {
        if *interaction == Interaction::Pressed {
            match game_state.0.apply(GameCommand::RecruitUnits {
                town_id: current_town.0,
                unit_type_idx: hire_btn.0,
                count: 1,
            }) {
                Ok(_) => {
                    hired = true;
                    break;
                }
                Err(e) => {
                    warn!("[TOWN] Recruit failed: {:?}", e);
                }
            }
        }
    }

    // Перестроить UI после найма
    if hired {
        for entity in &root_q {
            commands.entity(entity).despawn();
        }
        let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
        spawn_ui(&mut commands, &game_state.0, current_town.0, font);
    }
}
