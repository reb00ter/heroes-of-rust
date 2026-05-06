pub mod ui;

use bevy::prelude::*;

use crate::GameScreen;
use crate::core::player::TownId;

/// Ресурс: хранит `TownId` города, в котором сейчас находится герой.
#[derive(Resource)]
pub struct CurrentTownId(pub TownId);

/// Маркер кнопки найма; хранит индекс слота в `town.available_recruits`.
#[derive(Component)]
pub struct HireButton(pub usize);

/// Маркер кнопки «Покинуть город».
#[derive(Component)]
pub struct LeaveButton;

pub struct TownPlugin;

impl Plugin for TownPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameScreen::Town), ui::spawn_town_ui)
            .add_systems(OnExit(GameScreen::Town), ui::despawn_town_ui)
            .add_systems(
                Update,
                ui::handle_town_input.run_if(in_state(GameScreen::Town)),
            );
    }
}
