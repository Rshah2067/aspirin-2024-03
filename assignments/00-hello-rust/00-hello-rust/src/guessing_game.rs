#![warn(missing_docs)]
//! Creates a guessing game which parses user input and compares it to a randomly generated integer.
use rand::Rng;
use std::cmp::Ordering;
use std::io;
///Parses string input from user and returns a i32 int of their guess. In the case of an invalid input the program panics
fn get_input() -> i32 {
    println!("Please input your guess");

    let mut input = String::new();
    //Taking user input using Stdin
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    //Parsing input to an int, handling errors with a panic
    match input.trim().parse() {
        Ok(num) => num,
        Err(_) => panic!("Invalid entry."),
    }
}
///Main function containing Game flow
fn main() {
    println!("Guess the number!");
    //Generating random number between 1-100
    let secret_number = rand::thread_rng().gen_range(1..=100);
    //Loop until user guesses correctly
    loop {
        //calls input function to get user guess
        let guess = get_input();
        print!("You guessed: {}. ", guess);
        //Checks if guess is equal to number using compare function and gives user feedback on guess
        match secret_number.cmp(&guess) {
            Ordering::Equal => {
                println!("That is correct!");
                break;
            }
            Ordering::Greater => println!("You're guess is too low."),
            Ordering::Less => println!("You're guess is too high."),
        }
    }
}
