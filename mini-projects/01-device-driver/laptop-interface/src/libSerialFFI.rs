use std::ffi::c_void;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

use crate::error::SerialError;

#[repr(C)]
struct sp_port{
    port:[u8;0] //Creating an Opaque Struct that has nothing in ti
}
//wrapper for struct that allows us to pass it through functions
pub struct SerialPort{
    sp_port:*mut sp_port
}
impl SerialPort{
    //creates a new port object and checks to make sure that there is actually a port with this 
    //name
    pub fn new(name:CString)->Result<Self,SerialError>{
        let mut port:*mut sp_port = ptr::null_mut();
        let result:sp_return;
        unsafe{
            result = sp_get_port_by_name(name.as_ptr(), port);
        }
        match result{
            //Success
            sp_return::SP_OK =>{
                Ok(SerialPort{sp_port:port})
            },
            //Any Errror
            _ =>{
                Err(SerialError::FailedToFind)
            }
        }
    }
    
}
//enum representing failure cases of function calls
#[repr(C)]
enum sp_return {
    SP_OK,
    SP_ERR_ARG,	
    SP_ERR_FAIL,	
    SP_ERR_MEM,
    SP_ERR_SUPP,
}
#[repr(C)]
enum sp_mode{
    SP_MODE_READ,
    SP_MODE_WRITE,
    SP_MODE_READ_WRITE, 
}
#[repr(C)]
enum sp_flowcontrol{
    SP_FLOWCONTROL_NONE,
    SP_FLOWCONTROL_XONXOFF,
    SP_FLOWCONTROL_RTSCTS,
    SP_FLOWCONTROL_DTRDSR,
}

#[link(name ="serialport")]
extern "C"{
    fn sp_get_port_name(port:*mut sp_port)->*mut c_char;
    fn sp_list_ports(list:*mut *mut *mut sp_port) ->sp_return;
    fn sp_get_port_by_name(PORT_NAMEL:*const c_char, port:*mut sp_port) ->sp_return;
    fn sp_open(sp_port:*mut sp_port,mode:sp_mode)->sp_return;
    fn sp_set_baudrate(sp_port:*mut sp_port,buadrate:usize) ->sp_return;
    fn sp_set_bits(sp_port:*mut sp_port,bit:usize) ->sp_return;
    fn sp_set_flow_control(sp_port:*mut sp_port,control:sp_flowcontrol) ->sp_return;
    fn sp_non_blocking_write(sp_port:*mut sp_port,buf:*const c_char,count:u16) ->sp_return;
    fn sp_non_blocking_read(sp_port:*mut sp_port,buf:*const c_char,count:u16) ->sp_return;
}
pub fn list_ports()->Result<Vec<String>,SerialError>{
    let port_list:*mut *mut *mut sp_port = ptr::null_mut();
    let mut ports = Vec::new();
    let result:sp_return;
    unsafe {
        result = sp_list_ports(port_list);
    }
    match result{
        sp_return::SP_OK =>{
            let mut i = 0;
            unsafe{
                while !(*port_list.add(i)).is_null() {
                    let port = **port_list.add(i);
                    let name_ptr = sp_get_port_name(port);
                    let name = CStr::from_ptr(name_ptr).to_string_lossy();
                    ports.push(name);
                };
            };
            todo!()
        },
        _ =>Err(SerialError::FailedToList)
    }
}