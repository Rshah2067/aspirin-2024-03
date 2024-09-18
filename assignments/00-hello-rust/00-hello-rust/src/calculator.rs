use std::io;
enum Operation {
    And,
    Or,
    Xor,
    Invalid,
}
enum Inputformat {
    Binary,
    Numeric,
    Hex,
    Invalid,
}
struct Calculator {
    op: Operation,
    inputform: Inputformat,
    output: u32,
    input1: u32,
    input2: u32,
}
pub fn run() {
    let mut calculator = Calculator {
        op: Operation::Invalid,
        inputform: Inputformat::Invalid,
        output: 0,
        input1: 0,
        input2: 0,
    };
    request_for_input(1);
    let input = store_input();
    calculator = determine_input_type(&input, calculator, 1);
    request_for_input(3);
    calculator.op = determine_input_operation(&store_input());
    request_for_input(2);
    calculator = determine_input_type(&store_input(), calculator, 2);
    preform_operation(calculator);
}
fn preform_operation(mut calc: Calculator) {
    match calc.op {
        Operation::Or => {
            calc.output = calc.input1 | calc.input2;
            println!(
                "The result of {} | {} is {}",
                calc.input1, calc.input2, calc.output
            );
        }
        Operation::Xor => {
            calc.output = calc.input1 ^ calc.input2;
            println!(
                "The result of {} ^ {} is {}",
                calc.input1, calc.input2, calc.output
            );
        }
        Operation::And => {
            calc.output = calc.input1 & calc.input2;
            println!(
                "The result of {} & {} is {}",
                calc.input1, calc.input2, calc.output
            );
        }
        Operation::Invalid => println!("Provide Valid Input"),
    }
}

fn request_for_input(input_number: u32) {
    match input_number {
        1 => println!("Please enter first number:"),
        2 => println!("Please enter second number:"),
        3 => println!("Please enter Operation"),
        _ => println!("undefined"),
    }
}
fn store_input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input
}
fn determine_input_type(input: &str, mut calc: Calculator, inputnumb: u32) -> Calculator {
    //Checking if input is a base-10 number, if it is return string and set input type to Numeric
    match input.trim().parse::<u32>() {
        Ok(num) => {
            calc.inputform = Inputformat::Numeric;
            if inputnumb == 1 {
                calc.input1 = num;
                calc
            } else {
                calc.input2 = num;
                calc
            }
        }
        //if it is not a Numeric we now want to try to see if it contains a 0b or 0x indicating it is a Hex/Binary
        Err(_) => {
            let prefix = &input[0..2];
            match prefix {
                "0x" => {
                    calc.inputform = Inputformat::Hex;
                    let test = &input[2..];
                    match u32::from_str_radix(test.trim(), 16) {
                        Ok(num) => {
                            if inputnumb == 1 {
                                calc.input1 = num;
                                calc
                            } else {
                                calc.input2 = num;
                                calc
                            }
                        }
                        Err(_) => {
                            calc.inputform = Inputformat::Invalid;
                            calc
                        }
                    }
                }
                "0b" => {
                    calc.inputform = Inputformat::Binary;
                    let test = &input[2..];
                    match u32::from_str_radix(test.trim(), 2) {
                        Ok(num) => {
                            if inputnumb == 1 {
                                calc.input1 = num;
                                calc
                            } else {
                                calc.input2 = num;
                                calc
                            }
                        }
                        Err(_) => {
                            calc.inputform = Inputformat::Invalid;
                            calc
                        }
                    }
                }
                _ => {
                    calc.inputform = Inputformat::Invalid;
                    calc
                }
            }
        }
    }
}

//&, And, and
fn determine_input_operation(raw: &str) -> Operation {
    let input = raw.trim();

    let one_char = &input[0..1];
    //check for all 3 character operands
    match one_char {
        "^" => Operation::Xor,
        "|" => Operation::Or,
        "&" => Operation::And,
        _ => {
            let two_char = &input[0..2];
            match two_char {
                "or" => Operation::Or,
                "OR" => Operation::Or,
                _ => {
                    let three_char = &input[0..3];
                    match three_char {
                        "And" => Operation::And,
                        "and" => Operation::And,
                        "Xor" => Operation::Xor,
                        "xor" => Operation::Xor,
                        _ => Operation::Invalid,
                    }
                }
            }
        }
    }
}
