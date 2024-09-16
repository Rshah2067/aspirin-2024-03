use core::num;
use std::io;
enum operation{
    AND,
    OR,
    XOR,
}
enum inputformat{
    binary,
    numeric,
    hex,
    invalid,
}
struct Calculator{
    op:operation,
    inputform:inputformat,
    output: u32,
    input1:u32,
    input2:u32,
}
//Take input and parse string for input put

fn run_calculator(){
    //ask for input
    ask_for_input(1);
    //parse input to string containing value and determine input type
    //convert to decimel
    //preform operation
    //return input
}
fn ask_for_input(input_number: u32 ){
    match input_number{
        1 =>println!("Please enter first number:"),
        2 =>println!("Please enter second number:"),
        _ =>println!("undefined"),
    }
}
fn store_input() -> String{
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input
}
fn determine_input_type(input:&String) -> &str{
    //Checking if input is a base-10 number, if it is return string and set input type to numeric
    match input.trim().parse::<u32>() {
        Ok(_) => {
            inputformat::numeric;
            &input
        },
        //if it is not a numeric we now want to try to see if it contains a 0b or 0x indicating it is a 
        Err(_) => {
            let prefix = &input[0..2];
            match prefix {
                "0x" =>{
                    inputformat::hex;
                    &input[2..]
                }
                "0b" =>{
                    inputformat::binary;
                    &input[2..]
                }
                _ =>{
                    inputformat::invalid;
                    &input
                }
            }
        },
    }
}
fn convert_input_type(input:&str,calc:Calculator)->u32{
    let mut num = 0;
    match calc.inputform{
        inputformat::binary => {
           match u32::from_str_radix(input,2) {
                Ok(num) =>num,
                Err(_) => {
                    inputformat::invalid;
                    num
                },
            }
        },
        inputformat::hex =>{
            match u32::from_str_radix(input,16) {
                Ok(num) =>num,
                Err(_) => {
                    inputformat::invalid;
                    num
                },
            }
        },
        inputformat::numeric =>{
            match input.parse::<u32>(){
                Ok(num) =>num,
                Err(_) =>{
                    inputformat::invalid;
                    num
                }

            }
        },
        inputformat::invalid =>,
    }
}