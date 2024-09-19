pub fn print_fizz_buzz(max_num: u32) {
    //Check if number is divisible by 3 or 5 if not then count down
    
    for i in 1..=max_num{
        if i%3 ==0 && i %5==0{
            println!("FizzBuzz");
        }
        else if i %3 ==0{
            println!("Fizz");
        }else if i %5 == 0 {
            println!("Buzz");
        }else{
            let x: String = i.to_string();
            println!("{}", x);
        }
    }
   
}
