use crate::{controller::*, error::ModuleError, lib_serial_ffi::list_ports};
use log::{debug, error, info, warn};
use plotters::prelude::*;
use regex::Regex;
use std::{
    io::{self},
    sync::mpsc::{self, Receiver},
    thread,
    time::Instant,
};

/// Represents the main game structure.
pub struct Game {
    state: GameState,
    controller_manager: ControllerManager,
    players: Vec<Player>,
    // Receiver from thread that is monitoring Stdin for user input
    stdin_receiver: Receiver<String>,
    timer: Option<Instant>,
}

/// Represents a player in the game.
struct Player {
    position: (i32, i32),
    score: f64,
    /// A player has a player number and a controller number as the controller number corresponds to the
    /// actual serial port that the player connected their controller to. The player number is the number
    /// player they are (the first to connect is 1, the second is 2, etc.). This is more similar to how most
    /// video games work where the player number does not correspond to the hardware port of the controller.
    player_number: usize,
    controller_id: u32,
    history: Vec<(f64, f64)>,
}

impl Player {
    /// Creates a new player with the given controller ID and player number.
    fn new(id: u32, player: usize) -> Self {
        Player {
            position: (0, 0),
            score: 0.0,
            player_number: player,
            controller_id: id,
            history: Vec::new(),
        }
    }
}

/// Represents the various states of the game.
#[derive(PartialEq)]
pub enum GameState {
    /// Pregame is the phase where the user is connecting/initializing controllers.
    Pregame,
    /// Ingame is when we are actually playing the game and monitoring controller outputs and updating the game UI.
    Ingame,
    /// The current game round has ended, results are displayed, and the user is presented with options on how to proceed.
    Postgame,
    /// User decides to end the game.
    Endgame,
}

impl Game {
    /// Constructor which is run at start.
    pub fn new() -> Self {
        println!("To start the game, enter \"ready\".");
        Game {
            state: GameState::Pregame,
            controller_manager: ControllerManager::new(),
            players: Vec::new(),
            stdin_receiver: spawn_stdin_channel(),
            timer: None,
        }
    }

    /// Returns the current state of the game (used to end game).
    pub fn get_gamestate(&self) -> &GameState {
        &self.state
    }

    /// Based on current game state, performs different actions.
    pub fn run_game(&mut self) {
        match self.state {
            GameState::Pregame => {
                // Connect to a new controller using Game's helper method
                match self.controller_manager.connect_new_controller() {
                    Ok(Some(id)) => {
                        let player_number = self.add_player(id);
                        println!("Connected Player {}", player_number);
                        println!(
                            "If all controllers are connected, start the game by typing \"ready\"."
                        );
                    }
                    Ok(None) => (),
                    Err(e) => {
                        error!("Error Connecting Controller: {}. Try Again.", e);
                        println!("Error Connecting Controller: {}. Try Again.", e);
                    }
                }

                // Check if the user has indicated to start the game
                let mut input = String::new();
                match self.stdin_receiver.try_recv() {
                    Ok(user) => input = user.trim().to_string(),
                    // If we don't have any user input, move on
                    Err(_) => (),
                }

                // Determine action based on user input
                match input.as_str() {
                    "ready" => self.move_to_game(),
                    // If there is no input, do nothing
                    "" => (),
                    _ => {
                        warn!("Invalid Input: {}. Expected \"ready\".", input);
                        println!("Invalid Input. Please enter \"ready\" to start the game.");
                    }
                }
            }

            GameState::Ingame => {
                // Actual game logic

                // Check for new controller
                match self.controller_manager.connect_new_controller() {
                    Ok(Some(id)) => {
                        let player_number = self.add_player(id);
                        println!("Connected Player {}", player_number);
                    }
                    Ok(None) => (),
                    Err(e) => {
                        error!("Error Connecting Controller: {}", e);
                        println!("Error Connecting Controller: {}", e);
                    }
                }

                // Poll each player's controller
                for player in self.players.iter_mut() {
                    // Get the new state of their controller
                    match self
                        .controller_manager
                        .get_controller_state(player.controller_id)
                    {
                        Some(state) => {
                            // Convert controller state to movement
                            let movement = controller_state_to_movement(state);
                            player.position.0 += movement.0;
                            player.position.1 += movement.1;

                            // Update score based on position
                            player.score = (player.position.0 as f64).powi(2)
                                + (player.position.1 as f64).powi(2);

                            // Add score to history if at least 5 seconds have passed
                            if let Some(timer) = self.timer {
                                if timer.elapsed().as_secs() >= 5 {
                                    player
                                        .history
                                        .push((player.score, timer.elapsed().as_secs_f64()));
                                    self.timer = Some(Instant::now());
                                }
                            }
                        }
                        None => {
                            // Optionally handle the case where there is no new controller state
                        }
                    }
                }

                // Update player information and print current game state
                self.print_game_state();

                // Check to see if users have indicated to end the game
                let mut input = String::new();
                match self.stdin_receiver.try_recv() {
                    Ok(user) => input = user.trim().to_string(),
                    // If we don't have any user input, move on
                    Err(_) => (),
                }

                // Determine action based on user input
                match input.as_str() {
                    "end" => self.move_to_postgame(),
                    // If there is no input, do nothing
                    "" => (),
                    _ => {
                        warn!("Invalid Input: {}. Expected \"end\".", input);
                        println!("Invalid Input. Please enter \"end\" to end the game.");
                    }
                }
            }

            GameState::Postgame => {
                // Game has ended, monitor how the user wants to proceed
                let mut input = String::new();
                match self.stdin_receiver.try_recv() {
                    Ok(user) => input = user.trim().to_string(),
                    // If we don't have any user input, move on
                    Err(_) => (),
                }

                // Determine action based on user input
                match input.as_str() {
                    "new game" => self.move_to_pregame(),
                    "end" => self.move_to_endgame(),
                    // If there is no input, do nothing
                    "" => (),
                    _ => {
                        warn!(
                            "Invalid Input: {}. Expected \"new game\" or \"end\".",
                            input
                        );
                        println!("Invalid Input. Please enter \"new game\" to start a new game or \"end\" to stop playing.");
                    }
                }
            }

            GameState::Endgame => {
                // User wants to stop playing, shut down and signal to main loop to end
                info!("Ending game. Thank you for playing!");
                println!("Ending game. Thank you for playing!");
                // Here you would typically perform any necessary cleanup
            }
        }
    }


    /// Adds a new player to the game.
    fn add_player(&mut self, id: u32) -> usize {
        let player_number = self.players.len() + 1;
        self.players.push(Player::new(id, player_number));
        info!("Added Player {} with Controller ID {}", player_number, id);
        player_number
    }

    /// Sets game state to Pregame and resets player points and positions. Prompts user for input.
    fn move_to_pregame(&mut self) {
        // Reset all players
        for player in self.players.iter_mut() {
            player.position = (0, 0);
            player.score = 0.0;
            player.history.clear();
            info!("Player {} is connected.", player.player_number);
            println!("Player {} is connected.", player.player_number);
        }
        println!("To start the game, enter \"ready\".");
        self.state = GameState::Pregame;
        info!("Moved to Pregame.");
    }

    /// Initializes controllers and switches the game state to Ingame.
    fn move_to_game(&mut self) {
        // Initialize Controllers
        let ids = self.controller_manager.get_controller_ids();
        for id in ids {
            match self.controller_manager.init_controller(id) {
                Ok(_) => {
                    info!("Initialized Controller {}", id);
                }
                Err(e) => {
                    error!("Error Initializing Controller {}: {}", id, e);
                    println!("Error Initializing Controller {}: {}", id, e);
                }
            }
        }

        // Start a Timer that is used to plot data and end the game
        self.timer = Some(Instant::now());

        // Move State to Ingame
        info!("Starting Game.");
        print!("Game Has Started, Have Fun!");
        self.state = GameState::Ingame;
    }

    /// Transitions the game state to Postgame, determines the winner, and displays results.
    fn move_to_postgame(&mut self) {
        if self.players.is_empty() {
            info!("No players connected. Ending game.");
            println!("No players connected. Ending game.");
            self.move_to_endgame();
            return;
        }

        // Determine Winner
        let (winner, score) = self.players.iter().fold(
            (&self.players[0].player_number, self.players[0].score),
            |(current_winner, current_score), player| {
                if player.score > current_score {
                    (&player.player_number, player.score)
                } else {
                    (current_winner, current_score)
                }
            },
        );

        // Print Results
        info!("Game Over! Player {} won with {:.2} points.", winner, score);
        println!("Game Over!");
        println!("Player {} won with {:.2} points.", winner, score);

        // Plot Results for each player
        for player in &self.players {
            let plot_name = format!("Game Result For Player {}", player.player_number);
            let filename = format!("images/player_{}.png", player.player_number);
            let root_area = BitMapBackend::new(&filename, (600, 400)).into_drawing_area();
            root_area.fill(&BLACK).unwrap();
            let mut ctx = ChartBuilder::on(&root_area)
                .set_label_area_size(LabelAreaPosition::Left, 40)
                .set_label_area_size(LabelAreaPosition::Bottom, 40)
                .caption(plot_name, ("sans-serif", 40))
                .build_cartesian_2d(0.0..50.0, 0.0..50.0)
                .unwrap();

            ctx.configure_mesh().draw().unwrap();
            ctx.draw_series(
                player
                    .history
                    .iter()
                    .map(|point| TriangleMarker::new(*point, 5, &BLUE)),
            )
            .unwrap();
        }

        println!("To start a new game, enter \"new game\". To stop playing, enter \"end\".");
        info!("Moved to Postgame.");
        self.state = GameState::Postgame;
    }

    /// Drops controller manager and all of the players, indicating for the main function to drop and exit.
    fn move_to_endgame(&mut self) {
        info!("Ending game. Thank you for playing!");
        println!("Ending game. Thank you for playing!");
        self.state = GameState::Endgame;
    }

    /// Prints the current game state, including player positions and scores.
    fn print_game_state(&self) {
        info!("Current Game State:");
        for player in &self.players {
            info!(
                "Player {} - Position: ({}, {}), Score: {:.2}",
                player.player_number, player.position.0, player.position.1, player.score
            );
        }
    }
}

/// Function that creates a receiver for a thread that monitors stdin. During different game phases, this monitors
/// different game functions.
fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    // Thread that captures stdin and sends messages with string inputs
    thread::spawn(move || loop {
        let mut buffer = String::new();
        if io::stdin().read_line(&mut buffer).is_ok() {
            // Trim the input to remove trailing newline characters
            let input = buffer.trim().to_string();
            // This should always be successful because the receiver should be dropped at the same
            // time as the sender
            if tx.send(input).is_err() {
                break;
            }
        } else {
            break;
        }
    });
    rx
}

/// Converts a controller state into a change in character position.
fn controller_state_to_movement(state: ControllerState) -> (i32, i32) {
    let mut x: i32 = 0;
    let mut y: i32 = 0;

    if state.north_east {
        x += 1;
        y += 1;
    }
    if state.north_west {
        x -= 1;
        y += 1;
    }
    if state.south_east {
        x += 1;
        y -= 1;
    }
    if state.south_west {
        x -= 1;
        y -= 1;
    }
    if let Some(true) = state.north {
        y += 1;
    }
    if let Some(true) = state.south {
        y -= 1;
    }
    if let Some(true) = state.east {
        x += 1;
    }
    if let Some(true) = state.west {
        x -= 1;
    }

    (x, y)
}
