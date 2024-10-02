pub fn split_string<'a>(string: &'a str, delimeter: &str) -> Vec<&'a str> {
    if(string == ""){
        return vec![]
    }
    else{
        let mut output:Vec<&str> = string.split_terminator(delimeter).collect();
        output
    }
}

#[derive(PartialEq, Debug)]
struct Differences <'a>{
    only_in_first: Vec<&'a str>,
    only_in_second: Vec<&'a str>,
}

pub fn find_differences<'a>(first_string: &'a str, second_string: &'a str) ->Differences<'a> {
    let words_in_first = split_string(first_string, &" ");
    let words_in_second = split_string(second_string,&" " ); 
    let mut only_in_first:Vec<&str> = vec![]; 
    let mut only_in_second:Vec<&str> = vec![];
    for word in words_in_first{
        if !second_string.contains(word) {
            only_in_first.push(word);
        }
    }
    for word in words_in_second{
        if !first_string.contains(word){
            only_in_second.push(word);
        }
    }
    let output = Differences {
        only_in_first: only_in_first,
        only_in_second: only_in_second
    };
    output
}

fn merge_names(first_name: &str, second_name: &str) -> String {
    let mut buffer = String::from("");
    let vowels= vec!['a','e','i','o','u'];
    let mut buffer = String::from("");
    let vowels= vec!['a','e','i','o','u'];
    //copy variables
    let mut first_name_copy = &first_name[0..];
    let mut second_name_copy = &second_name[0..];
    let mut second_vowel:char ='1';
    //checking to see if the first letter of the word is a vowel
    match first_name_copy.chars().next(){
        Some(char) =>{
            if vowels.contains(&char) {
                buffer.push(char);
                first_name_copy = &first_name_copy[1..];
            }
        }
        None => {
            //if the first string is empty return second word
            buffer = second_name_copy.to_string();
            return buffer;
        },
    };
    match second_name_copy.chars().next(){
        Some(char) =>{
            if vowels.contains(&char) {
                second_vowel = char;
                second_name_copy = &second_name_copy[1..];
            }
        }
        None => {
            //if the first string is empty return second word
            buffer = first_name_copy.to_string();
            return buffer;
        },
    };
    //while either char.next is a vali
    for char in first_name_copy.chars(){
        //if the next char is not a vowel, add it to the buffer
        if !vowels.contains(&char) {
            buffer.push(char);
        }
        //once you hit a vowel - add the previously hit vowel
        else{
            if second_vowel.is_alphabetic() {
                buffer.push(second_vowel);
            }
            let mut count = 0;
            for second_char in second_name_copy.chars(){
                if !vowels.contains(&second_char) {
                    buffer.push(second_char);
                    count +=1;
                }
                else {
                    buffer.push(char);
                    second_vowel = second_char;
                    second_name_copy = &second_name_copy[count+1..];
                    break;
                }
            }
        }
    };
    if (second_vowel.is_alphabetic()){
        buffer.push(second_vowel);
    }
    buffer.push_str(second_name_copy);
    return buffer;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_string() {
        // First, make sure the lifetimes were correctly marked
        let matches;
        let string_to_split = String::from("Hello, World!");

        {
            let delimeter = String::from(", ");
            matches = split_string(&string_to_split, &delimeter);
        }
        println!("Matches can be printed! See: {:?}", matches);

        // Now check the split logic
        assert_eq!(split_string(&"", &""), Vec::<&str>::new());
        assert_eq!(
            split_string(&"Hello, World!", &", "),
            vec!["Hello", "World!"]
        );
        assert_eq!(
            split_string(
                &"I this think this that this sentence this is this very this confusing this ",
                &" this "
            ),
            vec!["I", "think", "that", "sentence", "is", "very", "confusing"]
        );
        assert_eq!(
            split_string(&"apple🍎banana🍎orange", &"🍎"),
            vec!["apple", "banana", "orange"]
        );
        assert_eq!(
            split_string(
                &"Ayush;put|a,lot~of`random;delimeters|in|this,sentence",
                &";"
            ),
            vec![
                "Ayush",
                "put|a,lot~of`random",
                "delimeters|in|this,sentence"
            ]
        );
    }

    #[test]
    fn test_find_differences() {
        assert_eq!(
            find_differences(&"", &""),
            Differences {
                only_in_first: Vec::new(),
                only_in_second: Vec::new()
            }
        );
        assert_eq!(
            find_differences(&"pineapple pen", &"apple"),
            Differences {
                only_in_first: vec!["pineapple", "pen"],
                only_in_second: Vec::new()
            }
        );
        assert_eq!(
            find_differences(
                &"Sally sold seashells at the seashore",
                &"Seashells seashells at the seashore"
            ),
            Differences {
                only_in_first: vec!["Sally", "sold"],
                only_in_second: vec!["Seashells"]
            }
        );
        assert_eq!(
            find_differences(
                "How much wood could a wood chuck chuck",
                "If a wood chuck could chuck wood"
            ),
            Differences {
                only_in_first: vec!["How", "much"],
                only_in_second: vec!["If"]
            }
        );
        assert_eq!(
            find_differences(
                &"How much ground would a groundhog hog",
                &"If a groundhog could hog ground"
            ),
            Differences {
                only_in_first: vec!["How", "much", "would"],
                only_in_second: vec!["If", "could"]
            }
        );
    }

    #[test]
    fn test_merge_names() {
        assert_eq!(merge_names(&"alex", &"jake"), "aljexake");
        assert_eq!(merge_names(&"steven", &"stephen"), "ststevephenen");
        assert_eq!(merge_names(&"gym", &"rhythm"), "gymrhythm");
        assert_eq!(merge_names(&"walter", &"gibraltor"), "wgaltibreraltor");
        assert_eq!(merge_names(&"baker", &"quaker"), "bqakueraker");
        assert_eq!(merge_names(&"", &""), "");
        assert_eq!(merge_names(&"samesies", &"samesies"), "ssamamesesiieses");
        assert_eq!(merge_names(&"heather", &"meagan"), "hmeeathageran");
        assert_eq!(merge_names(&"panda", &"turtle"), "ptandurtlae");
        assert_eq!(merge_names(&"hot", &"sauce"), "hsotauce");
        assert_eq!(merge_names(&"", &"second"), "second");
        assert_eq!(merge_names(&"first", &""), "first");
    }
}
