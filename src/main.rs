#![warn(clippy::all, clippy::pedantic)]

mod adventure;
mod core;
mod town;

use adventure::AdventurePlugin;
use bevy::prelude::*;
use town::TownPlugin;

/// Экран игры — переключается при входе в город и выходе из него.
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum GameScreen {
    #[default]
    Adventure,
    Town,
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
        .run();
}
