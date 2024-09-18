use core::num;
use std::io;
enum operation{
    AND,
    OR,
    XOR,
    invalid,
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
pub fn run() {
    println!("test");
    let mut calculator = Calculator {
        op:operation::invalid,
        inputform:inputformat::invalid,
        output: 0,
        input1: 0,
        input2: 0,
    };
    calculator = gather_all_inputs(calculator);
     preform_operation(calculator);

}
fn preform_operation(mut Calc:Calculator){
    match Calc.op {
        operation::OR => {
        Calc.output = Calc.input1 | Calc.input2;
        println!("The result of {} | {} is {}",Calc.input1,Calc.input2,Calc.output);
        },
        operation::XOR => {
            Calc.output = Calc.input1 ^ Calc.input2;
            println!("The result of {} | {} is {}",Calc.input1,Calc.input2,Calc.output);
        },
        operation::AND =>{
            Calc.output = Calc.input1 ^ Calc.input2;
            println!("The result of {} | {} is {}",Calc.input1,Calc.input2,Calc.output);
        } ,
        operation::invalid =>println!("Provide Valid Input"),
    }
    
}
//Gathering All inputs
fn gather_all_inputs(mut calc:Calculator) -> Calculator{
    //ask for input
    request_for_input(1);
    let mut input = store_input();
    let mut data = determine_input_type(&input);
    //check to see if user put in a valid input, if it does then parse the input for data
    match calc.inputform {
        inputformat::invalid =>{
            println!("invalid input, please try again");
            calc
        }
        _ =>{
            calc.input1 = convert_input_type(data,&calc);
            request_for_input(3);
            calc.op = determine_input_operation(&store_input());
            request_for_input(2);
            input = store_input();
            data = determine_input_type(&input);
            match calc.inputform {
                inputformat::invalid =>{
                    println!("invalid input, please try again");
                    calc
                }
                _=>{
                    calc.input2 = convert_input_type(data, &calc);
                    calc
                }
            }
        }
    }
}
fn request_for_input(input_number: u32 ){
    match input_number{
        1 =>println!("Please enter first number:"),
        2 =>println!("Please enter second number:"),
        3 =>println!("Please enter operation"),
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
fn convert_input_type(input:&str,calc:&Calculator)->u32{
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
        inputformat::invalid =>{
            println!("Invalid input! Please Try again");
            0
        },
    }
}

//&, AND, and
fn determine_input_operation(raw:&str) ->operation{
    let input = raw.trim();
    let three_char = &input[0..3];
    let one_char = &input[0..1];
    let two_char = &input[0..2];
    //check for all 3 character operands
    match three_char{
        "AND" =>operation::AND,
        "and" =>operation::AND,
        "XOR" =>operation::XOR,
        "xor" =>operation::XOR,
        _ => {
            match two_char {
                "or" =>operation::OR,
                "OR" =>operation::OR,
                _ =>{
                    match one_char {
                        "^" =>operation::XOR,
                        "|" =>operation::OR,
                        "&" =>operation::AND,
                        _ => operation::invalid,
                    }
                    }
                }
            }
        }
}