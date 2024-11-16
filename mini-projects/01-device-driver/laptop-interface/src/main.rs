
mod libSerialFFI;
use std::ffi::CString;
use libSerialFFI::*;
mod error;
fn main() {
    match list_ports(){
        Ok(list)=>println!("{:?}",list),
        Err(e) =>eprint!("Failed to List Ports: {}",e)
    }
    let port = SerialPort::new(CString::from(c"/dev/ttyACM1"));
    if let Ok(serialport) = port{
        match serialport.open(sp_mode::SP_MODE_READ_WRITE){
            Ok(_) =>{
                //now try to
                let mut buff:[u8;1024] = [0;1024];
                match serialport.read(&mut buff, 100){
                    Ok(_) =>{
                        println!("2");
                        println!("{:?}",buff)
                    },
                    Err(e) =>eprint!("Failed to Read{}",e)
                }
            },
            Err(e) =>eprint!("Failed to open Port{}",e)
        }
    }
    else{
        port.inspect_err(|e| eprint!("{}",e));
    }

}

