#![allow(dead_code)]

use crate::error::{ControllerError, ModuleError, SerialError};
use crate::lib_serial_ffi::*;
use std::ffi::CString;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::{default, str};

use log;
use regex::Regex;

pub struct ControllerManager {
    controllers: Vec<Controller>,
    input_receiver: Receiver<(u32, String)>,
    output_sender: Arc<Mutex<Sender<(u32, String)>>>,
    controller_senders: Vec<Option<Sender<String>>>,
}
#[derive(Clone, Debug)]
pub struct ControllerState {
    pub north_east: bool,
    pub north_west: bool,
    pub south_east: bool,
    pub south_west: bool,
    pub north: Option<bool>,
    pub south: Option<bool>,
    pub east: Option<bool>,
    pub west: Option<bool>,
}

impl ControllerState {
    /// Creates a new ControllerState from a bitmask.
    /// Bit 3: Northwest button
    /// Bit 2: Southwest button
    /// Bit 1: Southeast button
    /// Bit 0: Northeast button
    pub fn from_bitmask(bitmask: u8) -> Self {
        ControllerState {
            north_east: (bitmask & 0b0001) != 0,
            south_east: (bitmask & 0b0010) != 0,
            south_west: (bitmask & 0b0100) != 0,
            north_west: (bitmask & 0b1000) != 0,
            north: None, // Optional buttons not present in the current bitmask
            south: None,
            east: None,
            west: None,
        }
    }
}

impl default::Default for ControllerState {
    fn default() -> Self {
        ControllerState {
            north_east: false,
            north_west: false,
            south_east: false,
            south_west: false,
            north: None,
            south: None,
            east: None,
            west: None,
        }
    }
}

#[derive(Debug, Clone)]
enum ControllerKind {
    Base,
    Advanced,
}

#[derive(Debug)]
pub enum LedState {
    Ready,
    Set,
    Go,
    AllOn,
    AllOff,
}

#[derive(Debug, Clone)]
struct Controller {
    name: String,
    id: u32,
    serial_port: String,
    kind: ControllerKind,
    state: ControllerState,
}

impl ControllerManager {
    pub fn new() -> ControllerManager {
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
            state: ControllerState::default(),
        };

        log::trace!("Controller: {:?}", controller);

        let thread_controller = controller.clone();
        let thread_sender = self.output_sender.clone();
        let (tx, rx): (Sender<String>, Receiver<String>) = std::sync::mpsc::channel();

        // Push the controller and sender before initializing
        self.controllers.push(controller.clone());
        self.controller_senders.push(Some(tx.clone()));

        // Initialize controller before spawning thread
        let controller_id = controller.id;
        self.stop_controller(controller_id)?;
        self.reset_controller(controller_id)?;
        self.init_controller(controller_id)?;
        self.set_controller_led(controller_id, LedState::Ready)?;

        thread::spawn(move || -> Result<(), ControllerError> {
            let port =
                SerialPort::new(CString::new(thread_controller.serial_port.clone()).unwrap())?;

            port.open(sp_mode::SP_MODE_READ_WRITE)?;

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
                            thread_sender
                                .lock()
                                .unwrap()
                                .send((controller_id, received_data))?;
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

        log::info!("Controller connected successfully");
        Ok(())
    }

    fn send_message_to_controller(&self, id: u32, message: String) -> Result<(), ControllerError> {
        match self.controller_senders.get(id as usize) {
            Some(Some(sender)) => {
                sender.send(message)?;
                log::debug!("Message sent to controller {} successfully", id);
                Ok(())
            }
            Some(None) => {
                log::warn!(
                    "Controller sender for {} is None, possibly disconnected",
                    id
                );
                Err(ControllerError::MessageSendError)
            }
            None => {
                log::warn!("Controller {} not found", id);
                Err(ControllerError::ControllerNotFound)
            }
        }
    }

    pub fn disconnect_controller(&mut self, id: u32) {
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

    fn read_serial(&self) -> Result<Option<(u32, String)>, ControllerError> {
        match self.input_receiver.try_recv() {
            Ok(data) => {
                log::debug!("Read serial data (Controller {}): {}", data.0, data.1);
                Ok(Some(data))
            }
            Err(TryRecvError::Empty) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn update_controller_state(&mut self) {
        if let Ok(Some((id, data))) = self.read_serial() {
            if let Some(controller) = self.controllers.iter_mut().find(|c| c.id == id) {
                // Split data into lines and take the last non-empty one
                if let Some(last_line) = data
                    .lines()
                    .rev()
                    .filter(|line| !line.trim().is_empty())
                    .next()
                {
                    if let Ok(bitmask) = u8::from_str_radix(last_line.trim(), 16) {
                        controller.state = ControllerState::from_bitmask(bitmask);
                        log::debug!(
                            "Updated state for controller {}: {:?}",
                            id,
                            controller.state
                        );
                    } else {
                        log::warn!(
                            "Received invalid data format for controller {}: {}",
                            id,
                            last_line
                        );
                    }
                } else {
                    log::warn!("Received empty data for controller {}: {}", id, data);
                }
            } else {
                log::warn!("Received data for unknown controller ID: {}", id);
            }
        }
    }

    pub fn get_controller_state(&self, id: u32) -> Option<ControllerState> {
        self.controllers
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.state.clone())
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

    pub fn reset_controller(&self, id: u32) -> Result<(), ControllerError> {
        log::info!("Resetting controller with id: {}", id);
        self.send_message_to_controller(id, String::from("reset controller\n"))
    }

    pub fn restart_controller(&self, id: u32) -> Result<(), ControllerError> {
        log::info!("Restarting controller with id: {}", id);
        self.send_message_to_controller(id, String::from("restart controller\n"))
    }

    pub fn stop_controller(&self, id: u32) -> Result<(), ControllerError> {
        log::info!("Stopping controller with id: {}", id);
        self.send_message_to_controller(id, String::from("stop controller\n"))
    }

    pub fn set_controller_led(&self, id: u32, led_state: LedState) -> Result<(), ControllerError> {
        log::info!("Setting controller {} LED to: {:?}", id, led_state);
        let command = match led_state {
            LedState::Ready => "set ready led",
            LedState::Set => "set set led",
            LedState::Go => "set go led",
            LedState::AllOn => "set all leds",
            LedState::AllOff => "clear all leds",
        };
        let message = format!("{}\n", command);
        self.send_message_to_controller(id, message)?;
        Ok(())
    }

    pub fn get_controller_ids(&self) -> Vec<u32> {
        self.controllers.iter().map(|s| s.id).collect()
    }

    pub fn connect_new_controller(&mut self) -> Result<Option<u32>, ModuleError> {
        // Check for new controllers
        match list_ports() {
            Ok(ports) => {
                let ids = self.get_controller_ids();
                let regex = Regex::new(r"^/dev/ttyACM(\d+)$").unwrap();
                let valid_ports: Vec<u32> = ports
                    .iter()
                    .filter_map(|s| {
                        regex.captures(s).and_then(|caps| {
                            caps.get(1).and_then(|m| m.as_str().parse::<u32>().ok())
                        })
                    })
                    .collect();
                for port in valid_ports {
                    if !ids.contains(&port) {
                        let serial_string = format!("/dev/ttyACM{}", port);
                        match self.connect_controller(&serial_string) {
                            Ok(()) => return Ok(Some(port)),
                            Err(e) => return Err(ModuleError::ControllerError(e)),
                        }
                    }
                }
                Ok(None)
            }
            Err(e) => Err(ModuleError::SerialError(e)),
        }
    }
}

// TODO: Implement Drop for ControllerManager to ensure proper cleanup
// impl Drop for ControllerManager {
//     fn drop(&mut self) {
//         // Implement cleanup logic here
//     }
// }

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
        let real_port = "/dev/cu.usbmodem101"; // Example for macOS

        // Connect to the real controller
        manager.connect_controller(real_port).unwrap();

        // Give more time for the connection to establish
        thread::sleep(Duration::from_millis(1000));

        // Assert that the controller was added
        println!("{:?}", manager.controllers);
        assert_eq!(manager.controllers.len(), 1);

        // Check the properties of the connected controller
        let controller = &manager.controllers[0];
        assert_eq!(controller.serial_port, real_port);
        assert_eq!(controller.id, 0);

        // Switch the controller into run mode
        manager.set_controller_led(0, LedState::AllOn).unwrap();

        thread::sleep(Duration::from_millis(10));

        manager
            .send_message_to_controller(0, String::from("start controller\n"))
            .unwrap();

        // Give some time for the commands to take effect
        thread::sleep(Duration::from_millis(1000));

        // Wait for some data from the controller and update its state
        let mut state_updated = false;
        for _ in 0..50 {
            // Try for 5 seconds (50 * 100ms)
            manager.update_controller_state();
            if let Some(state) = manager.get_controller_state(0) {
                log::info!("Updated controller state: {:?}", state);
                state_updated = true;
            }
            thread::sleep(Duration::from_millis(100));
        }
        assert!(
            state_updated,
            "Controller state not updated after 5 seconds"
        );

        // Verify that we can get the controller state
        let final_state = manager.get_controller_state(0);
        assert!(final_state.is_some(), "Failed to get controller state");
        log::info!("Final controller state: {:?}", final_state.unwrap());

        // Clean up
        manager.disconnect_controller(0);
    }
}
