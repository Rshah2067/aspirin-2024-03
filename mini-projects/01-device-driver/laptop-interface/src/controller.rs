#![allow(dead_code)]

use crate::error::{ControllerError, ModuleError, SerialError};
use crate::lib_serial_ffi::*;

use std::ffi::CString;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::{default, str};

use log::{self, error, info, trace};
use regex::Regex;
use std::collections::HashMap;

pub struct ControllerManager {
    controllers: Vec<Controller>,
    input_receiver: Receiver<(u32, String)>,
    output_sender: Sender<(u32, String)>,
    pub join_handles: HashMap<u32, Option<JoinHandle<Result<(), ControllerError>>>>,
}

#[derive(Clone, Debug)]
pub struct ControllerState {
    pub north_east: bool,
    pub north_west: bool,
    pub south_east: bool,
    pub south_west: bool,
    pub north: Option<bool>,
    pub south: Option<bool>,
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
            north: Some((bitmask & 0b1000) != 0), // Optional buttons we added
            south: Some((bitmask & 0b1000) != 0),
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
            north: Some(false),
            south: Some(false),
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

#[derive(Debug)]
struct Controller {
    name: String,
    id: u32,
    serial_port: String,
    kind: ControllerKind,
    state: ControllerState,

    sender: Sender<String>, // Moved sender into Controller struct
}

impl ControllerManager {
    pub fn new() -> ControllerManager {
        let (output_sender, output_receiver) = std::sync::mpsc::channel();
        ControllerManager {
            controllers: Vec::new(),
            input_receiver: output_receiver,
            output_sender,
            join_handles: HashMap::new(),
        }
    }

    pub fn connect_controller(&mut self, serial_port: &str) -> Result<u32, ControllerError> {
        let id = self.extract_id_from_serial_port(serial_port)?;

        log::info!(
            "Connecting to controller on serial port: {}, assigning id {}",
            serial_port,
            id
        );

        let (tx, rx): (Sender<String>, Receiver<String>) = std::sync::mpsc::channel();

        let controller = Controller {
            name: format!("Controller_{}", serial_port.replace("/", "_")),
            id,
            serial_port: serial_port.to_string(),
            kind: ControllerManager::determine_controller_kind(serial_port),
            state: ControllerState::default(),
            sender: tx.clone(),
        };

        let controller_id = controller.id;

        self.join_handles.insert(
            controller_id,
            Some(self.spawn_controller_thread(&controller, rx)?),
        );
        self.controllers.push(controller);
        self.stop_controller(controller_id)?;
        self.reset_controller(controller_id)?;
        self.init_controller(controller_id)?;
        self.set_controller_led(controller_id, LedState::Ready)?;

        log::info!("Controller connected successfully");
        Ok(controller_id)
    }

    fn spawn_controller_thread(
        &self,
        controller: &Controller,
        rx: Receiver<String>,
    ) -> Result<JoinHandle<Result<(), ControllerError>>, ControllerError> {
        let thread_sender = self.output_sender.clone();
        let controller_id = controller.id;
        let serial_port = controller.serial_port.clone();

        // Move the necessary data into the closure
        Ok(thread::spawn(move || -> Result<(), ControllerError> {
            log::trace!("Controller {}: Thread spawned.", controller_id);
            // Initialize the serial port
            let port = SerialPort::new(CString::new(serial_port.clone()).map_err(|e| {
                log::error!(
                    "Controller {}: Failed to convert serial port name to CString: {}",
                    controller_id,
                    e
                );
                ControllerError::SerialError(SerialError::Unknown)
            })?)?;
            log::trace!("Controller {}: SerialPort object created.", controller_id);

            // Open the port
            port.open(sp_mode::SP_MODE_READ_WRITE).map_err(|e| {
                log::error!(
                    "Controller {}: Failed to open serial port: {:?}",
                    controller_id,
                    e
                );
                ControllerError::SerialError(e)
            })?;
            log::trace!(
                "Controller {}: Serial port opened in read/write mode.",
                controller_id
            );

            // Configure the port
            port.configure(9600, 8, sp_flowcontrol::SP_FLOWCONTROL_NONE)
                .map_err(|e| {
                    log::error!(
                        "Controller {}: Failed to configure serial port: {:?}",
                        controller_id,
                        e
                    );
                    ControllerError::SerialError(e)
                })?;
            log::trace!(
                "Controller {}: Serial port configured successfully.",
                controller_id
            );

            log::trace!(
                "Serial port {} opened and configured successfully.",
                serial_port
            );

            let mut buffer = vec![0u8; 1024];
            log::trace!("Controller {}: Buffer initialized.", controller_id);

            log::trace!("Controller {}: Entering main loop.", controller_id);

            loop {
                log::trace!("Controller {}: Start of loop iteration.", controller_id);

                // Handle incoming messages
                match rx.try_recv() {
                    Ok(message) => {
                        log::debug!(
                            "Controller {}: Received message to send: {}",
                            controller_id,
                            message
                        );
                        if let Err(e) = port.write(&message) {
                            log::warn!(
                                "Controller {}: Failed to write to serial port: {:?}",
                                controller_id,
                                e
                            );
                        }
                        log::info!(
                            "Controller {}: Message processing completed.",
                            controller_id
                        );
                    }
                    Err(TryRecvError::Empty) => {
                        log::trace!("Controller {}: No incoming messages.", controller_id);
                    }
                    Err(TryRecvError::Disconnected) => {
                        log::error!(
                            "Controller {}: Output channel disconnected. Exiting thread.",
                            controller_id
                        );
                        log::info!(
                            "Controller {}: Breaking loop due to channel disconnection.",
                            controller_id
                        );
                        break;
                    }
                }
                log::trace!(
                    "Controller {}: Incoming message handling completed.",
                    controller_id
                );

                log::debug!(
                    "Controller {}: Attempting to read from serial port.",
                    controller_id
                );
                match port.read(&mut buffer, 100) {
                    Ok(bytes_read) => {
                        if bytes_read > 0 {
                            let received_data =
                                String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                            log::trace!(
                                "Controller {}: Received raw data: {:?}",
                                controller_id,
                                &buffer[..bytes_read]
                            );
                            if let Err(e) = thread_sender.send((controller_id, received_data)) {
                                log::error!(
                                    "Controller {}: Failed to send received data: {}",
                                    controller_id,
                                    e
                                );
                                log::info!(
                                    "Controller {}: Breaking loop due to send failure.",
                                    controller_id
                                );
                                // Break the loop if sending fails
                                break;
                            }
                            log::trace!(
                                "Controller {}: Received data processed and sent.",
                                controller_id
                            );
                        } else {
                            log::trace!(
                                "Controller {}: No data read from serial port.",
                                controller_id
                            );
                        }
                    }
                    Err(e) => match e {
                        SerialError::Timeout => {
                            log::debug!(
                                    "Controller {}: Read operation timed out (This is normal when there is no data to be read)",
                                    controller_id
                                );
                        }
                        _ => {
                            log::error!(
                                "Controller {}: Error reading from serial port: {:?}",
                                controller_id,
                                e
                            );
                            break;
                        }
                    },
                }
                log::trace!(
                    "Controller {}: Serial port read operation completed.",
                    controller_id
                );

                log::debug!("Controller {}: End of loop iteration.", controller_id);
                thread::sleep(Duration::from_millis(100));
            }

            log::warn!("Controller {}: Exiting thread loop.", controller_id);

            Ok(())
        }))
    }

    fn send_message_to_controller(&self, id: u32, message: String) -> Result<(), ControllerError> {
        if let Some(controller) = self.controllers.iter().find(|c| c.id == id) {
            controller.sender.send(message)?;
            log::debug!("Message sent to controller {} successfully", id);
            Ok(())
        } else {
            log::warn!("Controller with id {} not found", id);
            Err(ControllerError::ControllerNotFound)
        }
    }

    pub fn disconnect_controller(&mut self, id: u32) {
        if let Err(e) = self.send_message_to_controller(id, String::from("stop controller\n")) {
            log::error!("Failed to send reset message to controller {}: {}", id, e);
        }
        log::info!("Disconnecting controller with id: {}", id);
        // Optionally join the controller's thread here if necessary
        self.controllers.retain(|controller| controller.id != id);
    }

    fn read_serial(&self) -> Result<Option<(u32, String)>, ControllerError> {
        trace!("read_serial called on main thread");
        match self.input_receiver.try_recv() {
            Ok(data) => {
                info!(
                    "Read serial data from channel on main thread (Controller {}): {}",
                    data.0, data.1
                );
                Ok(Some(data))
            }
            Err(TryRecvError::Empty) => {
                info!("No serial data available from channel on main thread");
                Ok(None)
            }
            Err(e) => {
                error!("Error reading serial data from channel: {:?}", e);
                Err(e.into())
            }
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
        while let Ok(Some((id, data))) = self.read_serial() {
            // Process each line separately
            for line in data.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match u8::from_str_radix(trimmed, 16) {
                    Ok(bitmask) => {
                        if let Some(controller) = self.controllers.iter_mut().find(|c| c.id == id) {
                            controller.state = ControllerState::from_bitmask(bitmask);
                            log::debug!(
                                "Updated state for controller {}: {:?}",
                                id,
                                controller.state
                            );
                        } else {
                            log::warn!("Unknown controller ID: {}", id);
                        }
                    }
                    Err(e) => {
                        log::warn!(
                            "Failed to parse bitmask '{}' for controller {}: {}",
                            trimmed,
                            id,
                            e
                        );
                    }
                }
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
                for (serial_string, port) in valid_ports {
                    trace!("Found valid port: {}", serial_string);
                    if !ids.contains(&port) {
                        match self.connect_controller(&serial_string) {
                            Ok(_) => return Ok(Some(port)),
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
