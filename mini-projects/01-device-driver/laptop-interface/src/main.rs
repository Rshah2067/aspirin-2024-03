
mod libSerialFFI;
use std::ffi::CString;

use libSerialFFI::*;
mod error;
fn main() {
    match list_ports(){
        Ok(list)=>println!("{:?}",list),
        Err(e) =>eprint!("{}",e)
    }
    match SerialPort::new(CString::from(c"/dev/ttyACM0")){
        Ok(_) =>println!("Success!"),
        Err(e) =>eprint!("{}",e)
    }
}

