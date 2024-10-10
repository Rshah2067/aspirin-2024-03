use regex::{Regex, RegexBuilder};
pub trait Searching {
    fn search<'a>(
        &self,
        input: &'a Vec<String>,
        invert_match: bool,
        ignore_case: bool,
    ) -> Vec<&'a String>;
}
pub struct regex_search<'a> {
    needle: &'a str,
}
pub struct string_search<'a> {
    needle: &'a str,
}
//constructors
impl<'a> regex_search<'a> {
    pub fn new(needle: &'a str) -> Self {
        let output = regex_search { needle: needle };
        output
    }
}
impl<'a> string_search<'a> {
    pub fn new(needle: &'a str) -> Self {
        let output = string_search { needle: needle };
        output
    }
}
impl<'a> Searching for regex_search<'a> {
    fn search<'b>(
        &self,
        input: &'b Vec<String>,
        invert_match: bool,
        ignore_case: bool,
    ) -> Vec<&'b String> {
        let mut output: Vec<&String> = Vec::new();
        if !ignore_case && !invert_match {
            //no flags
            let re = Regex::new(&self.needle).unwrap();
            output = input.iter().filter(|a| re.is_match(a)).collect();
        } else if !ignore_case && invert_match {
            //invert output
            let re = Regex::new(&self.needle).unwrap();
            output = input.iter().filter(|a| !re.is_match(a)).collect();
        } else if ignore_case && !invert_match {
            //ignore case
            let re = RegexBuilder::new(&self.needle)
                .case_insensitive(true)
                .build()
                .unwrap();
            output = input.iter().filter(|a| re.is_match(a)).collect();
        } else if ignore_case && invert_match {
            let re = RegexBuilder::new(&self.needle)
                .case_insensitive(true)
                .build()
                .unwrap();
            output = input.iter().filter(|a| !re.is_match(a)).collect();
        }
        output
    }
}
impl<'a> Searching for string_search<'a> {
    fn search<'b>(
        &self,
        input: &'b Vec<String>,
        invert_match: bool,
        ignore_case: bool,
    ) -> Vec<&'b String> {
        let mut output: Vec<&String> = Vec::new();
        if !ignore_case && !invert_match {
            //no flags
            output = input.iter().filter(|a| a.contains(self.needle)).collect();
        } else if !ignore_case && invert_match {
            //invert output
            output = input.iter().filter(|a| !a.contains(self.needle)).collect();
        } else if ignore_case && !invert_match {
            //ignore case
            output = input
                .iter()
                .filter(|a| {
                    a.to_ascii_lowercase()
                        .contains(&self.needle.to_ascii_lowercase())
                })
                .collect();
        } else if ignore_case && invert_match {
            //ignore case and invert match
            output = input
                .iter()
                .filter(|a| {
                    !a.to_ascii_lowercase()
                        .contains(&self.needle.to_ascii_lowercase())
                })
                .collect();
        }
        output
    }
}
mod test {
    use super::*;
    #[test]
    fn test_search() {
        //create a mock string search
        let needle_regex = "test";
        let needle_string = "test";
        let haystack = vec![
            String::from("Test no no"),
            String::from("look test blank"),
            String::from("fun fun fun"),
        ];
        let string_searcher = string_search::new(&needle_string);
        let regex_searcher = regex_search::new(&needle_regex);
        //regular (no flags)
        assert!(
            vec![&String::from("look test blank")]
                == string_searcher.search(&haystack, false, false)
        );
        assert!(
            vec![&String::from("look test blank")]
                == regex_searcher.search(&haystack, false, false)
        );
        //inversion flag
        assert!(
            vec![&String::from("Test no no"), &String::from("fun fun fun")]
                == string_searcher.search(&haystack, true, false)
        );
        assert!(
            vec![&String::from("Test no no"), &String::from("fun fun fun")]
                == regex_searcher.search(&haystack, true, false)
        );
        //case insensitve flag
        assert!(
            vec![
                &String::from("Test no no"),
                &String::from("look test blank")
            ] == string_searcher.search(&haystack, false, true)
        );
        assert!(
            vec![
                &String::from("Test no no"),
                &String::from("look test blank")
            ] == regex_searcher.search(&haystack, false, true)
        );
        //both case insensitive and inversion
        assert!(
            vec![&String::from("fun fun fun")] == string_searcher.search(&haystack, true, true)
        );
        assert!(vec![&String::from("fun fun fun")] == regex_searcher.search(&haystack, true, true));
    }
}
