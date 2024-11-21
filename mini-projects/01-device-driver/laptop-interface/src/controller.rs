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

use log::{self, trace};
use regex::Regex;
use std::collections::HashMap;

pub struct ControllerManager {
    controllers: Vec<Controller>,
    input_receiver: Receiver<(u32, String)>,
    output_sender: Arc<Mutex<Sender<(u32, String)>>>,
    controller_senders: HashMap<u32, Sender<String>>, // Changed to HashMap
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
            controller_senders: HashMap::new(), // Initialize as HashMap
        }
    }

    pub fn connect_controller(&mut self, serial_port: &str) -> Result<(), ControllerError> {
        let id = self.extract_id_from_serial_port(serial_port)?;

        log::info!(
            "Connecting to controller on serial port: {}, assigning id {}",
            serial_port,
            id
        );
        let controller = Controller {
            name: format!("Controller_{}", serial_port.replace("/", "_")),
            id,
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
        self.controller_senders.insert(controller.id, tx.clone()); // Insert into HashMap

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
                    Err(e) => match e {
                        SerialError::Timeout => {
                            // Timeout is normal, just continue the loop
                            log::trace!("Read timeout, continuing");
                            thread::sleep(Duration::from_millis(100));
                        }
                        _ => {
                            log::error!("Error reading from serial port: {:?}", e);
                            return Err(e.into());
                        }
                    },
                }
            }
            Ok(())
        });

        log::info!("Controller connected successfully");
        Ok(())
    }

    fn send_message_to_controller(&self, id: u32, message: String) -> Result<(), ControllerError> {
        match self.controller_senders.get(&id) {
            Some(sender) => {
                sender.send(message)?;
                log::debug!("Message sent to controller {} successfully", id);
                Ok(())
            }
            None => {
                log::warn!("Controller sender for {} not found", id);
                Err(ControllerError::ControllerNotFound)
            }
        }
    }

    pub fn disconnect_controller(&mut self, id: u32) {
        if let Err(e) = self.send_message_to_controller(id, String::from("stop controller\n")) {
            log::error!("Failed to send reset message to controller {}: {}", id, e);
        }
        log::info!("Disconnecting controller with id: {}", id);
        if self.controller_senders.remove(&id).is_some() {
            log::info!("Controller with id: {} disconnected successfully", id);
        } else {
            log::warn!("No controller found with id: {}", id);
        }
        self.controllers.retain(|controller| controller.id != id);
    }

    fn read_serial(&self) -> Result<Option<(u32, String)>, ControllerError> {
        println!("Reading serial data");
        match self.input_receiver.try_recv() {
            Ok(data) => {
                println!("Read serial data (Controller {}): {}", data.0, data.1);
                Ok(Some(data))
            }
            Err(TryRecvError::Empty) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn extract_id_from_serial_port(&self, serial_port: &str) -> Result<u32, ControllerError> {
        let re = Regex::new(r"(\d+)$").unwrap();
        if let Some(captures) = re.captures(serial_port) {
            if let Some(matched) = captures.get(1) {
                if let Ok(id) = matched.as_str().parse::<u32>() {
                    return Ok(id);
                }
            }
        }
        Err(ControllerError::CommandError(format!(
            "Failed to extract ID from serial port: {}",
            serial_port
        )))
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

    pub fn start_controller(&self, id: u32) -> Result<(), ControllerError> {
        log::info!("Starting controller with id: {}", id);
        self.send_message_to_controller(id, String::from("start controller\n"))
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
                //info!("Found ports: {:?}", ports);
                let ids = self.get_controller_ids();
                let regex = Regex::new(r"^/dev/(?:ttyACM|cu\.usbmodem)(\d+)$").unwrap();
                let valid_ports: Vec<(String, u32)> = ports
                    .iter()
                    .filter_map(|s| {
                        regex.captures(s).and_then(|caps| {
                            caps.get(1).and_then(|m| {
                                m.as_str()
                                    .parse::<u32>()
                                    .ok()
                                    .map(|num| (s.to_string(), num))
                            })
                        })
                    })
                    .collect();
                //println!("{:?}", valid_ports);
                for (serial_string, port) in valid_ports {
                    trace!("Found valid port: {}", serial_string);
                    if !ids.contains(&port) {
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
// ... existing code ...
