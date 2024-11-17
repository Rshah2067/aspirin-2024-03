#![allow(dead_code)]

use crate::lib_serial_ffi::*;
use crate::error::{SerialError, ControllerError};
use std::ffi::CString;
use std::str;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use log;

struct ControllerManager {
    controllers: Vec<Controller>,
    input_receiver: std::sync::mpsc::Receiver<String>,
    output_sender: Arc<Mutex<std::sync::mpsc::Sender<String>>>,
}

#[derive(Debug)]
enum ControllerKind {
    Base,
    Advanced,
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

    fn connect_controller(&mut self, serial_port: &str) -> Result<(), ControllerError> {
        log::info!("Connecting to controller on serial port: {}", serial_port);

        let controller = Controller {
            name: format!("Controller_{}", serial_port.replace("/", "_")),
            id: self.controllers.len() as u32,
            serial_port: serial_port.to_string(),
            kind: ControllerManager::determine_controller_kind(serial_port),
        };

        let output_sender_clone = Arc::clone(&self.output_sender);
        let serial_port_clone = controller.serial_port.clone();

        thread::spawn(move || -> Result<(), ControllerError> {
            let port = SerialPort::new(CString::new(serial_port_clone.clone()).unwrap())?;

            port.open(sp_mode::SP_MODE_READ_WRITE)?;
            
            log::info!("Port opened successfully in read/write mode");
            port.write(String::from("init controller"))?;
            port.write(String::from("set ready led"))?;

            let mut buffer = vec![0u8; 1024];
            loop {
                let count = buffer.len().min(u16::MAX as usize) as u16;
                match port.read(&mut buffer, count) {
                    Ok(read_slice) => {
                        let bytes_read = read_slice.len();
                        if bytes_read > 0 {
                            let received_data = str::from_utf8(read_slice)?.to_string();
                            output_sender_clone.lock().unwrap().send(received_data)?;
                        }
                    }
                    Err(e) => {
                        return Err(ControllerError::SerialReadError {
                            port: serial_port_clone.clone(),
                            error: e as isize,
                        });
                    }
                }
                thread::sleep(Duration::from_millis(100));
            }
        });

        log::debug!("Created controller: {:?}", controller);
        self.controllers.push(controller);
        log::info!("Controller connected successfully");
        Ok(())
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

    fn read_serial(&self) -> Result<String, ControllerError> {
        let data = self.input_receiver
            .recv_timeout(std::time::Duration::from_secs(5))?;
        log::debug!("Read serial data: {}", data);
        Ok(data)
    }

    fn determine_controller_kind(serial_port: &str) -> ControllerKind {
        log::info!("Determining controller kind for port: {}", serial_port);

        // TODO: Implement actual logic to determine controller kind
        // This could involve sending a command to the controller and analyzing the response

        // For now, we'll use a simple heuristic based on the serial port name
        if serial_port.contains("advanced") {
            log::debug!("Detected Advanced controller on port: {}", serial_port);
            ControllerKind::Advanced
        } else {
            log::debug!("Assumed Base controller on port: {}", serial_port);
            ControllerKind::Base
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_connect_real_controller() {
        let _ = env_logger::Builder::new()
            .is_test(true) // Ensures logs are printed in test mode
            .init();
        // Create a new ControllerManager
        let mut manager = ControllerManager::new();

        // Specify the actual serial port where your controller is connected
        // You may need to change this to match your system
        //let real_port = "/dev/ttyUSB0";  // Example for Linux
        // let real_port = "COM3";  // Example for Windows
        let real_port = "/dev/cu.usbmodem101"; // Example for macOS

        // Connect to the real controller
        manager.connect_controller(real_port);

        // Give some time for the connection to establish
        thread::sleep(Duration::from_secs(2));

        // Assert that the controller was added
        assert_eq!(manager.controllers.len(), 1);

        // Check the properties of the connected controller
        let controller = &manager.controllers[0];
        assert_eq!(controller.serial_port, real_port);
        assert_eq!(controller.id, 0);

        // Wait for some data from the controller
        match manager.read_serial() {
            Ok(data) => {
                println!("Received data from controller: {}", data);
                // Add assertions here based on the expected data format from your controller
            }
            Err(e) => panic!("Failed to receive data from controller: {}", e),
        }

        // Optionally, test sending a command to the controller
        // This depends on your controller's protocol
        // let command = "some_command";
        // manager.send_command_to_controller(0, command);

        // Clean up
        manager.disconnect_controller(0);
    }
}