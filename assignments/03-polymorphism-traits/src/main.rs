use anyhow::Result;
use clap::Parser;
use colored::Color;
use searching::string_search;
use std::path::PathBuf;
mod searching;
use searching::regex_search;
use searching::Searching;
mod input;
use input::*;
mod output;
use output::*;
#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long)]
    ignore_case: bool,

    #[clap(short = 'v', long)]
    invert_match: bool,

    #[clap(short, long)]
    regex: bool,

    #[clap(short, long)]
    color: Option<Color>,

    needle: String,

    file: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("{:?}", args);
    //match statement, if there is a path, create a a file input, if there is not create a string input
    let mut input: Box<dyn Input>;
    input = match args.file {
        Some(path) => Box::new(fileInput::new(path)),
        None => Box::new(stringInput::new()),
    };
    //parsed input
    let strings = input.parse_input()?;
    //do the searching depending on wheter the regex flag is true or not
    let mut results: Vec<&String> = Vec::new();
    //search using rexeg
    if args.regex {
        let regexsearch = regex_search::new(&args.needle);
        results = regexsearch.search(&strings, args.invert_match, args.ignore_case);
    }
    //search normally
    else {
        let normalsearch = string_search::new(&args.needle);
        results = normalsearch.search(&strings, args.invert_match, args.ignore_case)
    };
    //now that we have our results we want to print them
    //no color
    if args.color == None {
        let mut printer = output_nocolor::new();
        printer.print_output(results.into_iter().cloned().collect());
    }
    //colored printing
    else {
        let mut printer = output_color::new(args.needle, args.color.unwrap());
        printer.print_output(results.into_iter().cloned().collect());
    }
    Ok(())
}
