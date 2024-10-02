fn longest_equal_sequence_prescriptive<T:PartialEq>(sequence:&[T]) -> i32 {
    let mut greatest_count = 0;
    let mut current_count = 0;
    if sequence.len() ==0 {
        return 0
    }
    let mut lastelement = &sequence[0];
    for item in sequence{
        println!("greatest:{}, current:{}",greatest_count,current_count);
        if *item == *lastelement{
            current_count +=1;
        }
        else{
            lastelement = item;
            if current_count > greatest_count {
                greatest_count = current_count;
            }
            current_count = 1;
        }
    }
    if current_count > greatest_count {
        greatest_count = current_count;
    }
    greatest_count
}

fn longest_equal_sequence_functional<T:PartialEq>(sequence:&[T]) -> i32 {
    if sequence.len() ==0 {
        return 0
    }
    let output = sequence.iter().zip(sequence.iter()
    .skip(1))
    .fold((1,1), |(mut current,mut greatest), a| 
        if a.0 == a.1{
            current = current +1;
            if(current > greatest){
                greatest = current
            }
            (current,greatest)
        }
        else{
            (1,greatest)
        }
    );
    output.1
}

fn is_valid_paranthesis(paranthesis: &str) -> bool {
    let pairs = ["{","}","(",")","[","]"];
    let mut i = 0;
    while (i < pairs.len()-1){
        //this code checks for a single set of parenthises
        let opening: Vec<_> = paranthesis.match_indices(pairs[i]).collect();
        let closing: Vec<_> = paranthesis.match_indices(pairs[i+1]).collect(); 
        //if we have an equivilant amount of opening and closing brackets, check to make sure they align
        if (opening.len() == closing.len() && opening.len() != 0) {
            let mut j = 0;
            let len = opening.len();
            let mut left = opening.iter();
            let mut right = closing.iter();
            while(j < len) {
                match left.next(){
                    Some((num1,_)) =>{
                        match right.next(){
                            Some((num2,_)) =>{
                                if(num2>num1){
                                    //if this set of two match, then move on to the next one
                                    j +=2;
                                }
                                else {
 //                                   println!("failed because a {} was mismatched",i);
                                    return false
                                }
                            }
                            None =>break,
                        }
                    }
                    None =>break,
                }
            };
            //if all of these cases worked, move on to the next characters
            i+=2;
        }
        //if we have an non equal amount of parenthesis it is false
        if opening.len() != closing.len() {
            // println!("failed because there are a non equal amount of {}",i);
            // println!("opening len {:?} closing len{:?}",opening,closing);
            return  false;
        }
        //if there are zero then move on to the next set of characters
        if (opening.len() == closing.len() && opening.len() ==0){
            i+=2;
        }
    }
    return true;
}

fn longest_common_substring<'a>(first_str: &'a str, second_str: &'a str) -> &'a str {
    //create a vector of all substrings of the first string
    let mut substrings:Vec<&str> = vec![];
    let mut starting = 0;
    while starting < first_str.len(){
        let mut ending = first_str.len();
        while ending >= starting +1{
            substrings.push(&first_str[starting..ending]);
            ending -=1;
        }
        starting +=1;
    }
//    println!("{:?}",substrings);
    //check all to see if they were present, once all of the present ones are found
    let valid:Vec<&&str> = substrings.iter().filter(|string| second_str.contains(*string)).collect();
    let mut i = 0;
    let mut output:&str = ""; 
    for string in valid{
        if string.len() > i{
            i = string.len();
            output = string;
        }
    }
    println!("{:?}", output);
    return output
}

fn longest_common_substring_multiple<'a>(strings: &'a[&str]) -> &'a str {
    let first_str = strings[0];
    let mut substrings:Vec<&str> = vec![];
    let mut starting = 0;
    while starting < first_str.len(){
        let mut ending = first_str.len();
        while ending >= starting +1{
            substrings.push(&first_str[starting..ending]);
            ending -=1;
        }
        starting +=1;
    }
    //Check to see which substring that is in the
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_longest_equal_sequence_prescriptive() {
        assert_eq!(longest_equal_sequence_prescriptive(&vec![1, 1, 1, 1, 1]), 5);
        assert_eq!(
            longest_equal_sequence_prescriptive(&vec![1.0, 2.0, 2.0, 2.0, 3.0, 4.0, 4.0]),
            3
        );
        assert_eq!(longest_equal_sequence_prescriptive(&vec![-100]), 1);
        let empty_vec: Vec<char> = Vec::new();
        assert_eq!(longest_equal_sequence_prescriptive(&empty_vec), 0);
        assert_eq!(
            longest_equal_sequence_prescriptive(&vec![
                1000, 1000, 2000, 2000, 2000, 3000, 3000, 3000, 3000
            ]),
            4
        );
        assert_eq!(
            longest_equal_sequence_prescriptive(&vec!['a', 'b', 'a', 'b', 'a', 'b']),
            1
        );
        let vec: Vec<u8> = vec![5, 5, 5, 1, 2, 3];
        assert_eq!(longest_equal_sequence_prescriptive(&vec), 3);
        assert_eq!(
            longest_equal_sequence_prescriptive(&vec![1, 2, 3, 4, 4, 4]),
            3
        );
        assert_eq!(longest_equal_sequence_prescriptive(&vec![1, 2, 3, 4, 5]), 1);
        assert_eq!(
            longest_equal_sequence_prescriptive(&vec![1, 1, 2, 2, 2, 3, 1, 1, 1, 1, 1]),
            5
        );
    }
   #[test]
    fn test_longest_equal_sequence_functional() {
        assert_eq!(longest_equal_sequence_functional(&vec![1, 1, 1, 1, 1]), 5);
        assert_eq!(
            longest_equal_sequence_functional(&vec![1.0, 2.0, 2.0, 2.0, 3.0, 4.0, 4.0]),
            3
        );
        assert_eq!(longest_equal_sequence_functional(&vec![-100]), 1);
        let empty_vec: Vec<char> = Vec::new();
        assert_eq!(longest_equal_sequence_functional(&empty_vec), 0);
        assert_eq!(
            longest_equal_sequence_functional(&vec![
                1000, 1000, 2000, 2000, 2000, 3000, 3000, 3000, 3000
            ]),
            4
        );
        assert_eq!(
            longest_equal_sequence_functional(&vec!['a', 'b', 'a', 'b', 'a', 'b']),
            1
        );
        let vec: Vec<u8> = vec![5, 5, 5, 1, 2, 3];
        assert_eq!(longest_equal_sequence_functional(&vec), 3);
        assert_eq!(
            longest_equal_sequence_functional(&vec![1, 2, 3, 4, 4, 4]),
            3
        );
        assert_eq!(longest_equal_sequence_functional(&vec![1, 2, 3, 4, 5]), 1);
        assert_eq!(
            longest_equal_sequence_functional(&vec![1, 1, 2, 2, 2, 3, 1, 1, 1, 1, 1]),
            5
        );
    }

    #[test]
    fn test_is_valid_paranthesis() {
        assert_eq!(is_valid_paranthesis(&String::from("{}")), true);
        assert_eq!(is_valid_paranthesis(&String::from("()")), true);
        assert_eq!(is_valid_paranthesis(&String::from("()[]{}")), true);
        assert_eq!(is_valid_paranthesis(&String::from("({[]})")), true);
        assert_eq!(is_valid_paranthesis(&String::from("([]){}{}([]){}")), true);
        assert_eq!(is_valid_paranthesis(&String::from("()(")), false);
        assert_eq!(is_valid_paranthesis(&String::from("(()")), false);
        assert_eq!(is_valid_paranthesis(&String::from("([)]{[})")), false);
        assert_eq!(
            is_valid_paranthesis(&String::from("({[()]}){[([)]}")),
            false
        );
        assert_eq!(
            is_valid_paranthesis(&String::from("()[]{}(([])){[()]}(")),
            false
        );
    }

    #[test]
    fn test_common_substring() {
        assert_eq!(longest_common_substring(&"abcdefg", &"bcdef"), "bcdef");
        assert_eq!(longest_common_substring(&"apple", &"pineapple"), "apple");
        assert_eq!(longest_common_substring(&"dog", &"cat"), "");
        assert_eq!(longest_common_substring(&"racecar", &"racecar"), "racecar");
        assert_eq!(longest_common_substring(&"ababc", &"babca"), "babc");
        assert_eq!(longest_common_substring(&"xyzabcxyz", &"abc"), "abc");
        assert_eq!(longest_common_substring(&"", &"abc"), "");
        assert_eq!(longest_common_substring(&"abcdefgh", &"defghijk"), "defgh");
        assert_eq!(longest_common_substring(&"xyabcz", &"abcxy"), "abc");
        assert_eq!(longest_common_substring(&"ABCDEFG", &"abcdefg"), "");
        assert_eq!(
            longest_common_substring(
                &"thisisaverylongstringwithacommonsubstring",
                &"anotherlongstringwithacommonsubstring"
            ),
            "longstringwithacommonsubstring"
        );
        assert_eq!(longest_common_substring("a", "a"), "a");
    }

//     #[test]
//     fn test_common_substring_multiple() {
//         assert_eq!(
//             longest_common_substring_multiple(&vec!["abcdefg", "cdef"]),
//             "cdef"
//         );
//         assert_eq!(
//             longest_common_substring_multiple(&vec!["apple", "pineapple", "maple", "snapple"]),
//             "ple"
//         );
//         assert_eq!(
//             longest_common_substring_multiple(&vec!["dog", "cat", "fish"]),
//             ""
//         );
//         assert_eq!(
//             longest_common_substring_multiple(&vec!["racecar", "car", "scar"]),
//             "car"
//         );
//         assert_eq!(
//             longest_common_substring_multiple(&vec!["ababc", "babca", "abcab"]),
//             "abc"
//         );
//         assert_eq!(
//             longest_common_substring_multiple(&vec!["xyzabcxyz", "abc", "zabcy", "abc"]),
//             "abc"
//         );
//         assert_eq!(
//             longest_common_substring_multiple(&vec!["", "abc", "def"]),
//             ""
//         );
//         assert_eq!(
//             longest_common_substring_multiple(&vec![
//                 "abcdefgh",
//                 "bcd",
//                 "bcdtravels",
//                 "abcs",
//                 "webcam"
//             ]),
//             "bc"
//         );
//         assert_eq!(
//             longest_common_substring_multiple(&vec!["identical", "identical", "identical"]),
//             "identical"
//         );
//         assert_eq!(
//             longest_common_substring_multiple(&vec!["xyabcz", "abcxy", "zabc"]),
//             "abc"
//         );
//         assert_eq!(longest_common_substring_multiple(&vec!["a", "a", "a"]), "a");
//         assert_eq!(
//             longest_common_substring_multiple(&vec![
//                 "thisisaverylongstringwiththecommonsubstring",
//                 "anotherlongstringwithacommonsubstring",
//                 "yetanotherstringthatcontainsacommonsubstring",
//             ]),
//             "commonsubstring",
//         );
//     }
 }
