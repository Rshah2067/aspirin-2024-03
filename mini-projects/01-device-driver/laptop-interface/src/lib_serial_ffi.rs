#![allow(non_camel_case_types, dead_code)]
use log::{debug, error, info, trace, warn};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::ptr;
use std::time::{Duration, Instant};
use libc::{size_t};

use crate::error::SerialError;

#[repr(C)]
struct sp_port {
    _private: [u8; 0], // Opaque struct
}

pub struct SerialPort {
    sp_port: *mut sp_port,
}

impl SerialPort {
    pub fn new(name: CString) -> Result<Self, SerialError> {
        debug!("Creating new SerialPort with name: {:?}", name);
        let mut port: *mut sp_port = ptr::null_mut();
        let result: c_int;
        unsafe {
            result = sp_get_port_by_name(name.as_ptr(), &mut port);
        }
        trace!("sp_get_port_by_name returned: {}, port pointer: {:?}", result, port);
        if port.is_null() {
            error!("Failed to create port: null pointer");
            return Err(SerialError::MEM);
        }
        match result {
            0 => {
                info!("Successfully created SerialPort");
                Ok(SerialPort { sp_port: port })
            }
            -1 => {
                error!("Failed to create port: invalid argument");
                Err(SerialError::ARG)
            }
            -2 => {
                error!("Failed to create port: operation failed");
                Err(SerialError::FAIL)
            }
            -3 => {
                error!("Failed to create port: memory allocation error");
                Err(SerialError::MEM)
            }
            -4 => {
                error!("Failed to create port: operation not supported");
                Err(SerialError::SUPP)
            }
            _ => {
                error!("Failed to create port: unknown error");
                Err(SerialError::FAIL)
            }
        }
    }

    pub fn open(&self, mode: sp_mode) -> Result<(), SerialError> {
        debug!("Opening serial port with mode: {:?}", mode);
        let result: c_int;
        unsafe {
            result = sp_open(self.sp_port, mode);
        }
        trace!("sp_open returned: {}", result);
        match result {
            0 => {
                info!("Successfully opened SerialPort");
                Ok(())
            }
            -1 => {
                error!("Failed to open port: invalid argument");
                Err(SerialError::ARG)
            }
            -2 => {
                error!("Failed to open port: operation failed");
                Err(SerialError::FAIL)
            }
            -3 => {
                error!("Failed to open port: memory allocation error");
                Err(SerialError::MEM)
            }
            -4 => {
                error!("Failed to open port: operation not supported");
                Err(SerialError::SUPP)
            }
            _ => {
                error!("Failed to open port: unknown error");
                Err(SerialError::FAIL)
            }
        }
    }

    pub fn configure(&self, baudrate: c_int, bits: c_int, flowcontrol: sp_flowcontrol) -> Result<(), SerialError> {
        debug!(
            "Configuring SerialPort with baudrate: {}, bits: {}, flowcontrol: {:?}",
            baudrate, bits, flowcontrol
        );
        unsafe {
            if sp_set_baudrate(self.sp_port, baudrate) != 0 {
                error!("Failed to set baudrate to {}", baudrate);
                return Err(SerialError::CONFIG_BAUDRATE);
            }
            if sp_set_bits(self.sp_port, bits) != 0 {
                error!("Failed to set bits to {}", bits);
                return Err(SerialError::CONFIG_BITS);
            }
            if sp_set_flowcontrol(self.sp_port, flowcontrol) != 0 {
                error!("Failed to set flow control to {:?}", flowcontrol);
                return Err(SerialError::CONFIG_FLOWCONTROL);
            }
        }
        info!("SerialPort configured successfully");
        Ok(())
    }

    pub fn read(&self, buff: &mut [u8], timeout_ms: u32) -> Result<usize, SerialError> {
        debug!(
            "Attempting to read {} bytes with {}ms timeout",
            buff.len(),
            timeout_ms
        );

        trace!(
            "Calling sp_blocking_read with buffer size: {}, timeout_ms: {}",
            buff.len(),
            timeout_ms
        );
        let result = unsafe {
            sp_blocking_read(
                self.sp_port,
                buff.as_mut_ptr() as *mut c_void,
                buff.len(),
                timeout_ms as c_uint,
            )
        };
        trace!("sp_blocking_read returned: {}", result);

        if result > 0 {
            debug!("Successfully read {} bytes", result);
            Ok(result as usize)
        } else if result == 0 {
            warn!("No data read within the timeout period");
            Err(SerialError::Timeout)
        } else {
            let err_code = result as c_int;
            error!("Failed to read: error code {}", err_code);
            Err(SerialError::from(err_code))
        }
    }

    pub fn write(&self, message: &str) -> Result<(), SerialError> {
        let buf = message.as_bytes();
        let count = buf.len();
        debug!("Attempting to write {} bytes", count);

        trace!(
            "Calling sp_blocking_write with buffer size: {}, timeout_ms: 2000",
            count
        );
        let result = unsafe {
            sp_blocking_write(
                self.sp_port,
                buf.as_ptr() as *const c_void,
                count,
                2000, // Timeout in milliseconds
            )
        };
        trace!("sp_blocking_write returned: {}", result);

        if result >= 0 {
            info!("Successfully wrote {} bytes", result);
            Ok(())
        } else {
            let err_code = result as c_int;
            error!("Failed to write: error code {}", err_code);
            Err(SerialError::from(err_code))
        }
    }
}

// Implement Drop to clean up resources
impl Drop for SerialPort {
    fn drop(&mut self) {
        trace!("Dropping SerialPort");
        unsafe {
            sp_close(self.sp_port);
            sp_free_port(self.sp_port);
        }
        info!("SerialPort resources freed");
    }
}

// Enums and FFI function declarations remain the same
// Enums and FFI function declarations

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum sp_mode {
    SP_MODE_READ = 1,
    SP_MODE_WRITE = 2,
    SP_MODE_READ_WRITE = 3,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum sp_flowcontrol {
    SP_FLOWCONTROL_NONE = 0,
    SP_FLOWCONTROL_XONXOFF = 1,
    SP_FLOWCONTROL_RTSCTS = 2,
    SP_FLOWCONTROL_DTRDSR = 3,
}

#[link(name = "serialport")]
extern "C" {
    fn sp_get_port_name(port: *mut sp_port) -> *const c_char;
    fn sp_list_ports(list: *mut *mut *mut sp_port) -> c_int;
    fn sp_get_port_by_name(name: *const c_char, port: *mut *mut sp_port) -> c_int;
    fn sp_open(port: *mut sp_port, mode: sp_mode) -> c_int;
    fn sp_close(port: *mut sp_port) -> c_int;
    fn sp_set_baudrate(port: *mut sp_port, baudrate: c_int) -> c_int;
    fn sp_set_bits(port: *mut sp_port, bits: c_int) -> c_int;
    fn sp_set_flowcontrol(port: *mut sp_port, control: sp_flowcontrol) -> c_int;
    fn sp_blocking_write(
        port: *mut sp_port,
        buf: *const c_void,
        count: size_t,
        timeout_ms: c_uint,
    ) -> isize;
    fn sp_blocking_read(
        port: *mut sp_port,
        buf: *mut c_void,
        count: size_t,
        timeout_ms: c_uint,
    ) -> isize;
    fn sp_free_port_list(ports: *mut *mut sp_port);
    fn sp_free_port(port: *mut sp_port);
}

pub fn list_ports() -> Result<Vec<String>, SerialError> {
    debug!("Listing available serial ports.");

    // Allocate space for the port list pointer
    let mut port_list: *mut *mut sp_port = ptr::null_mut();

    // Call the C function to populate the port list
    let result: c_int;
    unsafe {
        result = sp_list_ports(&mut port_list);
    }

    trace!("sp_list_ports returned: {}", result);

    if result != 0 {
        // Handle different error codes
        return match result {
            -1 => {
                error!("Failed to list ports: invalid argument");
                Err(SerialError::ARG)
            },
            -2 => {
                error!("Failed to list ports: operation failed");
                Err(SerialError::FAIL)
            },
            -3 => {
                error!("Failed to list ports: memory allocation error");
                Err(SerialError::MEM)
            },
            -4 => {
                error!("Failed to list ports: operation not supported");
                Err(SerialError::SUPP)
            },
            _ => {
                error!("Failed to list ports: unknown error");
                Err(SerialError::FAIL)
            },
        };
    }

    // If port_list is null, return an empty vector
    if port_list.is_null() {
        warn!("sp_list_ports returned a null pointer.");
        return Ok(Vec::new());
    }

    let mut ports = Vec::new();
    let mut i = 0;

    unsafe {
        // Iterate until we find a null pointer indicating the end of the list
        while !(*port_list.add(i)).is_null() {
            let port = *port_list.add(i);
            let name_ptr = sp_get_port_name(port);
            if name_ptr.is_null() {
                warn!("Port {} has a null name pointer.", i);
                ports.push(format!("Unknown Port {}", i));
            } else {
                // Safely convert C string to Rust String
                let name = CStr::from_ptr(name_ptr).to_string_lossy().into_owned();
                trace!("Found port {}: {}", i, name);
                ports.push(name);
            }
            i += 1;
        }

        // Free the allocated port list
        sp_free_port_list(port_list);
    }

    info!("Successfully listed {} serial ports.", ports.len());
    Ok(ports)
}
