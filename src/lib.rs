//mod firefox_bookmarks_to_csv;
mod model_csv_manga;
mod model_json_mozilla_bookmarks;
mod model_manga;
mod model_sqlite3_manga;
mod text_type; // used by model_manga to make it flexible for different text types

pub mod my_libs {
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

    // turns out there are few punctuation marks that we'd have problems in CSV or SQLite:
    // * `,` (comma) - CSV will treat it as a new column, and SQLite will treat it as a new column
    // * `"` (double quote) - CSV will treat it as a string delimiter, so strings such as "Notes: I don't know" will be treated as "Notes: I "
    // * `'` (single quote) - SQLite will treat it as a string delimiter, so strings such as "Notes: I don't know" will be treated as "Notes: I "
    // * ``` (ticks) - BASH scripts hates this, so we need to replace it with something else
    // | sed 's/,/、/g' | sed 's/"/’/g' | sed "s/'/’/g"
    pub fn sanitize_string<T: AsRef<str>>(s: T) -> String {
        // first, trim the edges of the quotes, if any, BECAUSE we want to only sanitize the string INSIDE the quotes
        let s = trim_quotes(s);
        let s = s.replace(",", "、");
        let s = s.replace("'", "’");
        let s = s.replace("\"", "’");
        let s = s.replace("`", "’");
        s
    }

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

    // For CSV row-sets, when a cell contains a comma, EVEN IF IT IS INSIDE A QUOTED STRING, it will be treated as a new column
    // on some of the CSV tools and libraries, so we need to replace all commas with something else, such as "、" (UTF8)
    pub fn fix_comma_in_string(s: &str) -> String {
        // NOTE: cannot have commmas inside strings for MOST CSV utilities fails to know the differences...
        // so, we need to replace all commas with something else, such as "、"
        s.replace(",", "、")
    }

    pub fn from_epoch_to_str(epoch: i64) -> String {
        // convert the last_update i64 to datetime - last_update is encoded as unix epoch time in microseconds
        let from_epoch_timespan = chrono::NaiveDateTime::from_timestamp_opt(
            epoch / 1_000_000,
            (epoch % 1_000_000) as u32,
        )
        .unwrap();
        let last_update_yyyymmdd_thhmmss =
            from_epoch_timespan.format("%Y-%m-%dT%H:%M:%S").to_string(); // have to call to_string() to format
        last_update_yyyymmdd_thhmmss // and then convert it back to &str
    }
    pub fn str_to_epoch_millis(time_yyyymmdd_thhmmss: String) -> i64 {
        // convert the last_update i64 to datetime - last_update is encoded as unix epoch time in microseconds
        let timespan_yyyymmdd_thhmmss =
            chrono::NaiveDateTime::parse_from_str(&time_yyyymmdd_thhmmss, "%Y-%m-%dT%H:%M:%S")
                .unwrap()
                .timestamp_millis();
        timespan_yyyymmdd_thhmmss
    }
    pub fn str_to_epoch_micros(time_yyyymmdd_thhmmss: String) -> i64 {
        // convert the last_update i64 to datetime - last_update is encoded as unix epoch time in microseconds
        let timespan_yyyymmdd_thhmmss =
            chrono::NaiveDateTime::parse_from_str(&time_yyyymmdd_thhmmss, "%Y-%m-%dT%H:%M:%S")
                .unwrap()
                .timestamp_micros();
        timespan_yyyymmdd_thhmmss
    }
    // format a chrono::DateTime<chrono::Utc> into a String in the format of "YYYY-MM-DDTHH:MM:SS"
    pub fn datetime_to_string(datetime: &chrono::DateTime<chrono::Utc>) -> String {
        datetime.format("%Y-%m-%dT%H:%M:%S").to_string()
    }
    pub fn string_datetime_to_epoch_millis(datetime: &str) -> i64 {
        // Note that rfc3339() is formatted as "YYYY-MM-DDTHH:MM:SS+00:00" (with timezone offset)
        // like so: `1996-12-19T16:39:57-08:00` but our format is "YYYY-MM-DDTHH:MM:SS" (no timezone offset)
        // so we assume it is GMT/UTC timezone and append "+00:00" to it:
        let datetime = format!("{}+00:00", datetime);
        let datetime_fixed_offset =
            chrono::DateTime::parse_from_rfc3339(datetime.as_str()).unwrap();
        datetime_fixed_offset.timestamp_millis()
    }

    #[cfg(test)]
    mod tests {
        #[allow(dead_code, unused_variables)]
        use crate::my_libs::{make_none_if_empty, trim_quotes, Flattener};

        #[test]
        #[allow(dead_code, unused_variables)]
        fn test_flatten_vec_option() {
            let input = vec![Some(1), None, Some(2), None, Some(3)];
            let expected_output = vec![1, 2, 3];
            assert_eq!(input.flatten(), expected_output);
        }

        #[test]
        #[allow(dead_code, unused_variables)]
        fn test_flatten_vec_vec() {
            let input_as_slices = vec![vec![1, 2], vec![3], vec![], vec![4, 5, 6]];
            let expected_output = vec![1, 2, 3, 4, 5, 6];
            // the trivial Vec::concat() way...  Note, in future, when stablized, we can use slice_flatten() here, and ro arrays, Vec.into_flatten()
            assert_eq!(input_as_slices.concat(), expected_output);

            //// Create a vector of arrays of strings
            let vec = vec![["foo", "bar"], ["baz", "qux"], ["quux", "corge"]];

            // Flatten the vector into a single vector of strings
            //let flattened = vec.into_flattened();

            // The flattened vector should contain all the strings from the original vector
            //assert_eq!(flattened, vec!["foo", "bar", "baz", "qux", "quux", "corge"]);
        }

        #[test]
        #[allow(dead_code, unused_variables)]
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
        #[allow(dead_code, unused_variables)]
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

// Following mod is PRIVATE, it's just some notes for myself of new things I learned in Rust
// during the progress of THIS project
mod my_learnt_rust {

    // example of generic lifetime annotations (borrow checker)
    // sample assures lifetime of return value is same as lifetime of input parameters
    #[allow(dead_code, unused_variables)]
    fn lifetime_examples<'input_output_lifetime>(
        x: &'input_output_lifetime str,
        y: &'input_output_lifetime str,
    ) -> &'input_output_lifetime str {
        if x.is_empty() {
            x
        } else if y.is_empty() {
            y
        } else {
            if x.len() > y.len() {
                x
            } else {
                y
            }
        }
    }

    // Some insane syntax of Rust:
    // 2 pattern-matching methods below: method_match() and method_if_let() are equivalent (match vs if/else):
    #[allow(dead_code)]
    fn method_match() {
        let print_number = |n: &i32| println!("The number is {}", n);
        let my_number = Some(42);
        match my_number {
            Some(ref n) => print_number(n),
            None => println!("The number is None"),
        }
    }

    #[allow(dead_code)]
    fn method_if_let() {
        let print_number = |n: &i32| println!("The number is {}", n);
        let my_number = None;
        // NOTE: the 'let' statement below is '=', not '=='
        if let Some(ref n) = my_number {
            print_number(n);
        } else {
            println!("The number is None");
        }
    }

    // example of passing as mutable reference vesu passed by value and marked as mutable
    #[allow(dead_code, unused_variables)]
    fn prune_romaji_map_by_mutable_ref<'a>(
        romaji_title_map_mut: &mut std::collections::HashMap<String, Vec<i32>>,
        url_map_mut: std::collections::HashMap<String, Vec<i32>>,
        merged_duplicates_map: &mut Vec<i32>,
    ) -> Vec<i32> {
        todo!("")
    }
    #[allow(dead_code, unused_variables, unused_mut)]
    fn prune_romaji_map_by_value_as_mutable<'a>(
        mut romaji_title_map_mut: std::collections::HashMap<String, Vec<i32>>,
        url_map_mut: std::collections::HashMap<String, Vec<i32>>,
        mut merged_duplicates_map: Vec<i32>,
    ) -> Vec<i32> {
        todo!("")
    }
}
