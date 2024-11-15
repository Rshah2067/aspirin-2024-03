use std::sync::Mutex;
use std::sync::Arc;

use log;

struct ControllerManager {
    controllers: Vec<Controller>,
    input_receiver: std::sync::mpsc::Receiver<String>,
    output_sender: Arc<Mutex<std::sync::mpsc::Sender<String>>>,
}

#[derive(Debug)]
enum ControllerKind {
    Base,
}

#[derive(Debug)]
struct Controller {
    name: String,
    id: u32,
    serial_port: String,
    kind: ControllerKind,
}

impl ControllerManager {
    fn new() -> ControllerManager {
        let (output_sender, output_receiver) = std::sync::mpsc::channel();
        ControllerManager {
            controllers: Vec::new(),
            input_receiver: output_receiver,
            output_sender: Arc::new(Mutex::new(output_sender)),
        }
    }

    fn connect_controller(&mut self, serial_port: &str) {
        log::info!("Connecting to controller on serial port: {}", serial_port);
        std::thread::spawn(|| {todo!()}); //read the controller's serial port and send the data back to the main thread
        let controller = Controller {
            name: "Controller".to_string(), //TODO: figure out how to actually name controllers
            id: self.controllers.len() as u32, //TODO: figure out how to actually assign ids
            serial_port: serial_port.to_string(),
            kind: ControllerKind::Base, //TODO: figure out how to actually determine the kind of controller
        };
        log::debug!("Created controller: {:?}", controller);
        self.controllers.push(controller);
        log::info!("Controller connected successfully");
    }

    fn disconnect_controller(&mut self, id: u32) {
        log::info!("Disconnecting controller with id: {}", id);
        let initial_count = self.controllers.len();
        self.controllers.retain(|controller| controller.id != id);
        let final_count = self.controllers.len();
        if initial_count == final_count {
            log::warn!("No controller found with id: {}", id);
        } else {
            log::info!("Controller with id: {} disconnected successfully", id);
        }
    }

    fn get_inputs 

}
