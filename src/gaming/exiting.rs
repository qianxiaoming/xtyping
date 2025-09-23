use bevy::asset::AssetServer;
use bevy::prelude::*;
use crate::{GameFonts, GamePlayer, PlayState};
use crate::gaming::common::LastPlayState;
use crate::ui::{spawn_image_node, spawn_info_text};
use crate::widgets::ModelDialog;
pub fn confirm_exit_setup(mut commands: Commands,
                          game_player: Res<GamePlayer>,
                          game_fonts: Res<GameFonts>,
                          asset_server: Res<AssetServer>) {
    let dialog = ModelDialog::new(&mut commands, PlayState::Exiting, 60.);
    commands.entity(dialog.container).with_children(|builder| {
        
    });

    commands.insert_resource(LastPlayState(PlayState::Exiting));
}
