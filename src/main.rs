#![warn(clippy::all, clippy::pedantic)]

mod adventure;
mod battle;
mod core;
mod data;
mod gameover;
mod menu;
mod town;

use adventure::AdventurePlugin;
use battle::BattlePlugin;
use bevy::prelude::*;
use core::hero::{Army, HeroId};
use core::map::Position;
use gameover::GameOverPlugin;
use menu::MainMenuPlugin;
use town::TownPlugin;

/// Экран игры — переключается при входе в город/бой/меню.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameScreen {
    #[default]
    MainMenu,
    Adventure,
    Town,
    Battle,
    GameOver,
}

/// Параметры запуска игры — заполняются главным меню, читаются в `OnEnter(Adventure)`.
#[derive(Resource)]
pub struct GameStartConfig {
    pub map_path: String,
    pub hero_name: String,
}

/// Данные боя, передаваемые из Adventure в Battle при переходе состояния.
#[derive(Resource)]
pub struct PendingBattle {
    pub attacker_hero_id: HeroId,
    pub defender_army: Army,
    pub defender_pos: Position,
}

/// Результат завершённой игры — победа или поражение.
#[derive(Resource)]
pub struct GameOverResult {
    pub is_victory: bool,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Heroes of Rust".to_string(),
                resolution: (1280_u32, 720_u32).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.10, 0.25, 0.10)))
        .init_state::<GameScreen>()
        .add_plugins(MainMenuPlugin)
        .add_plugins(AdventurePlugin)
        .add_plugins(TownPlugin)
        .add_plugins(BattlePlugin)
        .add_plugins(GameOverPlugin)
        .run();
}
