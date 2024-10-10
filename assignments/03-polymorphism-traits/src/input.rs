
use anyhow::Ok;
use anyhow::Result;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;
pub trait Input {
    fn parse_input(&mut self) -> Result<Vec<String>>;
}
pub struct FileInput {
    file_path: PathBuf,
    contents: Vec<String>,
}
pub struct StringInput {
    contents: Vec<String>,
}
//constructors
impl FileInput {
    pub fn new(file_path: PathBuf) -> Self {
        let output = FileInput {
            file_path: file_path,
            contents: Vec::new(),
        };
        output
    }
}
impl StringInput {
    pub fn new() -> Self {
        let output = StringInput {
            contents: Vec::new(),
        };
        output
    }
}
impl Input for FileInput {
    fn parse_input(&mut self) -> Result<Vec<String>> {
        //take the file path that is given and parse it into a vector of strings
        let file = File::open(self.file_path.clone())?;
        let buf_reader = BufReader::new(file);
        let output = buf_reader
            .lines()
            .map(|l: Result<String, io::Error>| l.unwrap())
            .collect();
        Ok(output)
    }
}
impl Input for StringInput {
    fn parse_input(&mut self) -> Result<Vec<String>> {
        let stdin = io::stdin();
        let buf_reader = BufReader::new(stdin);
        Ok(buf_reader
            .lines()
            .map(|l: Result<String, io::Error>| l.unwrap())
            .collect())
    }
}
mod tests {
    use super::*;
    #[test]
    fn test_parse_input() {
        //test the parse input of a file
        //create a mock parser
        let mut path = PathBuf::new();
        path.push("testfile");
        let mut file_test = FileInput::new(path);
        //Text inside of test file
        let result = vec![
            String::from("Test Test Test"),
            String::from("look test test test"),
            String::from("fun fun fun"),
        ];
        assert!(result == file_test.parse_input().unwrap());
        //In order to right test code for the stdin parser I would need to refractor my code to accept a generic bufferreader
        //I did not realize that Stdin was tricky to unit test when I wrote this code, but don't want to go back and refractor all of my input code
    }
}
