use anyhow::Result;
use clap::Parser;
use colored::Color;
use searching::StringSearch;
use std::path::PathBuf;
mod searching;
use searching::RegexSearch;
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
        Some(path) => Box::new(FileInput::new(path)),
        None => Box::new(StringInput::new()),
    };
    //parsed input
    let strings = input.parse_input()?;
    //do the searching depending on wheter the regex flag is true or not
    let results: Vec<&String>;
    //search using rexeg
    if args.regex {
        let regexsearch = RegexSearch::new(&args.needle);
        results = regexsearch.search(&strings, args.invert_match, args.ignore_case);
    }
    //search normally
    else {
        let normalsearch = StringSearch::new(&args.needle);
        results = normalsearch.search(&strings, args.invert_match, args.ignore_case)
    };
    //now that we have our results we want to print them
    //no color
    if args.color == None {
        let mut printer = OutputNoColor::new();
        printer.print_output(results.into_iter().cloned().collect());
    }
    //colored printing
    else {
        let mut printer = OutputColor::new(args.needle, args.color.unwrap());
        printer.print_output(results.into_iter().cloned().collect());
    }
    Ok(())
}
