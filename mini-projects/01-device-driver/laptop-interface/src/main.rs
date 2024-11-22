#![allow(dead_code)]

use env_logger;
use laptop_interface::game_manager::{Game, GameState};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .init();
    let mut game = Game::new();
    //Main Loop
    println!("Welcome to the Game!");
    while *game.get_gamestate() != GameState::Endgame {
        game.run_game();
    }
    println!("Thank You For Playing!")
}
