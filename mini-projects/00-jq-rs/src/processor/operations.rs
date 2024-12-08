use serde_json::Value;
//Enum representing the different outputs we could get from an operation
#[derive(PartialEq, Debug)]
pub enum Result {
    String(String),
    Value(Value),
    Lenght(usize),
}
pub trait Operate {
    fn operate(&mut self) -> Result;
}
pub struct Add {
    input: Value,
}
impl Add {
    pub fn new(input: Value) -> Self {
        let output = Add { input: input };
        output
    }
}
//Implements adding functionality for all potential value types
impl Operate for Add {
    fn operate(&mut self) -> Result {
        let mut output: String = String::new();
        for val in self.input.as_array().unwrap() {
            match val {
                Value::Number(num) => output.push_str(&num.to_string()),
                Value::String(string) => output.push_str(string),
                Value::Array(arr) => {
                    for val in arr {
                        output.push_str(&val.as_str().unwrap());
                    }
                }
                Value::Bool(bool) => output.push_str(&bool.to_string()),
                Value::Object(obj) => {
                    for (key, value) in obj {
                        output.push_str(&key);
                        output.push_str(" ");
                        output.push_str(&value.to_string());
                    }
                }
                Value::Null => output.push_str("null"),
            };
        }
        Result::String(output)
    }
}
pub struct Delete {
    input: Value,
    needle: String,
}
impl Delete {
    pub fn new(input: Value, needle: String) -> Self {
        let output = Delete {
            input: input,
            needle: needle,
        };
        output
    }
}
impl Operate for Delete {
    fn operate(&mut self) -> Result {
        if self.input.is_array() {
            //don't need to error handle here as we explicitly check that input is an arr
            let arr = self.input.as_array_mut().unwrap();
            //do need to error handle here if the needle is not a
            let index: usize = self.needle.parse().unwrap();
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
pub struct Length {
    input: Value,
}
impl Length {
    pub fn new(input: Value) -> Self {
        let output = Length { input: input };
        output
    }
}
impl Operate for Length {
    fn operate(&mut self) -> Result {
        let mut output=0;
        match &self.input {
            Value::Null => output = 0,
            Value::Bool(_) => output = 0,
            Value::Array(arr) => output = arr.len(),
            Value::Object(map) => output = map.len(),
            Value::Number(_) => output = 1,
            Value::String(string) => output = string.len(),
        }
        Result::Lenght(output)
    }
}
mod test {
    use super::*;
    use serde_json::json;
    use std::fs;
    #[test]
    fn test_add() {
        //testing adding arrays
        let input: Value = parse_input_file(String::from("sample_data/array.json"));
        let mut adder = Add { input: input };
        assert_eq!(adder.operate(), Result::String(String::from("onetwothree")));
        //testing more complex array
        let input: Value = parse_input_file(String::from("sample_data/football.json"));
        let mut adder = Add { input: input };
        let output = Result::String(String::from(
            r#"name "Leo Lightning"position "Forward"team "Thunder FC"goals 32favorite_move {"name":"Rocket Shot","success_rate":85}name "Maximus Defender"position "Defender"team "Iron Wall United"goals 76favorite_move {"name":"Slide Tackle","success_rate":88}name "Sophie Swift"position "Midfielder"team "Falcon FC"goals 24favorite_move {"name":"Eagle Pass","success_rate":95}"#,
        ));
        assert_eq!(output, adder.operate())
    }
    #[test]
    fn test_length() {
        //where possible I am using test data, for some types like bool,null etc I am making mocks
        //array
        let input: Value = parse_input_file(String::from("sample_data/array.json"));
        let mut length = Length { input: input };
        assert_eq!(length.operate(), Result::Lenght(3));
        //object
        let input: Value = parse_input_file(String::from("sample_data/all_types.json"));
        let mut length = Length { input: input };
        assert_eq!(length.operate(), Result::Lenght(6));
        //null
        let input = json!(null);
        let mut length = Length { input: input };
        assert_eq!(length.operate(), Result::Lenght(0));
        //string
        let input = json!("test");
        let mut length = Length { input: input };
        assert_eq!(length.operate(), Result::Lenght(4));
        //num
        let input = json!(5);
        let mut length = Length { input: input };
        assert_eq!(length.operate(), Result::Lenght(1));
        //boolean
        let input = json!(true);
        let mut length = Length { input: input };
        assert_eq!(length.operate(), Result::Lenght(0));
    }
    #[test]
    fn test_delete() {
        let input: Value = parse_input_file(String::from("sample_data/array.json"));
        let mut del = Delete {
            input: input,
            needle: String::from("1"),
        };
        let output = json!(["one", "three"]);
        assert_eq!(del.operate(), Result::Value(output));
        //test more complex array
        let input: Value = parse_input_file(String::from("sample_data/football.json"));
        let mut del = Delete {
            input: input,
            needle: String::from("1"),
        };
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
                "name": "Sophie Swift",
                "position": "Midfielder",
                "team": "Falcon FC",
                "goals": 24,
                "favorite_move": {
                    "name": "Eagle Pass",
                    "success_rate": 95
                }
            }
        ]);
        assert_eq!(del.operate(), Result::Value(output));
        //test map
        let input: Value = parse_input_file(String::from("sample_data/all_types.json"));
        let mut del = Delete {
            input: input,
            needle: String::from("fizz"),
        };
        let output = json!({
            "baz": null,
            "fuzz": true,
            "bizz": 22.0,
            "biz": 42,
            "fizzes": [
                "buzz",
                null,
                true,
                22.0,
                42.0
            ]
        });
        assert_eq!(del.operate(), Result::Value(output));
    }
    //helper function to make testing simpler
    fn parse_input_file(path: String) -> Value {
        let contents = fs::read_to_string(path).unwrap();
        let parsed: Value = serde_json::from_str(&contents).unwrap();
        parsed
    }
}
