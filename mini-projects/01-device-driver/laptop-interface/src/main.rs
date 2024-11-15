
mod libSerialFFI;
use std::ffi::CString;
use libSerialFFI::*;
mod error;
fn main() {
    match list_ports(){
        Ok(list)=>println!("{:?}",list),
        Err(e) =>eprint!("Failed to List Ports: {}",e)
    }
    let port = SerialPort::new(CString::from(c"/dev/ttyACM0"));
    if let Ok(serialport) = port{
        match serialport.open(sp_mode::SP_MODE_READ_WRITE){
            Ok(_) =>println!("Connected"),
            Err(e) =>eprint!("{}",e)
        }
    }
    else{
        port.inspect_err(|e| eprint!("{}",e));
    }
}

