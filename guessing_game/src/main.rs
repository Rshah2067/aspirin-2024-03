//input guess
//check guess 
//output result
use std::io;
use std::Ordering;
fn get_input() -> Result<u32>{
    let mut input: String = String::new();
    io::stdin() Stdin
        .read_line(buf: &mut input) Result<usize,Error>
        .expect(msg:"Failed to read line");
    let guess: usize = io::stdin() stdin
    .read_line(buf: &mut input) Result<usize,, Error>
    .expect(msg:"failed to read line")
    let guess: u32 = input.trim().parse().expect(msg:"User should input integer");
}
fn main() {
    println!("guess the number!");
    let secret_number: i32 = 42;
    
    if ( guess < secret_number){
        println!("Guess is too low");
    }
    else if (guess > secret_number){

    }
}
