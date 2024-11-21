#![allow(dead_code)]
mod lib_serial_ffi;
use game_manager::Game;
use lib_serial_ffi::*;
mod error;

mod controller;
mod game_manager;
use game_manager::GameState;
fn main() {
    let mut game = Game::new();
    //Main Loop
    while *game.get_gamestate() != GameState::Endgame{
        game.run_game();
    }
    println!("Thank You For Playing!")
    // match list_ports() {
    //     Ok(list) => println!("{:?}", list),
    //     Err(e) => eprint!("Failed to List Ports: {}", e),
    // }
    // let port = SerialPort::new(CString::from(c"/dev/cu.usbmodem101"));
    // if let Ok(serialport) = port {
    //     match serialport.open(sp_mode::SP_MODE_READ_WRITE) {
    //         Ok(_) => {
    //             let _ = serialport.write("init controller");
    //             let _ = serialport.write("set ready led");
    //         }
    //         Err(e) => eprint!("Failed to open Port{}", e),
    //     }
    // } else {
    //     let _ = port.inspect_err(|e| eprint!("{}", e));
    // }
}
