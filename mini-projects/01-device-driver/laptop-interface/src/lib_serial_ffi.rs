#![allow(non_camel_case_types, dead_code)]
use log::{debug, error, info, trace, warn};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::ptr;
use std::time::{Duration, Instant};

use crate::error::SerialError;

#[repr(C)]
struct sp_port {
    _private: [u8; 0], // Opaque struct
}

// Wrapper for struct that allows us to pass it through functions
pub struct SerialPort {
    sp_port: *mut sp_port,
}

impl SerialPort {
    pub fn new(name: CString) -> Result<Self, SerialError> {
        let mut port: *mut sp_port = ptr::null_mut();
        let result: c_int;
        unsafe {
            result = sp_get_port_by_name(name.as_ptr(), &mut port);
        }
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
        let result: c_int;
        unsafe {
            result = sp_open(self.sp_port, mode);
        }
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

    pub fn read(&self, buff: &mut [u8], timeout_ms: u32) -> Result<usize, SerialError> {
        let start_time = Instant::now();
        let timeout = Duration::from_millis(timeout_ms as u64);

        debug!(
            "Attempting to read {} bytes with {}ms timeout",
            buff.len(),
            timeout_ms
        );

        let mut total_bytes_read: usize = 0;

        while total_bytes_read < buff.len() && start_time.elapsed() < timeout {
            let remaining_timeout = timeout
                .checked_sub(start_time.elapsed())
                .unwrap_or_else(|| Duration::from_millis(0));

            let result = unsafe {
                sp_blocking_read(
                    self.sp_port,
                    buff[total_bytes_read..].as_mut_ptr() as *mut c_void,
                    buff.len() - total_bytes_read,
                    remaining_timeout.as_millis() as c_uint,
                )
            };

            if result > 0 {
                total_bytes_read += result as usize;
                trace!("Read {} bytes", result);
            } else if result == 0 {
                trace!("No data available, continuing to wait");
                std::thread::sleep(Duration::from_millis(10));
            } else {
                // Handle error based on negative result
                let err_code = result as c_int;
                error!("Failed to read: error code {}", err_code);
                return Err(SerialError::from(err_code));
            }
        }

        if total_bytes_read > 0 {
            info!("Successfully read {} bytes", total_bytes_read);
            Ok(total_bytes_read)
        } else {
            warn!("No data read within the timeout period");
            Err(SerialError::Timeout)
        }
    }

    pub fn write(&self, message: &str) -> Result<(), SerialError> {
        let buf = message.as_bytes();
        let count = buf.len();
        debug!("Attempting to write {} bytes", count);

        let result = unsafe {
            sp_blocking_write(
                self.sp_port,
                buf.as_ptr() as *const c_void,
                count,
                2000, // Timeout in milliseconds
            )
        };

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
    }
}

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
        count: usize,
        timeout_ms: c_uint,
    ) -> isize;
    fn sp_blocking_read(
        port: *mut sp_port,
        buf: *mut c_void,
        count: usize,
        timeout_ms: c_uint,
    ) -> isize;
    fn sp_free_port_list(ports: *mut *mut sp_port);
    fn sp_free_port(port: *mut sp_port);
}

pub fn list_ports() -> Result<Vec<String>, SerialError> {
    let mut port_list: *mut *mut sp_port = ptr::null_mut();
    let mut ports = Vec::new();
    let result: c_int;
    unsafe {
        result = sp_list_ports(&mut port_list);
    }
    match result {
        0 => {
            let mut i = 0;
            unsafe {
                while !(*port_list.add(i)).is_null() {
                    let port = *port_list.add(i);
                    let name_ptr = sp_get_port_name(port);
                    let name = CStr::from_ptr(name_ptr).to_string_lossy().into_owned();
                    trace!("Found port: {}", name);
                    ports.push(name);
                    i += 1;
                }
                sp_free_port_list(port_list);
            }
            trace!("Successfully listed {} ports", ports.len());
            Ok(ports)
        }
        -1 => {
            error!("Failed to list ports: invalid argument");
            Err(SerialError::ARG)
        }
        -2 => {
            error!("Failed to list ports: operation failed");
            Err(SerialError::FAIL)
        }
        -3 => {
            error!("Failed to list ports: memory allocation error");
            Err(SerialError::MEM)
        }
        -4 => {
            error!("Failed to list ports: operation not supported");
            Err(SerialError::SUPP)
        }
        _ => {
            error!("Failed to list ports: unknown error");
            Err(SerialError::FAIL)
        }
    }
}
