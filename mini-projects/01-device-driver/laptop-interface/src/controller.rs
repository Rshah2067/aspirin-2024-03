#![allow(dead_code)]

use crate::error::{ControllerError, SerialError};
use crate::lib_serial_ffi::*;
use std::ffi::CString;
use std::str;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use log;

pub struct ControllerManager {
    controllers: Vec<Controller>,
    input_receiver: Receiver<String>,
    output_sender: Arc<Mutex<Sender<String>>>,
    controller_senders: Vec<Option<Sender<String>>>,
}

#[derive(Debug, Clone)]
enum ControllerKind {
    Base,
    Advanced,
}

#[derive(Debug, Clone)]
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
            controller_senders: Vec::new(),
        }
    }

    pub fn connect_controller(&mut self, serial_port: &str) -> Result<(), ControllerError> {
        log::info!("Connecting to controller on serial port: {}", serial_port);

        let controller = Controller {
            name: format!("Controller_{}", serial_port.replace("/", "_")),
            id: self.controllers.len() as u32,
            serial_port: serial_port.to_string(),
            kind: ControllerManager::determine_controller_kind(serial_port),
        };

        let thread_controller = controller.clone();
        let thread_sender = self.output_sender.clone();
        let (tx, rx): (Sender<String>, Receiver<String>) = std::sync::mpsc::channel();

        thread::spawn(move || -> Result<(), ControllerError> {
            let port =
                SerialPort::new(CString::new(thread_controller.serial_port.clone()).unwrap())?;

            port.open(sp_mode::SP_MODE_READ_WRITE)?;
            thread::sleep(Duration::from_millis(100));
            log::info!("Port opened successfully in read/write mode");
            port.write("stop controller\n")?;
            thread::sleep(Duration::from_millis(10));
            port.write("reset\n")?;
            thread::sleep(Duration::from_millis(10));
            port.write("init controller\n")?;
            thread::sleep(Duration::from_millis(10));
            port.write("set ready led\n")?;

            let mut buffer = vec![0u8; 1024];
            loop {
                match rx.try_recv() {
                    Ok(message) => {
                        log::debug!("Received message for controller: {}", message);
                        port.write(&message)?;
                    }
                    Err(TryRecvError::Empty) => {
                        // No message received, continue with read operation
                    }
                    Err(TryRecvError::Disconnected) => {
                        log::warn!("Channel to controller disconnected, exiting thread");
                        break;
                    }
                }
                log::trace!("Reading from serial port");
                match port.read(&mut buffer, 100) {
                    Ok(bytes_read) => {
                        if bytes_read > 0 {
                            let received_data =
                                String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                            log::debug!("Read {} bytes from serial port", bytes_read);
                            thread_sender.lock().unwrap().send(received_data)?;
                        }
                    }
                    Err(e) => {
                        match e {
                            SerialError::Timeout => {
                                // Timeout is normal, just continue the loop
                                log::trace!("Read timeout, continuing");
                                thread::sleep(Duration::from_millis(100));
                            }
                            _ => {
                                log::error!("Error reading from serial port: {:?}", e);
                                return Err(e.into());
                            }
                        }
                    }
                }
            }
            Ok(())
        });

        log::debug!("Created controller: {:?}", controller);
        self.controllers.push(controller);
        self.controller_senders.push(Some(tx));
        log::info!("Controller connected successfully");
        Ok(())
    }

    fn send_message_to_controller(&self, id: u32, message: String) -> Result<(), ControllerError> {
        if let Some(Some(sender)) = self.controller_senders.get(id as usize) {
            sender.send(message).map_err(ControllerError::from)?;
            log::debug!("Message sent to controller {} successfully", id);
            Ok(())
        } else {
            log::warn!("Controller {} not found or disconnected", id);
            Err(ControllerError::ControllerNotFound)
        }
    }

    fn disconnect_controller(&mut self, id: u32) {
        if let Err(e) = self.send_message_to_controller(id, String::from("stop controller\n")) {
            log::error!("Failed to send reset message to controller {}: {}", id, e);
        }
        log::info!("Disconnecting controller with id: {}", id);
        if let Some(sender) = self.controller_senders.get_mut(id as usize) {
            *sender = None; // This will close the channel
        }
        let initial_count = self.controllers.len();
        self.controllers.retain(|controller| controller.id != id);
        let final_count = self.controllers.len();
        if initial_count == final_count {
            log::warn!("No controller found with id: {}", id);
        } else {
            log::info!("Controller with id: {} disconnected successfully", id);
        }
    }

    fn read_serial(&self) -> Result<Option<String>, ControllerError> {
        match self.input_receiver.try_recv() {
            Ok(data) => {
                log::debug!("Read serial data: {}", data);
                Ok(Some(data))
            }
            Err(TryRecvError::Empty) => Ok(None),
            Err(e) => Err(e.into()),
        }
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

    pub fn init_controller(&self, id: u32) -> Result<(), ControllerError> {
        log::info!("Initializing controller with id: {}", id);
        self.send_message_to_controller(id, String::from("init controller\n"))
    }

    fn reset_controller(&self, id: u32) -> Result<(), ControllerError> {
        log::info!("Resetting controller with id: {}", id);
        self.send_message_to_controller(id, String::from("reset controller\n"))
    }

    fn restart_controller(&self, id: u32) -> Result<(), ControllerError> {
        log::info!("Restarting controller with id: {}", id);
        self.send_message_to_controller(id, String::from("restart controller\n"))
    }

    fn stop_controller(&self, id: u32) -> Result<(), ControllerError> {
        log::info!("Stopping controller with id: {}", id);
        self.send_message_to_controller(id, String::from("stop controller\n"))
    }

    fn set_controller_led(&self, id: u32, led_state: LedState) -> Result<(), ControllerError> {
        log::info!("Setting controller {} LED to: {:?}", id, led_state);
        let message = format!("set led {}\n", led_state as u8);
        Ok(())
    }
    pub fn get_controller_ids(&self) ->Vec<u32>{
        let output = self.controllers.iter().map(|s|s.id).collect();
        output
    }   
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_connect_real_controller() {
        let _ =
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
                .is_test(true)
                .try_init();

        // Create a new ControllerManager
        let mut manager = ControllerManager::new();

        // Specify the actual serial port where your controller is connected
        // You may need to change this to match your system
        // let real_port = "/dev/ttyUSB0";  // Example for Linux
        // let real_port = "COM3";  // Example for Windows
        let real_port = "/dev/cu.usbmodem101"; // Example for macOS

        // Connect to the real controller
        if let Err(e) = manager.connect_controller(real_port) {
            panic!("Failed to connect to controller: {}", e);
        }

        // Give more time for the connection to establish
        thread::sleep(Duration::from_millis(1000));

        // Assert that the controller was added
        assert_eq!(manager.controllers.len(), 1);

        // Check the properties of the connected controller
        let controller = &manager.controllers[0];
        assert_eq!(controller.serial_port, real_port);
        assert_eq!(controller.id, 0);

        // Switch the controller into run mode
        match manager.send_message_to_controller(0, String::from("set all leds\n")) {
            Ok(_) => log::info!("'set all leds' command sent successfully"),
            Err(e) => log::warn!("Failed to send 'set all leds' command: {:?}", e),
        }

        thread::sleep(Duration::from_millis(10));

        match manager.send_message_to_controller(0, String::from("start controller\n")) {
            Ok(_) => log::info!("'start controller' command sent successfully"),
            Err(e) => log::warn!("Failed to send 'start controller' command: {:?}", e),
        }

        // Give some time for the commands to take effect
        thread::sleep(Duration::from_millis(5000));

        // Wait for some data from the controller
        let mut received_data = false;
        for _ in 0..50 {
            // Try for 5 seconds (50 * 100ms)
            match manager.read_serial() {
                Ok(Some(data)) => {
                    log::info!("Received data from controller: {:?}", data);
                    received_data = true;
                    break;
                }
                Ok(None) => {
                    // No data available, wait and try again
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => panic!("Failed to receive data from controller: {}", e),
            }
        }
        assert!(
            received_data,
            "No data received from controller after 5 seconds"
        );

        // Clean up
        manager.disconnect_controller(0);
    }
}
