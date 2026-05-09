use bevy::prelude::*;

use crate::adventure::{GameStateResource, ShowBanner, build_initial_game_state};
use crate::{GameOverResult, GameScreen};

// ---------------------------------------------------------------------------
// Компоненты
// ---------------------------------------------------------------------------

#[derive(Component)]
struct GameOverRoot;

#[derive(Component)]
struct PlayAgainButton;

// ---------------------------------------------------------------------------
// Плагин
// ---------------------------------------------------------------------------

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScreen::GameOver), spawn_gameover_ui)
            .add_systems(OnExit(GameScreen::GameOver), despawn_gameover_ui)
            .add_systems(
                Update,
                handle_gameover_input.run_if(in_state(GameScreen::GameOver)),
            );
    }
}

// ---------------------------------------------------------------------------
// Системы
// ---------------------------------------------------------------------------

#[allow(clippy::needless_pass_by_value)]
fn spawn_gameover_ui(
    mut commands: Commands,
    game_over: Option<Res<GameOverResult>>,
    asset_server: Res<AssetServer>,
) {
    let is_victory = game_over.as_ref().is_some_and(|r| r.is_victory);
    let (label_text, label_color) = if is_victory {
        ("ПОБЕДА!", Color::srgb(0.15, 0.90, 0.15))
    } else {
        ("ПОРАЖЕНИЕ", Color::srgb(0.90, 0.15, 0.15))
    };

    info!(
        "[GAMEOVER] Showing game over screen: {}",
        if is_victory { "Victory" } else { "Defeat" }
    );

    let font: Handle<Font> = asset_server.load("fonts/Roboto-Regular.ttf");

    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            GlobalZIndex(200),
            GameOverRoot,
        ))
        .id();

    let label = commands
        .spawn((
            Text::new(label_text),
            TextFont {
                font: font.clone(),
                font_size: 96.0,
                ..default()
            },
            TextColor(label_color),
        ))
        .id();

    let button = commands
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(60.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::WHITE),
            PlayAgainButton,
        ))
        .id();

    let button_text = commands
        .spawn((
            Text::new("Играть снова"),
            TextFont {
                font,
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::srgb(0.05, 0.05, 0.05)),
        ))
        .id();

    commands.entity(button).add_child(button_text);
    commands.entity(root).add_children(&[label, button]);
}

#[allow(clippy::needless_pass_by_value)]
fn despawn_gameover_ui(mut commands: Commands, root_q: Query<Entity, With<GameOverRoot>>) {
    for entity in &root_q {
        commands.entity(entity).despawn();
    }
}

#[allow(clippy::needless_pass_by_value)]
fn handle_gameover_input(
    mut commands: Commands,
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<PlayAgainButton>)>,
    mut game_state: ResMut<GameStateResource>,
    mut show_banner: ResMut<ShowBanner>,
    mut next_state: ResMut<NextState<GameScreen>>,
) {
    for interaction in &interaction_q {
        if *interaction == Interaction::Pressed {
            info!("[GAMEOVER] Play again pressed. Resetting game state.");

            // Сбросить игровое состояние
            game_state.0 = build_initial_game_state();

            // Сбросить баннер боя
            show_banner.0 = None;

            // Убрать результат
            commands.remove_resource::<GameOverResult>();

            // Вставить маркер сброса карты
            commands.insert_resource(crate::adventure::NeedsMapReset);

            // Переход на карту
            next_state.set(GameScreen::Adventure);
        }
    }
}
