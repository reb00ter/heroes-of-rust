#![warn(clippy::all, clippy::pedantic)]

mod adventure;
mod battle;
mod core;
mod town;

use adventure::AdventurePlugin;
use battle::BattlePlugin;
use bevy::prelude::*;
use core::hero::{Army, HeroId};
use core::map::Position;
use town::TownPlugin;

/// Экран игры — переключается при входе в город/бой и выходе.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameScreen {
    #[default]
    Adventure,
    Town,
    Battle,
}

/// Данные боя, передаваемые из Adventure в Battle при переходе состояния.
#[derive(Resource)]
pub struct PendingBattle {
    pub attacker_hero_id: HeroId,
    pub defender_army: Army,
    pub defender_pos: Position,
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
        .insert_resource(ClearColor(Color::srgb(0.08, 0.08, 0.12)))
        .init_state::<GameScreen>()
        .add_plugins(AdventurePlugin)
        .add_plugins(TownPlugin)
        .add_plugins(BattlePlugin)
        .run();
}
