use std::{collections::HashMap, process::Output};
use serde_json::{value, Value};
use std::ops::Range;
//The pipe and iterator filter modify the way filters are executed and aren't treated as filters
#[derive(PartialEq, Debug)]
pub enum FilterType{
    Identity,
    ObjectIdentity,
    ArrayIndex,
    ArraySlice,
}
//Filters an input value based on the filtertype and a needle if needed
pub fn filter(input:Value,filter:FilterType,needle:String) -> Value{
    match filter {
        FilterType::Identity =>{
            input
        }
        FilterType::ObjectIdentity =>{
            let output = match input.get(needle){
                Some(val) =>val.clone(),
                None => Value::Null
            };
            output
        }
        FilterType::ArrayIndex =>{
            let index:usize = needle.parse().unwrap();
            let output = match input.get(index){
                Some(val) =>val.clone(),
                None => Value::Null
            };
            output
        }
        FilterType::ArraySlice =>{
            let array = input.as_array().unwrap();
            let mut stringvec:Vec<&str> = Vec::new();
            for val in array{
                stringvec.push(val.as_str().unwrap());
            }
            //use helper function to correctly slice array
            let output:Value = Value::from(&stringvec[parseslice(needle)]);
            output
        }
    }
}

//Helper Function that parses the string for a range
fn parseslice(stringneedle:String) ->Range<usize>{
    let parts:Vec<&str> = stringneedle.split(":").collect();
    let start:usize = parts[0].parse().unwrap();
    let finish:usize = parts[1].parse().unwrap();
    start..finish
}
mod test{
    use super::*;
    use std::fs;
    #[test]
    fn test_identity(){
        let test:Value = parseInputFile(String::from("sample_data/students.json"));
        assert_eq!(test,filter(test.clone(),FilterType::Identity,String::new()))
    }
    #[test]
    fn test_object_index(){
        let test:Value = parseInputFile(String::from("sample_data/students.json"));
        assert_eq!(test,filter(test.clone(),FilterType::Identity,String::new()))
    }
    //helper function to make testing simpler
    fn parseInputFile(path:String)->Value{
        let contents = fs::read_to_string(path).unwrap();
        let parsed:Value= serde_json::from_str(&contents).unwrap();
        parsed
    }
}



