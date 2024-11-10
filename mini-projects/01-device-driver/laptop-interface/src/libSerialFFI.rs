#![allow(non_snake_case)]
#![allow(non_camel_case_types)]

use std::os::raw::c_void;

struct SerialPort{
    BaudRate:BaudRate,
    CharacterSize:CharacterSize,
    Parity:
}
#[repr(C)]
enum BaudRate {
    BAUD_50, 	
    BAUD_75, 	
    BAUD_110, 	
    BAUD_134, 	
    BAUD_150, 	
    BAUD_200, 	
    BAUD_300, 	
    BAUD_600, 	
    BAUD_1200, 	
    BAUD_1800, 	
    BAUD_2400, 	
    BAUD_4800, 	
    BAUD_9600, 	
    BAUD_19200, 	
    BAUD_38400, 	
    BAUD_57600, 	
    BAUD_115200, 	
    BAUD_230400, 	
    BAUD_DEFAULT, 
}
#[repr(C)]
enum CharacterSize{
    CHAR_SIZE_5,
    CHAR_SIZE_6,
    CHAR_SIZE_7,
    CHAR_SIZE_8,
    CHAR_SIZE_DEFAULT
}
#[repr(C)]
enum StopBits { 
    STOP_BITS_1,
    STOP_BITS_2,
    STOP_BITS_DEFAULT, 
}
#[repr(C)]
enum Parity{
    PARITY_EVEN,
    PARITY_ODD,
    PARITY_NONE,
    PARITY_DEFAULT
}
#[repr(C)]
enum FlowControl {
    FLOW_CONTROL_HARD,
    FLOW_CONTROL_SOFT,
    FLOW_CONTROL_NONE,
    FLOW_CONTROL_DEFAULT
}

#[link(name ="libSerial")]
extern "C"{
    fn SetBaudRate();
    fn SetCharacterSize();
    fn SetParity();
    fn SetFlowControl();
    fn Write();
    fn Read();
}