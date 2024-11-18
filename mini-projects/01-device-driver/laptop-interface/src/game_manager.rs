use crate::controller::*;
struct Game{
    state:game_state,
    controller_manager:ControllerManager,
    players:<Vec<String>,
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