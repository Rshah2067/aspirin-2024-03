use std::io;
#[derive(PartialEq, Debug)]
enum Operation {
    And,
    Or,
    Xor,
    Invalid,
}
#[derive(PartialEq, Debug)]
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
fn preform_operation(mut calc: Calculator) -> Calculator {
    match calc.op {
        Operation::Or => {
            calc.output = calc.input1 | calc.input2;
            println!(
                "The result of {} | {} is {}",
                calc.input1, calc.input2, calc.output
            );
            calc
        }
        Operation::Xor => {
            calc.output = calc.input1 ^ calc.input2;
            println!(
                "The result of {} ^ {} is {}",
                calc.input1, calc.input2, calc.output
            );
            calc
        }
        Operation::And => {
            calc.output = calc.input1 & calc.input2;
            println!(
                "The result of {} & {} is {}",
                calc.input1, calc.input2, calc.output
            );
            calc
        }
        Operation::Invalid => {
            println!("Provide Valid Input");
            calc
        }
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
                        "AND" => Operation::And,
                        "and" => Operation::And,
                        "XOR" => Operation::Xor,
                        "xor" => Operation::Xor,
                        _ => Operation::Invalid,
                    }
                }
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::calculator::{
        determine_input_operation, determine_input_type, preform_operation, Calculator,
        Inputformat, Operation,
    };

    #[test]
    fn test_determine_input_operation() {
        assert_eq!(determine_input_operation("^"), Operation::Xor);
        assert_eq!(determine_input_operation("|"), Operation::Or);
        assert_eq!(determine_input_operation("&"), Operation::And);
        assert_eq!(determine_input_operation("AND"), Operation::And);
        assert_eq!(determine_input_operation("and"), Operation::And);
        assert_eq!(determine_input_operation("XOR"), Operation::Xor);
        assert_eq!(determine_input_operation("xor"), Operation::Xor);
        assert_eq!(determine_input_operation("or"), Operation::Or);
        assert_eq!(determine_input_operation("OR"), Operation::Or);
        assert_eq!(determine_input_operation("gibberish"), Operation::Invalid);
    }
    #[test]
    fn test_determine_input_type() {
        let mut Calc: Calculator = Calculator {
            input1: 0,
            input2: 0,
            output: 0,
            inputform: Inputformat::Invalid,
            op: Operation::Invalid,
        };
        //Creating test cases
        let input_num = String::from("5");
        let input_hex = String::from("0xF8");
        let input_binary = String::from("0b011");
        let input_gibberish = String::from("Hello World");
        //testing numeric
        Calc = determine_input_type(&input_num, Calc, 1);
        assert_eq!(Calc.input1, 5);
        assert_eq!(Calc.inputform, Inputformat::Numeric);
        //testing hex
        Calc = determine_input_type(&input_hex, Calc, 1);
        assert_eq!(Calc.input1, 248);
        assert_eq!(Calc.inputform, Inputformat::Hex);
        //testing Binary
        Calc = determine_input_type(&input_binary, Calc, 1);
        assert_eq!(Calc.input1, 3);
        assert_eq!(Calc.inputform, Inputformat::Binary);
        //resseting Calc input field to default
        Calc.input1 = 0;
        //testing gibberish
        Calc = determine_input_type(&input_gibberish, Calc, 1);
        assert_eq!(Calc.input1, 0);
        assert_eq!(Calc.inputform, Inputformat::Invalid);
    }
    #[test]
    fn test_preform_operation() {
        //Case 1 XOR
        let calc: Calculator = Calculator {
            input1: 12,
            input2: 32,
            output: 0,
            inputform: Inputformat::Invalid,
            op: Operation::Xor,
        };
        let xor_test: Calculator = preform_operation(calc);
        assert_eq!(xor_test.output, 44);
        //Case 2 AND
        let calc: Calculator = Calculator {
            input1: 2,
            input2: 27,
            output: 0,
            inputform: Inputformat::Invalid,
            op: Operation::And,
        };
        let and_test: Calculator = preform_operation(calc);
        assert_eq!(and_test.output, 2);
        //Case 3 Or
        let calc: Calculator = Calculator {
            input1: 248,
            input2: 58,
            output: 0,
            inputform: Inputformat::Invalid,
            op: Operation::Or,
        };
        let or_test: Calculator = preform_operation(calc);
        assert_eq!(or_test.output, 250);
        //Case 4 Invalid Operation
        let calc: Calculator = Calculator {
            input1: 2,
            input2: 27,
            output: 0,
            inputform: Inputformat::Invalid,
            op: Operation::Invalid,
        };
        let invalid_test: Calculator = preform_operation(calc);
        assert_eq!(invalid_test.output, 0);
    }
}
