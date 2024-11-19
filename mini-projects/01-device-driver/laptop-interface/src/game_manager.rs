use crate::{controller::*, error::{ControllerError, ModuleError, SerialError}, list_ports};
use regex::Regex;
struct Game{
    state:game_state,
    controller_manager:ControllerManager,
    //players:<Vec<String>,
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
    //Based on currrent game state does different thing
    pub fn run_game(&mut self){
        match self.state{
            game_state::pregame =>{
                //connect to a new controller, if a new controller is added 
                //ask the user if all controllers are connected and players are ready to begin
        
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
    fn connect_new_controller(&mut self)->Result<(),ModuleError>{
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
                        match self.controller_manager.connect_controller(&serial_string){
                            Ok(()) =>return  Ok(()),
                            Err(e) =>return Err(ModuleError::ControllerError(e))
                        }
                    }
                };
                Ok(())
            },
            Err(e) =>Err(ModuleError::SerialError(e)),
        }
    }
}