mod content;
mod events;
mod game;
mod model;
mod persistence;
pub mod ui;

fn main() {
    if let Err(err) = game::run() {
        eprintln!("Fatal error: {err}");
        std::process::exit(1);
    }
}
