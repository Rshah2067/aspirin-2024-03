use filter::*;
use operations::*;
use regex::Regex;
use serde_json::Value;
mod filter;
mod operations;
use printer::*;
mod printer;
#[derive(PartialEq, Debug)]
enum Commands {
    Add,
    Delete,
    Length,
    ArrayIterator,
    Filter(FilterType),
}
//filter needle returns a list of commands to execute and a String with the needle for operations that require parameters
//if a parameter is not needed the String is empty.
//this is uglier than it could be due to excessive nesting, I could split up the searching into different functions
fn filterneedle(input: String) -> Vec<(Commands, String)> {
    //start by splitting by pipes, each resulting vector should be a command
    let command_strings: Vec<&str> = input.split("|").collect();
    let mut output: Vec<(Commands, String)> = vec![];
    //we will now go through each arguement and try to find our commands
    for mut string in command_strings {
        string = string.trim();
        //First search for our operations that don't have modifiying parameters (add, length, identity, array iterator)
        //If it is not one of those then we will check for one of the filters/operations that has a modifiable parameter
        match string {
            "add" => output.push((Commands::Add, String::new())),
            "length" => output.push((Commands::Length, String::new())),
            "." => output.push((Commands::Filter(FilterType::Identity), String::new())),
            ".[]" => output.push((Commands::ArrayIterator, String::new())),
            //now we want to search for commands like delete[], delete<>,array index,."key" and array slice
            //most convient way to do this I found was to use REGEX
            _ => {
                //no need for errorhandling as the regex is not userfacing
                let delete_key_regex = Regex::new(r"del\(\.(.*?)\)").unwrap();
                let delete_index_regex = Regex::new(r"del\[\.(.*?)\]").unwrap();
                let array_slice_regex = Regex::new(r"\.\[([^\]]*:[^\]]*)\]").unwrap();
                let array_index_regex = Regex::new(r"\.\[(.*?)\]").unwrap();
                let object_regex = Regex::new(r"\.(\w+)").unwrap();
                match delete_key_regex.captures(string) {
                    //this means we found a delete regex with a key needle
                    Some(needle) => output.push((Commands::Delete, String::from(&needle[1]))),
                    None => {
                        //check if we have a delete with an index
                        match delete_index_regex.captures(string) {
                            Some(needle) => {
                                output.push((Commands::Delete, String::from(&needle[1])))
                            }
                            None => {
                                //Check for an array slice
                                match array_slice_regex.captures(string) {
                                    Some(needle) => output.push((
                                        Commands::Filter(FilterType::ArraySlice),
                                        String::from(&needle[1]),
                                    )),
                                    None => {
                                        //check if it is an array index
                                        match array_index_regex.captures(string) {
                                            Some(needle) => output.push((
                                                Commands::Filter(FilterType::ArrayIndex),
                                                String::from(&needle[1]),
                                            )),
                                            None => {
                                                //finally we can check if it is an object index, if it isn't this than the input was invalid so the code will panic
                                                let needle = object_regex.captures(string).unwrap();
                                                output.push((
                                                    Commands::Filter(FilterType::ObjectIdentity),
                                                    String::from(&needle[1]),
                                                ))
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    output
}
//takes in a list of commands and executes them, if there is an invalid command then this will trigger error handling in the filter code
fn process(input: String, initial: Value) -> Result {
    let list = filterneedle(input);
    let mut input = Result::Value(initial);
    for command in list {
        //if the input for the next stage is anything other than a value we should panic and tell the user that they gave an invalid filter
        match input {
            Result::Value(ref val) => input = execute(command.0, command.1, val.clone()),
            _ => panic!("Invalid input filter!"),
        };
    }
    input
}
pub fn call_printer(
    input: String,
    initial: Value,
    color: bool,
    sort: bool,
    indent: u32,
    compact: bool,
    jq_color: &str,
) {
    let output = process(input, initial);
    let input = match output {
        Result::Lenght(numb) => PrinterInputs::Number(numb),
        Result::String(string) => PrinterInputs::String(string),
        Result::Value(val) => PrinterInputs::Value(val),
        //the array iterator shouldn't be the
    };
    print(input, color, sort, indent, compact, jq_color);
}
//executes a specifc command
fn execute(command: Commands, needle: String, val: Value) -> Result {
    match command {
        Commands::Add => {
            let mut adder = Add::new(val);
            adder.operate()
        }
        Commands::Delete => {
            let mut delete = Delete::new(val, needle);
            delete.operate()
        }
        Commands::Length => {
            let mut length = Length::new(val);
            length.operate()
        }
        Commands::ArrayIterator => todo!(),
        Commands::Filter(filtertype) => {
            Result::Value(filter(val, filtertype, needle).expect("Unexpected Erorr with filter"))
        }
    }
}
mod test {
    use super::*;
    //testing simple cases without pipes first
    #[test]
    fn test_filter_add() {
        let input = String::from("add");
        assert_eq!(filterneedle(input), vec![(Commands::Add, String::new())])
    }
    #[test]
    fn test_filter_length() {
        let input = String::from("length");
        assert_eq!(filterneedle(input), vec![(Commands::Length, String::new())])
    }
    #[test]
    fn test_filter_identity() {
        let input = String::from(".");
        assert_eq!(
            filterneedle(input),
            vec![(Commands::Filter(FilterType::Identity), String::new())]
        )
    }
    #[test]
    fn test_filter_delete() {
        let input = String::from("del(.test)");
        assert_eq!(
            filterneedle(input),
            vec![(Commands::Delete, String::from("test"))]
        );
        let input = String::from("del[.0]");
        assert_eq!(
            filterneedle(input),
            vec![(Commands::Delete, String::from("0"))]
        );
    }
    #[test]
    fn test_filter_array_index() {
        //testing array index
        let input = String::from(".[0]");
        assert_eq!(
            filterneedle(input),
            vec![(Commands::Filter(FilterType::ArrayIndex), String::from("0"))]
        );
        //even though this is an invalid input we want to catch it later with our filter code
        let input = String::from(".[balal]");
        assert_eq!(
            filterneedle(input),
            vec![(
                Commands::Filter(FilterType::ArrayIndex),
                String::from("balal")
            )]
        );
    }
    #[test]
    fn test_filter_array_slice() {
        //testing array slice
        let input = String::from(".[0:10]");
        assert_eq!(
            filterneedle(input),
            vec![(
                Commands::Filter(FilterType::ArraySlice),
                String::from("0:10")
            )]
        );
        //even though this is an invalid input we want to catch it later with our filter code
        let input = String::from(".[balal:rand]");
        assert_eq!(
            filterneedle(input),
            vec![(
                Commands::Filter(FilterType::ArraySlice),
                String::from("balal:rand")
            )]
        );
    }
    #[test]
    fn test_filter_object_index() {
        //testing array index
        let input = String::from(".filter");
        assert_eq!(
            filterneedle(input),
            vec![(
                Commands::Filter(FilterType::ObjectIdentity),
                String::from("filter")
            )]
        );
    }
    #[test]
    fn test_filter_array_iterator() {
        //testing array index
        let input = String::from(".[]");
        assert_eq!(
            filterneedle(input),
            vec![(Commands::ArrayIterator, String::new())]
        );
    }
    //Pipe tests
    #[test]
    fn test_filter_pipe() {
        //start with a simple identity to add
        let input = String::from(".|add");
        let output = vec![
            (Commands::Filter(FilterType::Identity), String::new()),
            (Commands::Add, String::new()),
        ];
        assert_eq!(filterneedle(input), output);
        //try something more complicated
        let input = String::from(".[]|.name");
        let output = vec![
            (Commands::ArrayIterator, String::new()),
            (
                Commands::Filter(FilterType::ObjectIdentity),
                String::from("name"),
            ),
        ];
        assert_eq!(filterneedle(input), output);
    }
}
