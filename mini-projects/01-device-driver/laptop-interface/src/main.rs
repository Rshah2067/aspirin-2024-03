#![allow(dead_code)]
mod lib_serial_ffi;
use lib_serial_ffi::*;
use std::ffi::CString;
mod error;

mod controller;

fn main() {
    match list_ports() {
        Ok(list) => println!("{:?}", list),
        Err(e) => eprint!("Failed to List Ports: {}", e),
    }
    let port = SerialPort::new(CString::from(c"/dev/cu.usbmodem101"));
    if let Ok(serialport) = port {
        match serialport.open(sp_mode::SP_MODE_READ_WRITE) {
            Ok(_) => {
                let _ = serialport.write("init controller");
                let _ = serialport.write("set ready led");
            }
            Err(e) => eprint!("Failed to open Port{}", e),
        }
    } else {
        let _ = port.inspect_err(|e| eprint!("{}", e));
    }
}
