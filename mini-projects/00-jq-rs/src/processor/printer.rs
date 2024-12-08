use serde_json::Value;
//All of the types of outputs our filtering/operations could leave for us to print
pub enum PrinterInputs {
    String(String),
    Value(Value),
    Number(usize),
}
//may be neater to compact inputs into a tuple
pub fn print(
    input: PrinterInputs,
    color: bool,
    sort: bool,
    indent: u32,
    compact: bool,
    jq_color: &str,
) {
    //convert the given JQ_Env string into a vector of numbers representing the color/format
    let jq_vec = parse_jq_env(jq_color);
    //while match statements may not be super scalable, short of adding a new type of value JSON supports
    //we shouldn't need to add more conditions here
    match input {
        PrinterInputs::String(string) => {
            println!("{}", print_string(string, color, jq_vec[4].0, jq_vec[4].1))
        }
        PrinterInputs::Number(num) => {
            println!("{}", print_numb(num as f64, color, jq_vec))
        }
        PrinterInputs::Value(val) => {
            println!("{}", print_value(val, color, sort, indent, compact, jq_vec));
        }
    };
}
fn print_value(
    input: Value,
    color: bool,
    sort: bool,
    indent: u32,
    compact: bool,
    jq_vec: Vec<(&str, &str)>,
) -> String {
    let mut output: String = String::new();
    match input {
        Value::Null => output.push_str(&print_null(color, jq_vec)),
        Value::Bool(bool) => {
            if bool {
                output.push_str(&print_true(color, jq_vec))
            } else {
                output.push_str(&print_false(color, jq_vec))
            }
        }
        Value::Number(numb) => output.push_str(&print_numb(numb.as_f64().unwrap(), color, jq_vec)),
        Value::String(string) => {
            //for some reason Value drops the quotes, I can't figure it out so I am adding them back manually
            let mut fixed_string = String::from("\"");
            fixed_string.push_str(&string);
            fixed_string.push_str("\"");
            output.push_str(&print_string(
                fixed_string,
                color,
                &jq_vec[4].0,
                &jq_vec[4].1,
            ))
        }
        Value::Array(arr) => {
            output.push_str(&print_array(arr, color, indent, compact, sort, jq_vec));
        }
        Value::Object(obj) => {
            //convert map to a vector with a tuple to make sorting and working easier
            let mut vectorized_map: Vec<(String, Value)> = obj.into_iter().collect();
            //if we do want to sort we do it here
            if sort {
                vectorized_map.sort_by_key(|(key, _value)| key.clone());
            }
            output.push_str(&print_object(
                vectorized_map,
                color,
                sort,
                indent,
                compact,
                jq_vec,
            ));
            //now send to printer
        }
    };
    output
}
//Helper function for printing all of our data types
//This is not the most scalable code, but this is because there are a finite amount of Value types and this won't change
fn print_object(
    input: Vec<(String, Value)>,
    color: bool,
    sort: bool,
    indent: u32,
    compact: bool,
    jq_env: Vec<(&str, &str)>,
) -> String {
    let mut output = String::new();
    //start by printing outer curly bracket
    output.push_str(&array_color_helper(
        String::from("{"),
        color,
        compact,
        &jq_env[6].0,
        &jq_env[6].1,
        indent,
    ));
    //now we will loop through Value key pairs and print them
    for tuple in input {
        //add back the quotes of the string
        let mut fixed = String::from("\"");
        fixed.push_str(&tuple.0);
        fixed.push_str("\"");
        output.push_str(&print_string(fixed, color, &jq_env[7].0, &jq_env[7].1));
        output.push_str(": ");
        //now we can print the value
        output.push_str(&print_value(
            tuple.1,
            color,
            sort,
            indent,
            compact,
            jq_env.clone(),
        ));
        //now we need to add the comma and indents
        output.push_str(&array_color_helper(
            String::from(","),
            color,
            compact,
            &jq_env[6].0,
            &jq_env[6].1,
            indent,
        ));
    }
    //deal with additional indent
    let mut output = String::from(output.trim_end());
    output.push_str("\n");
    output.push_str(&array_color_helper(
        String::from("}"),
        color,
        true,
        &jq_env[6].0,
        &jq_env[6].1,
        indent,
    ));
    output
}
fn print_array(
    arr: Vec<Value>,
    color: bool,
    indent: u32,
    compact: bool,
    sort: bool,
    jq_env: Vec<(&str, &str)>,
) -> String {
    //start by printing the outer square bracket and a new line
    let mut output = String::new();
    output.push_str(&array_color_helper(
        String::from("["),
        color,
        compact,
        &jq_env[5].0,
        &jq_env[5].1,
        indent,
    ));
    //now we are ready to loop through the elements of the array and print each value, if the value is itself an array we will recursively call this function
    for val in arr {
        //prints whatever value it finds
        output.push_str(&print_value(
            val,
            color,
            sort,
            indent,
            compact,
            jq_env.clone(),
        ));
        //print deliminating comma and indent if desired
        output.push_str(&array_color_helper(
            String::from(","),
            color,
            compact,
            &jq_env[5].0,
            &jq_env[5].1,
            indent,
        ));
    }
    //dealing with the additional indent
    let mut output = String::from(output.trim_end());
    output.push_str("\n");
    output.push_str(&array_color_helper(
        String::from("]"),
        color,
        true,
        &jq_env[5].0,
        &jq_env[5].1,
        indent,
    ));
    output //now we print out each value of the array, for each
}
fn create_indent(indent: u32) -> String {
    let mut output = String::new();
    let mut i = 0;
    while i <= indent {
        output.push_str(" ");
        i += 1;
    }
    output
}
fn array_color_helper(
    input: String,
    color: bool,
    compact: bool,
    print_format: &str,
    print_color: &str,
    indent: u32,
) -> String {
    let mut output = String::new();
    if color {
        output.push_str("\x1b[");
        output.push_str(print_color);
        if print_format != "0" {
            output.push_str(print_format);
        }
        output.push_str("m");
        output.push_str(&input);
        output.push_str("\x1b[0m");
    } else {
        output.push_str(&input);
    }
    if !compact {
        output.push_str("\n");
        output.push_str(&create_indent(indent));
    }
    output
}
fn print_string(input: String, color: bool, print_format: &str, print_color: &str) -> String {
    let mut output = String::new();
    if color {
        output.push_str("\x1b[");
        output.push_str(print_color);
        if print_format != "0" {
            output.push_str(";");
            output.push_str(print_format);
        }
        output.push_str("m");
        output.push_str(&input);
        output.push_str("\x1b[0m");
    } else {
        output.push_str(&input);
    }
    output
}
fn print_numb(input: f64, color: bool, jq_env: Vec<(&str, &str)>) -> String {
    let mut output = String::new();
    let stringin = input.to_string();
    if color {
        output.push_str("\x1b[");
        output.push_str(jq_env[3].1);
        if jq_env[4].0 != "0" {
            output.push_str(jq_env[3].0);
        }
        output.push_str("m");
        output.push_str(&stringin);
        output.push_str("\x1b[0m");
    } else {
        output.push_str(&stringin);
    }
    output
}
fn print_null(color: bool, jq_env: Vec<(&str, &str)>) -> String {
    let mut output = String::new();
    let input = "null";
    if color {
        output.push_str("\x1b[");
        output.push_str(jq_env[0].1);
        if jq_env[4].0 != "0" {
            output.push_str(jq_env[0].0);
        }
        output.push_str("m");
        output.push_str(&input);
        output.push_str("\x1b[0m");
    } else {
        output.push_str(&input);
    }
    output
}
fn print_false(color: bool, jq_env: Vec<(&str, &str)>) -> String {
    let mut output = String::new();
    let input = "false";
    if color {
        output.push_str("\x1b[");
        output.push_str(jq_env[1].1);
        if jq_env[4].0 != "0" {
            output.push_str(jq_env[1].0);
        }
        output.push_str("m");
        output.push_str(&input);
        output.push_str("\x1b[0m");
    } else {
        output.push_str(&input);
    }
    output
}
fn print_true(color: bool, jq_env: Vec<(&str, &str)>) -> String {
    let mut output = String::new();
    let input = "true";
    if color {
        output.push_str("\x1b[");
        output.push_str(jq_env[2].1);
        if jq_env[4].0 != "0" {
            output.push_str(jq_env[2].0);
        }
        output.push_str("m");
        output.push_str(&input);
        output.push_str("\x1b[0m");
    } else {
        output.push_str(&input);
    }
    output
}
//Helper function that takes in the jq_env string and gives back a vector of desired colors and formats
fn parse_jq_env<'a>(jq_env: &'a str) -> Vec<(&'a str, &'a str)> {
    let split_array: Vec<&str> = jq_env.split(":").collect();
    let mut output: Vec<(&str, &str)> = vec![];
    for string in split_array {
        let args: Vec<&str> = string.split(";").collect();
        //code will panic if somehow the enviorment variable is messed up, not user facing so it does not get error handling
        let format_num: &str = args[0];
        let color_num: &str = args[1];
        output.push((format_num, color_num));
    }
    output
}
mod test {
    use super::*;
    use std::fs;
    #[test]
    fn test_print_string() {
        //testing if the string is correctly printing using the color specified in JQ Color
        let jq_color = "0;90:0;37:0;37:0;37:0;32:1;37:1;37:1;34";
        let jq_vec = parse_jq_env(jq_color);
        assert_eq!(
            print_string(String::from("\"test\""), true, jq_vec[4].0, jq_vec[4].1),
            "\x1b[32m\"test\"\x1b[0m"
        );
        //no color
        let jq_vec = parse_jq_env(jq_color);
        assert_eq!(
            print_string(String::from("test"), false, jq_vec[4].0, jq_vec[4].1),
            "test"
        );
    }
    #[test]
    fn test_print_numb() {
        //testing if the string is correctly printing using the color specified in JQ Color
        let jq_color = "0;90:0;37:0;37:0;37:0;32:1;37:1;37:1;34";
        let jq_vec = parse_jq_env(jq_color);
        assert_eq!(print_numb(4.0, true, jq_vec), "\x1b[37m4\x1b[0m");
        //no color
        let jq_vec = parse_jq_env(jq_color);
        assert_eq!(print_numb(4.0, false, jq_vec), "4");
    }
    #[test]
    fn test_print_null() {
        let jq_color = "0;90:0;37:0;37:0;37:0;32:1;37:1;37:1;34";
        let jq_vec = parse_jq_env(jq_color);
        assert_eq!(print_null(true, jq_vec), "\x1b[90mnull\x1b[0m");
        //no color
        let jq_vec = parse_jq_env(jq_color);
        assert_eq!(print_null(false, jq_vec), "null");
    }
    #[test]
    fn test_print_false() {
        let jq_color = "0;90:0;37:0;37:0;37:0;32:1;37:1;37:1;34";
        let jq_vec = parse_jq_env(jq_color);
        assert_eq!(print_false(true, jq_vec), "\x1b[37mfalse\x1b[0m");
        //no color
        let jq_vec = parse_jq_env(jq_color);
        assert_eq!(print_false(false, jq_vec), "false");
    }
    #[test]
    fn test_print_true() {
        let jq_color = "0;90:0;37:0;37:0;37:0;32:1;37:1;37:1;34";
        let jq_vec = parse_jq_env(jq_color);
        assert_eq!(print_true(true, jq_vec), "\x1b[37mtrue\x1b[0m");
        //no color
        let jq_vec = parse_jq_env(jq_color);
        assert_eq!(print_true(false, jq_vec), "true");
    }
    #[test]
    fn test_print_array() {
        let jq_color = "0;90:0;37:0;37:0;37:0;32:1;37:1;37:1;34";
        let jq_env = parse_jq_env(jq_color);
        let input = parse_input_file(String::from("sample_data/array.json"));
        let vec = input.as_array().unwrap();
        let output = String::from("[\n   \"one\",\n   \"two\",\n   \"three\",\n]");
        //testing non colored
        assert_eq!(
            print_array(vec.to_vec(), false, 2, false, false, jq_env.clone()),
            output
        );
        //testing colored
        let output = String::from("\u{1b}[371m[\u{1b}[0m\n   \u{1b}[32m\"one\"\u{1b}[0m\u{1b}[371m,\u{1b}[0m\n   \u{1b}[32m\"two\"\u{1b}[0m\u{1b}[371m,\u{1b}[0m\n   \u{1b}[32m\"three\"\u{1b}[0m\u{1b}[371m,\u{1b}[0m\n\u{1b}[371m]\u{1b}[0m");
        assert_eq!(
            print_array(vec.to_vec(), true, 2, false, false, jq_env),
            output
        );
    }
    #[test]
    fn test_print_obj() {
        let jq_color = "0;90:0;37:0;37:0;37:0;32:1;37:1;37:1;34";
        let jq_env = parse_jq_env(jq_color);
        let input = parse_input_file(String::from("sample_data/all_types.json"));
        println!(
            "{}",
            print_value(input, true, false, 2, false, jq_env.clone())
        );
    }
    //helper function to make testing simpler
    fn parse_input_file(path: String) -> Value {
        let contents = fs::read_to_string(path).unwrap();
        let parsed: Value = serde_json::from_str(&contents).unwrap();
        parsed
    }
}
