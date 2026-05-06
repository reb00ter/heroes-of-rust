use bevy::prelude::*;

use crate::battle::state::{BattleStack, BattleState, Side, StackId};

use super::input::AttackButton;

#[derive(Component)]
pub struct BattleScreenRoot;

#[derive(Component)]
pub struct BattleStackWidget(#[allow(dead_code)] pub StackId);

#[derive(Component)]
pub struct TurnLabel;

// ---------------------------------------------------------------------------
// OnEnter / OnExit
// ---------------------------------------------------------------------------

#[allow(clippy::needless_pass_by_value)]
pub fn spawn_battle_ui(
    mut commands: Commands,
    battle_state: Res<BattleState>,
    asset_server: Res<AssetServer>,
) {
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    spawn_ui_inner(&mut commands, &battle_state, font);
}

#[allow(clippy::needless_pass_by_value)]
pub fn despawn_battle_ui(
    mut commands: Commands,
    root_q: Query<Entity, With<BattleScreenRoot>>,
) {
    for entity in &root_q {
        commands.entity(entity).despawn();
    }
}

// ---------------------------------------------------------------------------
// Update
// ---------------------------------------------------------------------------

#[allow(clippy::needless_pass_by_value)]
pub fn update_battle_ui(
    mut commands: Commands,
    battle_state: Res<BattleState>,
    root_q: Query<Entity, With<BattleScreenRoot>>,
    asset_server: Res<AssetServer>,
) {
    if !battle_state.is_changed() {
        return;
    }
    for entity in &root_q {
        commands.entity(entity).despawn();
    }
    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");
    spawn_ui_inner(&mut commands, &battle_state, font);
}

// ---------------------------------------------------------------------------
// Построение UI
// ---------------------------------------------------------------------------

fn spawn_ui_inner(commands: &mut Commands, battle: &BattleState, font: Handle<Font>) {
    let current_id = battle.current_stack_id;
    let is_attacker_turn = battle
        .current_stack()
        .map_or(false, |s| s.side == Side::Attacker);

    let attackers: Vec<&BattleStack> = battle
        .stacks
        .iter()
        .filter(|s| s.side == Side::Attacker && s.count > 0)
        .collect();
    let defenders: Vec<&BattleStack> = battle
        .stacks
        .iter()
        .filter(|s| s.side == Side::Defender && s.count > 0)
        .collect();

    let turn_label = battle.current_stack().map_or_else(
        || "Ход: —".to_string(),
        |s| {
            let side_name = if s.side == Side::Attacker {
                "Атакующий"
            } else {
                "Защитник"
            };
            format!("Ход: {} ({})", s.unit_type.name, side_name)
        },
    );

    // Корневой узел
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.98)),
            GlobalZIndex(200),
            BattleScreenRoot,
        ))
        .id();

    // Заголовок
    let title = commands
        .spawn((
            Text::new("БОЙ"),
            TextFont { font: font.clone(), font_size: 36.0, ..default() },
            TextColor(Color::WHITE),
        ))
        .id();
    commands.entity(root).add_child(title);

    // Ряд с двумя колонками
    let content_row = commands
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            width: Val::Percent(100.0),
            flex_grow: 1.0,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .id();
    commands.entity(root).add_child(content_row);

    let att_col = spawn_column(
        commands,
        &font,
        "АТАКУЮЩИЕ",
        Color::srgb(0.6, 1.0, 0.6),
        &attackers,
        current_id,
        is_attacker_turn,
        false,
    );
    commands.entity(content_row).add_child(att_col);

    let def_col = spawn_column(
        commands,
        &font,
        "ЗАЩИТНИКИ",
        Color::srgb(1.0, 0.6, 0.6),
        &defenders,
        current_id,
        is_attacker_turn,
        true,
    );
    commands.entity(content_row).add_child(def_col);

    // Строка хода
    let turn_text = commands
        .spawn((
            Text::new(turn_label),
            TextFont { font, font_size: 22.0, ..default() },
            TextColor(Color::srgb(0.9, 0.9, 0.5)),
            TurnLabel,
        ))
        .id();
    commands.entity(root).add_child(turn_text);
}

#[allow(clippy::too_many_arguments)]
fn spawn_column(
    commands: &mut Commands,
    font: &Handle<Font>,
    header: &str,
    header_color: Color,
    stacks: &[&BattleStack],
    current_id: StackId,
    is_attacker_turn: bool,
    show_attack_button: bool,
) -> Entity {
    let col = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(50.0),
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        })
        .id();

    let header_entity = commands
        .spawn((
            Text::new(header),
            TextFont { font: font.clone(), font_size: 20.0, ..default() },
            TextColor(header_color),
        ))
        .id();
    commands.entity(col).add_child(header_entity);

    for stack in stacks {
        let active = stack.id == current_id;
        let bg_color = if active {
            Color::srgb(0.28, 0.22, 0.04)
        } else {
            Color::srgb(0.12, 0.12, 0.18)
        };
        let name_color = if active {
            Color::srgb(1.0, 0.85, 0.1)
        } else {
            Color::WHITE
        };

        let stack_node = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    row_gap: Val::Px(4.0),
                    ..default()
                },
                BackgroundColor(bg_color),
                BattleStackWidget(stack.id),
            ))
            .id();
        commands.entity(col).add_child(stack_node);

        let prefix = if active { "► " } else { "  " };
        let name_text = commands
            .spawn((
                Text::new(format!(
                    "{}{} ×{}",
                    prefix, stack.unit_type.name, stack.count
                )),
                TextFont { font: font.clone(), font_size: 20.0, ..default() },
                TextColor(name_color),
            ))
            .id();
        commands.entity(stack_node).add_child(name_text);

        let stats_text = commands
            .spawn((
                Text::new(format!(
                    "HP: {}  Dmg: {}",
                    stack.hp_remaining, stack.unit_type.damage_per_unit
                )),
                TextFont { font: font.clone(), font_size: 16.0, ..default() },
                TextColor(Color::srgb(0.75, 0.75, 0.75)),
            ))
            .id();
        commands.entity(stack_node).add_child(stats_text);

        if show_attack_button {
            let btn_bg = if is_attacker_turn {
                Color::srgb(0.55, 0.10, 0.10)
            } else {
                Color::srgb(0.25, 0.25, 0.25)
            };
            let btn = commands
                .spawn((
                    Button,
                    Node {
                        padding: UiRect::all(Val::Px(6.0)),
                        margin: UiRect::top(Val::Px(4.0)),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(btn_bg),
                    AttackButton(stack.id),
                ))
                .id();
            commands.entity(stack_node).add_child(btn);

            let btn_text = commands
                .spawn((
                    Text::new("Атаковать"),
                    TextFont { font: font.clone(), font_size: 16.0, ..default() },
                    TextColor(Color::WHITE),
                ))
                .id();
            commands.entity(btn).add_child(btn_text);
        }
    }

    col
}
