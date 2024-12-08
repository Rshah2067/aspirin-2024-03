use serde_json::Value;
use std::ops::Range;
use std::num::ParseIntError;
use thiserror::Error;
//The pipe and iterator filter modify the way filters are executed and aren't treated as filters
#[derive(PartialEq, Debug)]
pub enum FilterType {
    Identity,
    ObjectIdentity,
    ArrayIndex,
    ArraySlice,
}
#[derive(Error, Debug)]
pub enum FilterError {
    #[error("Invalid needle: {0}")]
    InvalidNeedle(#[from] ParseIntError),
    #[error("Not an array, invalid filter stream")]
    NotArray,
}
//Filters an input value based on the filtertype and a needle if needed
pub fn filter(input: Value, filter: FilterType, needle: String) -> Result<Value, FilterError> {
    match filter {
        FilterType::Identity => Ok(input),
        FilterType::ObjectIdentity => {
            let output = match input.get(needle) {
                Some(val) => val.clone(),
                None => Value::Null,
            };
            Ok(output)
        }
        FilterType::ArrayIndex => {
            let index: usize = needle.parse()?;
            let output = match input.get(index) {
                Some(val) => val.clone(),
                None => Value::Null,
            };
            Ok(output)
        }
        FilterType::ArraySlice => {
            let array: &Vec<Value> = input.as_array().ok_or(FilterError::NotArray)?;
            //use helper function to correctly slice array
            let output: Value = match parseslice(needle) {
                Ok(range) => Value::from(&array[range]),
                Err(e) => {
                    eprint!("Error: {}", e);
                    Value::Null
                }
            };
            Ok(output)
        }
    }
}
//Helper Function that parses the string for a range
fn parseslice(stringneedle: String) -> Result<Range<usize>, FilterError> {
    let parts: Vec<&str> = stringneedle.split(":").collect();
    let start: usize = parts[0].parse()?;
    let finish: usize = parts[1].parse()?;
    Ok(start..finish)
}
mod test {
    use super::*;
    use serde_json::json;
    use std::fs;
    //errors here are not bubbled up as they are not user facing
    #[test]
    fn test_identity() {
        let test: Value = parse_input_file(String::from("sample_data/students.json"));
        assert_eq!(
            test,
            filter(test.clone(), FilterType::Identity, String::new()).expect("Failed Unit Test")
        )
    }
    #[test]
    fn test_object_index() {
        let test: Value = parse_input_file(String::from("sample_data/students.json"));
        assert_eq!(
            test,
            filter(test.clone(), FilterType::Identity, String::new()).expect("Failed Unit Test")
        )
    }
    #[test]
    fn test_array_index() {
        //simple case
        let input: Value = parse_input_file(String::from("sample_data/array.json"));
        let output = json!("one");
        assert_eq!(
            filter(input.clone(), FilterType::ArrayIndex, String::from("0"))
                .expect("Failed Unit Test"),
            output
        );
        //more complex case
        let input: Value = parse_input_file(String::from("sample_data/football.json"));
        let output = json!({
        "name": "Leo Lightning",
        "position": "Forward",
        "team": "Thunder FC",
        "goals": 32,
        "favorite_move": {
            "name": "Rocket Shot",
            "success_rate": 85
        }
        });
        assert_eq!(
            filter(input.clone(), FilterType::ArrayIndex, String::from("0"))
                .expect("Failed Unit Test"),
            output
        );
    }
    #[test]
    fn test_array_slice() {
        let input: Value = parse_input_file(String::from("sample_data/array.json"));
        let output = json!(["one", "two"]);
        assert_eq!(
            filter(input, FilterType::ArraySlice, String::from("0:2")).expect("Failed Unit Test"),
            output
        );
        //more complex array
        let input: Value = parse_input_file(String::from("sample_data/football.json"));
        let output = json!([
        {
            "name": "Leo Lightning",
            "position": "Forward",
            "team": "Thunder FC",
            "goals": 32,
            "favorite_move": {
                "name": "Rocket Shot",
                "success_rate": 85
            }
        },
        {
            "name": "Maximus Defender",
            "position": "Defender",
            "team": "Iron Wall United",
            "goals": 76,
            "favorite_move": {
                "name": "Slide Tackle",
                "success_rate": 88
            }
        }]);
        assert_eq!(
            filter(input, FilterType::ArraySlice, String::from("0:2")).expect("Failed Unit Test"),
            output
        );
    }
    #[test]
    fn test_helper() {
        assert_eq!(
            0..1,
            parseslice(String::from("0:1")).expect("Failed Unit Test")
        );
        //demonstrating ability to parse numbers with multiple digits
        assert_eq!(
            11..15,
            parseslice(String::from("11:15")).expect("Failed Unit Test")
        );
    }
    //helper function to make testing simpler
    fn parse_input_file(path: String) -> Value {
        let contents = fs::read_to_string(path).unwrap();
        let parsed: Value = serde_json::from_str(&contents).unwrap();
        parsed
    }
}
