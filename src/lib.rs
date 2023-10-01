mod model_csv_manga;
mod model_json_mozilla_bookmarks;
mod model_manga;
mod model_sqlite3_manga;
mod text_type; // used by model_manga to make it flexible for different text types

pub mod my_utils {
    pub trait Flattener<T: Clone> {
        fn flatten(&self) -> Vec<T>;
    }

    impl<T: Clone> Flattener<T> for Vec<Option<T>> {
        fn flatten(&self) -> Vec<T> {
            self.iter()
                .filter_map(|opt| opt.as_ref())
                .cloned()
                .collect()
        }
    }

    // See Vec::into_flatten() in Rust 1.42.0, and slice_flatten() in Rust 1.43.0
    // https://doc.rust-lang.org/std/vec/struct.Vec.html#method.into_flatten
    // https://doc.rust-lang.org/std/primitive.slice.html#method.flatten
    // Also, the one I was mostly inteested in, you can use concat() on Vec<Vec<T>> to flatten it
    //impl<T: Clone> Flattener<T> for Vec<Vec<T>> {
    //    fn flatten(&self) -> Vec<T> {
    //        self.iter()
    //            .flat_map(|inner_vec| inner_vec.iter())
    //            .cloned()
    //            .collect()
    //    }
    //}

    // NOTE: There is already an Option::flatten() in Rust 1.40.0
    //impl<T: Clone> Flattener<T> for Option<Option<T>> {
    //    fn flatten(&self) -> Vec<T> {
    //        match self {
    //            Some(inner_opt) => inner_opt.iter().cloned().collect(),
    //            None => Vec::new(),
    //        }
    //    }
    //}

    // Allow both String and &str to be passed in with magic of AsRef<T> and s.as_ref() combination
    pub fn trim_quotes<T: AsRef<str>>(s: T) -> String {
        let s = s.as_ref().trim().trim_end_matches('"').to_string();
        if s.starts_with('"') || s.ends_with('"') || s.starts_with(' ') || s.ends_with(' ') {
            trim_quotes(&s[1..s.len() - 1])
        } else {
            s.to_string()
        }
    }
    // turn Some("") into None - NOTE: We do NOT want to return `Option<&'static str>` static lifetime, so we return Option<String>
    pub fn make_none_if_empty<T: AsRef<str>>(s: Option<T>) -> Option<String> {
        // no need to transform 's' since trim_quotes() will do it for us
        match s.as_ref() {
            Some(s) => match trim_quotes(s).is_empty() {
                false => Some(trim_quotes(s)),
                true => None,
            },
            None => None,
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::my_utils::{make_none_if_empty, trim_quotes, Flattener};

        #[test]
        fn test_flatten_vec_option() {
            let input = vec![Some(1), None, Some(2), None, Some(3)];
            let expected_output = vec![1, 2, 3];
            assert_eq!(input.flatten(), expected_output);
        }

        #[test]
        fn test_flatten_vec_vec() {
            let input_as_slices = vec![vec![1, 2], vec![3], vec![], vec![4, 5, 6]];
            let expected_output = vec![1, 2, 3, 4, 5, 6];
            // the trivial Vec::concat() way...  Note, in future, when stablized, we can use slice_flatten() here, and ro arrays, Vec.into_flatten()  
            assert_eq!(input_as_slices.concat(), expected_output);

            //// Create a vector of arrays of strings
            let mut vec = vec![["foo", "bar"], ["baz", "qux"], ["quux", "corge"]];

            // Flatten the vector into a single vector of strings
            //let flattened = vec.into_flattened();

            // The flattened vector should contain all the strings from the original vector
            //assert_eq!(flattened, vec!["foo", "bar", "baz", "qux", "quux", "corge"]);
        }

        #[test]
        fn test_trim_quotes() {
            assert_eq!(trim_quotes(""), "");
            assert_eq!(trim_quotes(" "), "");
            assert_eq!(trim_quotes(" \" "), "");
            assert_eq!(trim_quotes(" \"    \" "), "");
            assert_eq!(trim_quotes(" \" x     \" "), "x");
            assert_eq!(trim_quotes(" \" x     \" "), "x");

            // tests to make sure it can accept both String and &str
            let s1 = "  \"Hello\"  ";
            let s2 = String::from("  \"World\"  ");
            let trimmed1 = trim_quotes(s1);
            let trimmed2 = trim_quotes(s2);
            println!("{}", trimmed1); // prints "Hello"
            println!("{}", trimmed2); // prints "World"
        }
        fn test_make_none() {
            assert_eq!(make_none_if_empty(Some("")), None);
            assert_eq!(make_none_if_empty(Some(" ")), None);
            assert_eq!(make_none_if_empty(Some(" \" ")), None);
            assert_eq!(make_none_if_empty(Some(" \"    \" ")), None);
            assert_eq!(
                make_none_if_empty(Some(" \" x     \" ")),
                Some("x".to_string())
            );
            let s1 = Some("  \"\"  ");
            let s2 = Some(String::from("  \"Hello\"  "));
            let none1 = make_none_if_empty(s1.as_deref());
            let none2 = make_none_if_empty(s2.as_deref());
            println!("{:?}", none1); // prints "None"
            println!("{:?}", none2); // prints "Some(\"Hello\")"
        }
    }
}

// Some insane syntax of Rust:
// method_match() and method_if_let() are equivalent:
//      fn method_match() {
//          let my_number = Some(42);
//          match my_number {
//              Some(ref n) => print_number(n),
//              None => println!("The number is None"),
//          }
//      }
//      fn method_if_let() {
//          let my_number = None;
//          if let Some(ref n) = my_number {
//              print_number(n);
//          } else {
//              println!("The number is None");
//          }
//      }
