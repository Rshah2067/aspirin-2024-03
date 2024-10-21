use std::{collections::HashMap, path::{Path, PathBuf},env};
use clap::{Error, Parser};
use printer::tolines;
use serde_json::Value;
use std::fs;
mod printer;
mod processor;
#[derive(Parser)]
struct Args{
    #[arg(short,long,default_value_t = true)]
    coloroutput:bool,
    #[arg(short,long,default_value_t = false)]
    monochromeoutput:bool,
    #[arg(short,long,default_value_t = false)]
    sortkeys:bool,
    #[arg(short,long,default_value_t = 2)]
    indent:u32,
    #[arg(long,default_value_t = false)]
    compactoutput:bool,
    needle:String,
    path:String,
}
fn main() {
    let args = Args::parse();
    let test_var: std::result::Result<String, env::VarError> = env::var("JQ_COLORS");
    let json:Value = parseInputFile(args.path); 
    println!("{}",json.is_object());
    
    // let mut length = length::new(json);
    // let output = length.operate();
    // match output {
    //     Result::String(string) =>println!("{:#}",string),
    //     Result::Value(val)=>println!("{:?}",val),
    //     Result::Lenght(num) =>println!("{:?}",num),
    // }
    
    }
//TODO add error handling
fn parseInputFile(path:String)->Value{
    let contents = fs::read_to_string(path).unwrap();
    let parsed:Value= serde_json::from_str(&contents).unwrap();
    parsed
}