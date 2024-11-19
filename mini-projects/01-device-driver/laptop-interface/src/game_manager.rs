use crate::{controller::*, list_ports};
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
                // monitor for new controller connections and wait for user to signal to move out of this state
                //list ports
            }
            game_state::ingame =>{
                //actual game logic
            }
            game_state::postgame =>{
                //game has ended, display results and give user options on how to proceed
            }
        }
    }
    //private function that monitors for new controller additions
    fn connect_new_controller(){
        //check to see what ports exist and if any new ones pop up, connect to them
        match list_ports(){
            Ok(ports) =>{
                //using regexes
                //ports.iter().filter(predicate)
            },
            Err(e) =>(),
        }
    }
}