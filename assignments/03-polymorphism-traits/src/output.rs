use colored::Color;
use colored::Colorize;
use regex::Regex;
use std::io::Write;
pub trait Output {
    fn print_output(&mut self, desired_output: Vec<String>);
}
pub struct output_color {
    output: Vec<String>,
    needle: String,
    color: Color,
}
pub struct output_nocolor {
    output: Vec<String>,
}
impl output_color {
    pub fn new(needle: String, color: Color) -> Self {
        let output = output_color {
            output: Vec::new(),
            needle: needle,
            color: color,
        };
        output
    }
}
impl output_nocolor {
    pub fn new() -> Self {
        let ouput = output_nocolor { output: Vec::new() };
        ouput
    }
}
impl Output for output_color {
    fn print_output(&mut self, desired_output: Vec<String>) {
        //need to search for indices to color
        //check if needle is a regex or not
        if Regex::new(&self.needle).is_ok() {
            let re = Regex::new(&self.needle).unwrap();
            //for each line of desired output find needle position and annotate
            for line in desired_output {
                let m = re.find(&line).unwrap();
                //the range of the match corresponds to bits with our needle, we want to color them to our desired coulor
                let colored = &line[m.range()];
                match self.color {
                    Color::Black => colored.black(),
                    Color::Blue => colored.blue(),
                    Color::Red => colored.red(),
                    Color::Green => colored.green(),
                    Color::Yellow => colored.yellow(),
                    //enum contains other color types even though it shouldn't
                    _ => colored.black(),
                };
                //we now want to recombine our line into one string and repackage the combined line into
                let mut colored_line: String = String::from(&line[0..m.start()]);
                colored_line.push_str(colored);
                colored_line.push_str(&line[m.end()..]);
                self.output.push(colored_line);
            }
        }
        //if our needle is a string
        else {
            for line in desired_output {
                let mut colored: String = String::new();
                self.needle.clone_into(&mut colored);
                match self.color {
                    Color::Black => colored.black(),
                    Color::Blue => colored.blue(),
                    Color::Red => colored.red(),
                    Color::Green => colored.green(),
                    Color::Yellow => colored.yellow(),
                    //enum contains other color types even though it shouldn't
                    _ => colored.black(),
                };
                //repackage colored needle and add to output
                let mut colored_line = String::from(&line[0..line.find(&self.needle).unwrap()]);
                colored_line.push_str(&colored);
                colored_line.push_str(&line[line.find(&self.needle).unwrap() + 1..]);
                self.output.push(colored_line);
            }
        }
        let output = self.output.clone();
        for line in output {
            println!("{}", line);
        }
    }
}
impl Output for output_nocolor {
    fn print_output<'b>(&mut self, desired_output: Vec<String>) {
        desired_output.clone_into(&mut self.output);
        for line in desired_output {
            println!("{}", line);
        }
    }
}
mod test {
    use std::clone;

    use super::*;
    #[test]
    fn test_output() {
        //create mock output lines
        let output = vec![String::from("look test blank")];
        //create mock interfaces
        let mut no_color = output_nocolor::new();
        no_color.print_output(output.clone());
        //test uncolored
        assert!(output == no_color.output);
        //test colored by creating a mock input for each color
        //blue
        let mut blue = output_color::new(String::from("test"), Color::Blue);
        blue.print_output(output.clone());
        let mut blue_string = String::from("look ");
        blue_string.push_str(&String::from("test").blue());
        blue_string.push_str(" blank");
        let out_vec = vec![blue_string.clone()];
        assert!(out_vec == blue.output);
        //black
        let mut black = output_color::new(String::from("test"), Color::Black);
        black.print_output(output.clone());
        let mut black_string = String::from("look ");
        black_string.push_str(&String::from("test").blue());
        black_string.push_str(" blank");
        let out_vec = vec![black_string.clone()];
        assert!(out_vec == black.output);
        //green
        let mut green = output_color::new(String::from("test"), Color::Green);
        green.print_output(output.clone());
        let mut green_string = String::from("look ");
        green_string.push_str(&String::from("test").green());
        green_string.push_str(" blank");
        let out_vec = vec![green_string.clone()];
        //yellow
        let mut yellow = output_color::new(String::from("test"), Color::Yellow);
        yellow.print_output(output.clone());
        let mut yellow_string = String::from("look ");
        yellow_string.push_str(&String::from("test").yellow());
        yellow_string.push_str(" blank");
        let out_vec = vec![yellow_string.clone()];
    }
}
