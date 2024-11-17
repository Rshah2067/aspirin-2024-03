use std::ffi::c_void;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::ptr;

use crate::error::SerialError;

#[repr(C)]
struct sp_port {
    port: [u8; 0], //Creating an Opaque Struct that has nothing in ti
}
//wrapper for struct that allows us to pass it through functions
#[allow(dead_code)]
pub struct SerialPort {
    sp_port: *mut sp_port,
}
//%TODO implement Drop, refactor sp_return to usize
impl SerialPort {
    //creates a new port object and checks to make sure that there is actually a port with this
    //name
    pub fn new(name: CString) -> Result<Self, SerialError> {
        let mut port: *mut sp_port = ptr::null_mut();
        let result: sp_return;
        unsafe {
            result = sp_get_port_by_name(name.as_ptr(), &mut port);
        }
        //Check to see if the port is null due to there not being one with given name
        if port.is_null() {
            return Err(SerialError::MEM);
        }
        match result {
            //Success
            sp_return::SP_OK => Ok(SerialPort { sp_port: port }),
            //Any Errror
            sp_return::SP_ERR_ARG => Err(SerialError::ARG),
            sp_return::SP_ERR_FAIL => Err(SerialError::FAIL),
            sp_return::SP_ERR_MEM => Err(SerialError::MEM),
            sp_return::SP_ERR_SUPP => Err(SerialError::SUPP),
        }
    }
    pub fn open(&self, mode: sp_mode) -> Result<(), SerialError> {
        let result: sp_return;
        unsafe {
            result = sp_open(self.sp_port, mode);
        }
        match result {
            sp_return::SP_OK => Ok(()),
            sp_return::SP_ERR_ARG => Err(SerialError::ARG),
            sp_return::SP_ERR_FAIL => Err(SerialError::FAIL),
            sp_return::SP_ERR_MEM => Err(SerialError::MEM),
            sp_return::SP_ERR_SUPP => Err(SerialError::SUPP),
        }
    }
    //Pass in a mutable reference to an array and the number of bits you want to read
    pub fn read<'a>(&'a self, buff: &'a mut [u8], count: u16) -> Result<String, SerialError> {
        let result: sp_return;
        println!("0");
        unsafe {
            result = sp_blocking_read(self.sp_port, buff.as_mut_ptr() as *mut c_char, count, 2);
            println!("1");
        }

        //check to see if we have successfully read
        match result {
            sp_return::SP_ERR_ARG => Err(SerialError::ARG),
            sp_return::SP_ERR_FAIL => Err(SerialError::FAIL),
            sp_return::SP_ERR_MEM => Err(SerialError::MEM),
            sp_return::SP_ERR_SUPP => Err(SerialError::SUPP),
            _ => Ok(String::from_utf8_lossy(buff).to_string()),
        }
    }
    pub fn write(&self, message: String) -> Result<(), SerialError> {
        let buf: &[u8] = message.as_bytes();
        let count = buf.len();
        let result;
        unsafe {
            //convert the u8 to a i8
            let signedbuf: &[i8] = std::slice::from_raw_parts(buf.as_ptr() as *const i8, count);
            result = sp_blocking_write(self.sp_port, signedbuf.as_ptr(), count as u16, 2);
        }
        match result {
            sp_return::SP_ERR_ARG => Err(SerialError::ARG),
            sp_return::SP_ERR_FAIL => Err(SerialError::FAIL),
            sp_return::SP_ERR_MEM => Err(SerialError::MEM),
            sp_return::SP_ERR_SUPP => Err(SerialError::SUPP),
            _ => Ok(()),
        }
    }
    pub fn Drop() {}
}
//enum representing failure cases of function calls
#[repr(C)]
enum sp_return {
    SP_OK = 0,
    SP_ERR_ARG = -1,
    SP_ERR_FAIL = -2,
    SP_ERR_MEM = -3,
    SP_ERR_SUPP = -4,
}
#[repr(C)]
pub enum sp_mode {
    SP_MODE_READ,
    SP_MODE_WRITE,
    SP_MODE_READ_WRITE,
}
#[repr(C)]
enum sp_flowcontrol {
    SP_FLOWCONTROL_NONE,
    SP_FLOWCONTROL_XONXOFF,
    SP_FLOWCONTROL_RTSCTS,
    SP_FLOWCONTROL_DTRDSR,
}

#[link(name = "serialport")]
extern "C" {
    fn sp_get_port_name(port: *mut sp_port) -> *mut c_char;
    fn sp_list_ports(list: *mut *mut *mut sp_port) -> sp_return;
    fn sp_get_port_by_name(PORT_NAMEL: *const c_char, port: *mut *mut sp_port) -> sp_return;
    fn sp_open(sp_port: *mut sp_port, mode: sp_mode) -> sp_return;
    fn sp_set_baudrate(sp_port: *mut sp_port, buadrate: usize) -> sp_return;
    fn sp_set_bits(sp_port: *mut sp_port, bit: usize) -> sp_return;
    fn sp_set_flow_control(sp_port: *mut sp_port, control: sp_flowcontrol) -> sp_return;
    fn sp_blocking_write(
        sp_port: *mut sp_port,
        buf: *const c_char,
        count: u16,
        timout: usize,
    ) -> sp_return;
    fn sp_blocking_read(
        sp_port: *mut sp_port,
        buf: *mut c_char,
        count: u16,
        timeout: usize,
    ) -> sp_return;
    fn sp_free_port_list(ports: *mut *mut sp_port);
    fn sp_free_port(port: *mut sp_port);

}
pub fn list_ports() -> Result<Vec<String>, SerialError> {
    let mut port_list: *mut *mut sp_port = ptr::null_mut();
    let mut ports = Vec::new();
    let result: sp_return;
    unsafe {
        result = sp_list_ports(&mut port_list);
    }
    match result {
        sp_return::SP_OK => {
            let mut i = 0;
            unsafe {
                while !(*port_list.add(i)).is_null() {
                    let port = *port_list.add(i);
                    let name_ptr = sp_get_port_name(port);
                    let name = CStr::from_ptr(name_ptr).to_string_lossy().into_owned();
                    ports.push(name);
                    i += 1;
                }
                sp_free_port_list(port_list);
            };
            Ok(ports)
        }
        sp_return::SP_ERR_ARG => Err(SerialError::ARG),
        sp_return::SP_ERR_FAIL => Err(SerialError::FAIL),
        sp_return::SP_ERR_MEM => Err(SerialError::MEM),
        sp_return::SP_ERR_SUPP => Err(SerialError::SUPP),
    }
}
