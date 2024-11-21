use std::{io::{self, Read}, sync::mpsc::{self, Receiver}, thread, time::Instant};
use plotters::prelude::*;
use crate::{controller::*, error::{ControllerError, ModuleError, SerialError}, list_ports};
use log::{info, log};
use regex::Regex;
pub struct Game{
    state:GameState,
    controller_manager:ControllerManager,
    players:Vec<Player>,
    //Reciever from thread that is monitoring Stdin for user input
    stdin_reciever:Receiver<String>,
    timer:Option<Instant>
}
struct Player{
    position:(i32,i32),
    score:f64,
    //A player has a player number and a controller number as the controller number corresponds to the
    //actual serial port that the player connected their controller to. The player number is the number
    //player they are (the first to connect is 1, the second is 2 etc) This is more similar to how most 
    //video games work where the player number is not corresponding to the hardware port of the controller
    player_number:usize,
    controller_id:u32,
    history:Vec<(f64,f64)>
}
impl Player{
    fn new(id:u32,player:usize)->Self{
        Player { position: (0,0), score: (0.0), player_number:(player),controller_id: (id),history: Vec::new() }
    }
}
#[derive(PartialEq)]
pub enum GameState{
    //Pregame is the phase where the user is connecting/initializing controllers
    Pregame,
    //in game is when we are actually playing the game and we are monitoring the output
    //of the controllers and updated game UI
    Ingame,
    //the current game round has ended and results are displayed, the user is then presented with the opperunity
    Postgame,
    //user decides to end game
    Endgame,
}
impl Game{
    //Constructor which is run at start
    pub fn new()->Self{ 
        Game { 
            state: GameState::Pregame,
            controller_manager: ControllerManager::new(),
            players:Vec::new(),
            stdin_reciever:spawn_stdin_channel(),
            timer:None
        }
    }
    //returns the current state of the game (used to end game)
    pub fn get_gamestate(& self)->&GameState{
        &self.state
    }
    //Based on currrent game state does different thing
    pub fn run_game(&mut self){
        match self.state{
            GameState::Pregame =>{
                //connect to a new controller, if a new controller is added ask if this if the player wants to start
                match self.connect_new_controller(){
                    Ok(Some(player)) => {
                        println!("Connected Player {}",player);
                        println!("If All Controllers are Connected, Start Game by Typing \"ready\"");
                    }
                    Ok(None) =>(),
                    Err(e) =>eprintln!("Error Connecting Controller {} Try Again",e)
                };
                let mut input = String::new();
                //Check if the User has indicated to start the game
                let mut input = String::new();
                match self.stdin_reciever.try_recv(){
                    Ok(user) =>input = user,
                    //If we don't have any user input we move on
                    Err(_) =>(),
                }
                //see what to do with user input
                match input.as_str(){
                    "ready" =>self.move_to_game(),
                    //if there is no input do nothing
                   ""=> (),
                    _ =>println!("Invalid Input Please Enter \"ready\" to start game")
                }
            }
            GameState::Ingame =>{
                //actual game logic
                //Check for New controller
                //Poll each players controller
               for player in self.players.iter_mut(){
                    //get the new state of their controller
                    self.controller_manager.get_controller_state(player.controller_number);
                    //calculate their new position and score
                    player.score = player.position.0*player.position.0;
                    //add there score to their history if at least 5 seconds have passed (so we don't use too much memory)
                    //can safely unwrap as our state machine ensures that we will have a timer here
                    if self.timer.unwrap().elapsed().as_secs() >=5{
                        player.history.push((player.score,self.timer.unwrap().elapsed().as_secs_f32().into()));
                    }
                }
                //update player information and print current game state

                //check to see if users have indicated for game to end
                let mut input = String::new();
                //Check if the User has indicated to start the game
                let mut input = String::new();
                match self.stdin_reciever.try_recv(){
                    Ok(user) =>input = user,
                    //If we don't have any user input we move on
                    Err(_) =>(),
                }
                //see what to do with user input
                match input.as_str(){
                    "end" =>self.move_to_postgame(),
                    //if there is no input do nothing
                   ""=> (),
                    _ =>println!("Invalid Input Please Enter \"end\" to end game")
                }
            }
            GameState::Postgame =>{
                //game has ended, monitor how to user wants to proceed
                let mut input = String::new();
                //Check if the User has indicated to start the game
                let mut input = String::new();
                match self.stdin_reciever.try_recv(){
                    Ok(user) =>input = user,
                    //If we don't have any user input we move on
                    Err(_) =>(),
                }
                //see what to do with user input
                match input.as_str(){
                    "end" =>self.move_to_endgame(),
                    //if there is no input do nothing
                   ""=> (),
                    _ =>println!("Invalid Input Please Enter \"end\" to end game")
                }
            }
            GameState::Endgame =>{
                //User wants to stop playing, shut down and signal to main loop to end
            }
        }
    }
    //TODO port to controller Manager
    //helper function that monitors for new controller additions, returns a controller id if it connects
    //a new controller
    fn connect_new_controller(&mut self)->Result<Option<u32>,ModuleError>{
        //check to see what ports exist and if any new ones pop up, connect to them
        match list_ports(){
            Ok(ports) =>{
                //get a list of controller ID's that are already connected
                let ids = self.controller_manager.get_controller_ids();
                //using regexes
                let regex = Regex::new(r"^/dev/ttyACM(\d+)$").unwrap();
                //am calling unwrap as regex garuntees sucessful parsing
                let validports:Vec<u32> = ports
                .iter()
                .filter_map(|s| regex.captures(s).and_then(|caps| caps.get(1).map(|m| m.as_str().parse::<u32>().ok().unwrap())))
                .collect();
                //go through the connected ports and if there is a connected port that is not an existing port, connect to it
                for port in validports{
                    if !ids.contains(&port){
                        let mut serial_string:String = String::from("/dev/ttyACM");
                        serial_string.push_str(&port.to_string());
                        //return the player number correspondign to the connected control
                        match self.controller_manager.connect_controller(&serial_string){
                            Ok(()) =>return {
                                //add the player corrosponding to the controller
                                self.add_player(port);
                                Ok(Some(port))
                            },
                            Err(e) =>return Err(ModuleError::ControllerError(e))
                        }
                    }
                };
                Ok(None)
            },
            Err(e) =>Err(ModuleError::SerialError(e)),
        }
    }
    //Adds a new Player to the game
    fn add_player(&mut self,id:u32)->usize{
        let player_number = self.players.len()+1;
        self.players.push(Player::new(id,player_number));
        player_number
    }
    //Sets game state to pregram and resets Player points and positions. Prompts User For input
    fn move_to_pregame(&mut self){
        //go through all players and reset their scores and state
        for player in self.players.iter_mut(){
            player.position = (0,0);
            player.score = 0.0;
            player.history = Vec::new();
            println!("Player {} is connected", player.player_number);
        };
        println!("In order to start game enter \"ready\"");
        self.state = GameState::Pregame;
        log::info!("Moved To Pregame");

    }
    //Command Controllers to move to gain and switch internal state
    fn move_to_game(&mut self){
        //Intialize Controllers
        let ids = self.controller_manager.get_controller_ids();
        for id in ids{
            match self.controller_manager.init_controller(id){
                Ok(_) => (),
                Err(e) =>eprint!("Error Initializing Controller, Check that they were all Plugged in")
            }
        }
        //Start a Timer that is used to plot data and end the game
        self.timer = Some(Instant::now());
        //Move State to in game
        log::info!("Starting Game");
        self.state = GameState::Ingame;
    }
    fn move_to_postgame(&mut self){
        //move controllers
        //Determine Winner
        let  (mut winner,mut score) = (self.players[0].player_number,self.players[0].score);
        for player in self.players.iter_mut(){
            if player.score >score{
                winner = player.player_number;
                score = player.score;
            }
        }
        //Print Results
        println!("Game Over!");
        println!("Player {} won with {} points",winner,score);
        //Plot Results for each player
        for player in self.players.iter_mut(){
            let mut plot_name = String::from("Game Result For Player");
            plot_name.push_str(&player.player_number.to_string());
            let root_area = BitMapBackend::new("images/2.6.png", (600, 400))
            .into_drawing_area();
            root_area.fill(&BLACK).unwrap();
            let mut ctx = ChartBuilder::on(&root_area)
                .set_label_area_size(LabelAreaPosition::Left, 40)
                .set_label_area_size(LabelAreaPosition::Bottom, 40)
                .caption(plot_name, ("sans-serif", 40))
                .build_cartesian_2d(-10.0..50.0, -10.0..50.0)
                .unwrap();
        
            ctx.configure_mesh().draw().unwrap();
            ctx.draw_series(
                player.history.iter().map(|point| TriangleMarker::new(*point, 5, &BLUE)),
            )
            .unwrap();
        }

        println!("To start a new game enter \"new game\" to stop playing enter \"end\"");
        self.state = GameState::Postgame;
    }
    //drops controller manager and all of the players and indicates for the main function to drop and exit
    fn move_to_endgame(&mut self){
        self.state = GameState::Endgame;
    }
}

//Function that creates a reciever for a thread that monitors stdin, during different game phases this monitors
//different game functions 
fn spawn_stdin_channel()->Receiver<String>{
    let (tx,rx) = mpsc::channel();
    //thread that captures std::in and sends a message with string inputs
    thread::spawn(move||{
        loop{
            let mut buffer = String::new();
            io::stdin()
                .read_line(&mut buffer)
                .expect("Failed to Read Line");
            //this should always be sucessful, because the reciever should be dropped at the same
            //as the tx
            tx.send(buffer).unwrap();
        }   
    });
    rx
}
//converts a controller state into a change in character position
fn controller_state_to_movement(state:ControllerState) ->(i32,i32){
    let (mut x,mut y):(i32,i32) = (0,0);
    if state.north_east{
        x +=1;
        y +=1;
    };
    if state.north_west{
        x +=-1;
        y +=1;
    };
    if state.south_east{
        x +=1;
        y +=-1;
    };
    if state.north_west{
        x +=-1;
        y +=-1;
    };
    if let Some(true)=state.north{
        y+=1;
    };
    if let Some(true)=state.south{
        y+=-1;
    };
    (x,y)
}