use crate::game::{character, legacy};
use crate::model::{Condition, GameState};
use crate::persistence::save_game;
use crate::ui::{choose_from_list, narrate, pause, prompt};
use std::path::Path;

macro_rules! println {
    () => {
        crate::ui::line("");
    };
    ($($arg:tt)*) => {
        crate::ui::line(&format!($($arg)*))
    };
}
