use clap::Parser;
use processor::*;
use serde_json::Value;
use std::fs;
use std::io;
use std::env;
use thiserror::Error;
mod processor;
#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value_t = true)]
    coloroutput: bool,
    #[arg(short, long, default_value_t = false)]
    monochromeoutput: bool,
    #[arg(short, long, default_value_t = false)]
    sortkeys: bool,
    #[arg(short, long, default_value_t = 2)]
    indent: u32,
    #[arg(long, default_value_t = false)]
    compactoutput: bool,
    needle: String,
    path: String,
}
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("File error: {0}")]
    FileError(#[from] io::Error), // Automatically convert from std::io::Error

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error), // Automatically convert from serde_json::Error
}
fn main() {
    let args = Args::parse();
    let jq_colors: std::result::Result<String, env::VarError> = env::var("JQ_COLORS");
    //check if we found an env variable
    let jq_env = match jq_colors {
        Ok(env) => env,
        //default value is used if one is not found
        Err(_) => String::from("0;90:0;37:0;37:0;37:0;32:1;37:1;37:1;34"),
    };
    let mut json: Value = Value::Null;
    match parse_input_file(args.path) {
        Ok(value) => json = value,
        Err(e) => eprintln!("Error: {}", e),
    }
    call_printer(
        args.needle,
        json,
        args.coloroutput,
        args.sortkeys,
        args.indent,
        args.compactoutput,
        &jq_env,
    );
}
//TODO add error handling
fn parse_input_file(path: String) -> Result<Value, ParseError> {
    let contents = fs::read_to_string(path)?;
    let parsed: Value = serde_json::from_str(&contents)?;
    Ok(parsed)
}
