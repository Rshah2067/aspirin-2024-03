use std::collections::{hash_map, HashMap};

use serde_json::{value, Value};
//Enum representing the different outputs we could get from an operation
pub enum Result{
    String(String),
    Value(Value),
    Lenght(usize),
}
pub trait operate{
    fn operate(&mut self)->Result;
}
pub struct add{
    input:Value,
}
impl add{
    pub fn new (input:Value)->Self{
        let output = add{
            input: input,
        };
        output
    }
}
//Implements adding functionality for all potential value types
impl operate for add{
    fn operate(&mut self) ->Result{
        let mut output:String = String::new();
        for val in self.input.as_array().unwrap(){
            match val{
                Value::Number(num) => output.push_str(&num.to_string()),
                Value::String(string) =>output.push_str(string),
                Value::Array(arr) =>{
                    for val in arr{
                        output.push_str(&val.as_str().unwrap());
                    }
                },
                Value::Bool(bool) =>output.push_str(&bool.to_string()),
                Value::Object(obj) =>{
                    for (key,value) in obj{
                        output.push_str(&key);
                        output.push_str(" ");
                        output.push_str(&value.to_string());
                    }
                },
                Value::Null => output.push_str("null"),
            };
        }
        Result::String(output)
    }
}
pub struct delete {
    input:Value,
    needle:String,
}
impl delete {
    pub fn new(input:Value,needle:String)->Self{
        let output = delete{
            input:input,
            needle:needle,
        };
        output
    }
}
impl operate for delete{
    fn operate(&mut self) ->Result{
        if self.input.is_array() {
            //don't need to error handle here as we explicitly check that input is an arr
            let  arr = self.input.as_array_mut().unwrap();
            //do need to error handle here if the needle is not a 
            let index:usize = self.needle.parse().unwrap();
            arr.remove(index);
            Result::Value(Value::Array(arr.to_vec()))
        }
        //if it is not an array it must be an object
        //error handle
        else {
            let map = self.input.as_object_mut().unwrap();
            map.remove(&self.needle);
            Result::Value(Value::Object(map.clone()))
        }
    }
}
pub struct length{
    input:Value,
}
impl length{
    pub fn new (input:Value) ->Self {
            let output = length{
                input:input,
            };
            output
    }
}
impl operate for length{
    fn operate(&mut self)->Result {
        let mut output = 0;
        match &self.input{
            Value::Null => output = 0,
            Value::Bool(_) => output =0,
            Value::Array(arr) => output = arr.len(),
            Value::Object(map) => output = map.len(),
            Value::Number(_) => output = 1,
            Value::String(string) =>output = string.len(),

        }
        Result::Lenght(output)
    }
}


