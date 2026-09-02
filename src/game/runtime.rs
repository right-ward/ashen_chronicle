use crate::game::{lifecycle, world, world_screen};
use crate::model::GameState;
use crate::persistence::character_save_path;
use std::io;
use std::path::PathBuf;

pub(crate) fn main_loop(state: &mut GameState, save_path: &mut PathBuf) -> io::Result<()> {
    loop {
        if !state.character.alive {
            if !lifecycle::death_screen(state)? {
                return Ok(());
            }
            *save_path = character_save_path(PathBuf::from(".").as_path(), &state.character.name);
            world::bootstrap_campaign_content(state);
            continue;
        }
        if world_screen::run(state, save_path)? {
            return Ok(());
        }
    }
}
