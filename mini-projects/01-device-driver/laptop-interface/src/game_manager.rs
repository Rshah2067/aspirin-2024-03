use std::io::{self, Read};

use crate::{controller::*, error::{ControllerError, ModuleError, SerialError}, list_ports};
use log::log;
use regex::Regex;
struct Game{
    state:game_state,
    controller_manager:ControllerManager,
    players:Vec<Player>,
}
struct Player{
    position:(u32,u32),
    score:u32,
    //A player has a player number and a controller number as the controller number corresponds to the
    //actual serial port that the player connected their controller to. The player number is the number
    //player they are (the first to connect is 1, the second is 2 etc) This is more similar to how most 
    //video games work where the player number is not corresponding to the hardware port of the controller
    player_number:usize,
    controller_number:u32,
}
impl Player{
    fn new(id:u32,player:usize)->Self{
        Player { position: (0,0), score: (0), player_number:(player),controller_number: (id) }
    }
}
enum game_state{
    //Pregame is the phase where the user is connecting/initializing controllers
    pregame,
    //in game is when we are actually playing the game and we are monitoring the output
    //of the controllers and updated game UI
    ingame,
    //the current game round has ended and results are displayed, the user is then presented with the opperunity
    postgame,
}
impl Game{
    //Constructor which is run at start
    pub fn new()->Self{
        
        Game { state: game_state::pregame, controller_manager: ControllerManager::new(),players:Vec::new()}
    }
    //Based on currrent game state does different thing
    pub fn run_game(&mut self){
        match self.state{
            game_state::pregame =>{
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
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to Read Line");
                if input == String::from("ready"){
                    //Move State to in game
                    log::info!("Starting Game");
                    self.state = game_state::ingame;
                }
                else if input != "ready" && input != ""{

                }
            }
            game_state::ingame =>{
                //actual game logic
            }
            game_state::postgame =>{
                //game has ended, display results and give user options on how to proceed
            }
        }
    }
    //helper function that monitors for new controller additions
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
    //creates the 
    fn move_to_pregame(){

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
        //Move State to in game
        log::info!("Starting Game");
        self.state = game_state::ingame;
    }
}