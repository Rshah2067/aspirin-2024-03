use serde_json::{value, Value};
//All of the types of outputs our filtering/operations could leave for us to print
enum printer_inputs{
    String(String),
    Value(Value),
    Number(usize),
}
fn print(input:printer_inputs,color:bool,sort:bool,indent:u32,compact:bool,JQ_Color:&str){
    //convert the given JQ_Env string into a vector of numbers representing the color/format
    let jq_vec = parse_jq_env(JQ_Color);
    //while match statements may not be super scalable, short of adding a new type of value JSON supports
    //we shouldn't need to add more conditions here
    match input {
        
        printer_inputs::String(string) =>{
            println!("{}",printstring(string, color, jq_vec))
        }
        printer_inputs::Number(num) =>{
            println!("{}",printnumb(num as f64,color,jq_vec))
        }
        printer_inputs::Value(val) =>{
            match val {
                Value::Null =>todo!(),
                Value::Bool(bool) =>todo!(),
                Value::Number(numb) =>println!("{}",printnumb(numb.as_f64().unwrap(),color,jq_vec)),
                Value::Array(arr) =>{
                    todo!()
                }
                Value::Object(obj) =>{
                    //need to do this recursively
                }
                Value::String(string) => println!("{}",printstring(string, color, jq_vec)),
            };
        }
    };

}
//Helper function for printing strings, as this code appears twice as we can get a String from the add operator
//or a string from a Value
fn printstring(input:String,color:bool,jq_vec:Vec<(&str,&str)>) ->String{
    let mut output = input;
    if color{
        //construct the appropriate color escape sequence
        let mut start_color:String = String::from("\x1b[");
        start_color.push_str(jq_vec[4].1);
        start_color.push_str(";");
        start_color.push_str(jq_vec[4].0);
        start_color.push_str("]");
        let end_color = String::from("[0m");
        //insert the color escape sequence for the string
        output.push_str(&end_color);
        output.insert_str(0, &start_color);
        output
    }
    else{
        output
    }
}
//Helper function for printing numbers, as this code appears twice as we can either get a number from the length operator
fn printnumb(input:f64,color:bool,jq_vec:Vec<(&str,&str)>) ->String{
    let mut output = input.to_string();
    if color{
        //construct the appropriate color escape sequence
        let mut start_color:String = String::from("\x1b[");
        start_color.push_str(jq_vec[3].1);
        start_color.push_str(";");
        start_color.push_str(jq_vec[3].0);
        start_color.push_str("]");
        let end_color = String::from("[0m");
        //insert the color escape sequence for the number
        output.push_str(&end_color);
        output.insert_str(0, &start_color);
        output
    }
    else{
        output
    }
}
fn sortkeys(input:Value) ->Value{
    
    todo!()

}
pub fn tolines(input:Value) ->String{
    let mut line = input.to_string();
    line = line.replace("{", "{\n");
    line = line.replace("}","}\n");
    line = line.replace("[", "[\n");
    line = line.replace("]","]\n");
    line = line.replace(",",",\n");

    line
}

//Helper function that takes in the jq_env string and gives back a vector of desired colors and formats
fn parse_jq_env<'a>(jq_env:&'a str) -> Vec<(&'a str,&'a str)>{
    let split_array:Vec<&str> = jq_env.split(":").collect();
    let mut output:Vec<(&str,&str)> = vec![];
    for string in split_array{
        let args:Vec<&str> = string.split(";").collect();
        //code will panic if somehow the enviorment variable is messed up, not user facing so it does not get error handling
        let format_num:&str = args[0];
        let color_num:&str = args[1];
        output.push((format_num,color_num));
    }
    output
}