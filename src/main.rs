mod model_json; // this is the same as `mod model_json; pub use model_json::*;`

mod json_to_csv {
    //use serde_json::Value;
    
    use std::{
        fs::File,
        io::{self, BufRead, BufReader, BufWriter, Write},
    };

    pub fn parse_args(
        args: Vec<String>,
    ) -> Result<(Box<dyn BufRead + 'static>, Box<dyn Write + 'static>), String> {
        let mut input_file = false;
        let mut output_file = false;
        let mut input = String::new();
        let mut output = String::new();
        for i in 0..args.len() {
            if args[i] == "-i" {
                input_file = true;
                input = args[i + 1].clone();
            }
            if args[i] == "-o" {
                output_file = true;
                output = args[i + 1].clone();
            }
        }

        if input_file {
            match File::open(&input) {
                Ok(file) => {
                    println!("Input file: {}", input);
                    let input_reader = Box::new(BufReader::new(file));
                    if output_file {
                        match File::create(&output) {
                            Ok(file) => {
                                println!("Output file: {}", output);
                                let output_writer = Box::new(BufWriter::new(file));
                                return Ok((input_reader, output_writer));
                            }
                            Err(e) => {
                                return Err(format!(
                                    "Error creating output file '{}': {}",
                                    output, e
                                ));
                            }
                        }
                    } else {
                        println!("Output file: stdout");
                        let output_writer = Box::new(BufWriter::new(io::stdout()));
                        return Ok((input_reader, output_writer));
                    }
                }
                Err(e) => {
                    return Err(format!(
                        "Error opening input file (cwd: {}, input: '{}'): {}",
                        std::env::current_dir().unwrap().display(),
                        input,
                        e
                    ));
                }
            }
        } else {
            println!("Input file: stdin");
            let input_reader = Box::new(BufReader::new(io::stdin()));
            if output_file {
                match File::create(&output) {
                    Ok(file) => {
                        println!("Output file: {}", output);
                        let output_writer = Box::new(BufWriter::new(file));
                        return Ok((input_reader, output_writer));
                    }
                    Err(e) => {
                        return Err(format!("Error creating output file: {}", e));
                    }
                }
            } else {
                println!("Output file: stdout");
                let output_writer = Box::new(BufWriter::new(io::stdout()));
                return Ok((input_reader, output_writer));
            }
        }
    }

    #[test]
    fn test_parse_args() {
        // Test with input file and output file
        let args = vec![
            String::from("-i"),
            String::from("tests/input.json"),
            String::from("-o"),
            String::from("/dev/shm/output.csv"),
        ];
        match parse_args(args) {
            Ok((mut input, mut output)) => {
                // clean up and close
                output.flush().unwrap();
            }
            Err(e) => {
                panic!("{}", e);
            }
        }

        // read test JSON files and attempt to deserialize it
        let args = vec![String::from("-i"), String::from("tests/input.json")];
        match parse_args(args) {
            Ok((mut input, mut output)) => {
                // deserialize - from_reader() method needs to access io::Read::bytes() method

                //// For now, read the whol buffer into memory and pass that on
                //// allocate buffer of 256MB
                //let mut buf = Vec::new();
                //let _read_count = input.read_to_end(&mut buf).unwrap(); // read the whole file into the buffer
                //let str_buf = std::str::from_utf8(&buf).unwrap();
                //println!("Buffer read_count: {}", _read_count);
                //for buf_index in 0.._read_count {
                //    // print byte-by-byte instead of str_buf in case data is bogus
                //    let ch = buf[buf_index];
                //    if ch == 0 {
                //        print!(".");
                //    } else {
                //        print!("{}", buf[buf_index] as char);
                //    }
                //}
                //println!("\n########################################################");
                //let bookmark_folders: BookmarkRootFolder = serde_json::from_str(str_buf).unwrap();
                let bookmark_folders: BookmarkRootFolder = serde_json::from_reader(input).unwrap();

                // for test, just recursively traverse down each children and print the title and lastModified and the type
                fn traverse_children(children: &Vec<BookmarkNodes>) {
                    for child in children {
                        println!(
                            "title: {}, lastModified: {}, uri: {:#?}",
                            child.title(),
                            child.last_modified(),
                            child.uri()
                        );
                        if let Some(children) = &child.children() {
                            traverse_children(children);
                        }
                    }
                }
                traverse_children(bookmark_folders.children());

                // clean up and close
                output.flush().unwrap();
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // read in JSON either from stdin or file
    let (input_reader, output_writer) = match json_to_csv::parse_args(args) {
        Ok((input_reader, output_writer)) => (input_reader, output_writer),
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    // read in JSON and deserialize it as Bookmark structure
    let bookmark_folders: Result<model_json::model_json::BookmarkRootFolder, _> =
        serde_json::from_reader(input_reader);
    let bookmarks: Vec<model_json::model_json::BookmarkNodes> = match bookmark_folders {
        Ok(bookmark_folders) => {
            // recursively visit each child and return Some tuple if it is bookmark, else return None for containers and separators
            fn traverse_children(
                children: &Vec<model_json::model_json::BookmarkNodes>,
            ) -> Vec<model_json::model_json::BookmarkNodes> {
                let mut bookmarks: Vec<model_json::model_json::BookmarkNodes> = Vec::new();
                for child in children {
                    if child.is_bookmark() {
                        bookmarks.push(child.clone());
                    } else if let Some(children) = &child.children() {
                        bookmarks.append(&mut traverse_children(children));
                    }
                }
                bookmarks
            }
            traverse_children(bookmark_folders.children())
        }
        Err(e) => {
            println!("Error deserializing JSON: {}", e);
            return;
        }
    };

    // now that we've got it as data-model, we will just travese down each child and print out the title, URI, and last modified date, sorted by last modified date
    let mut bookmarks_sorted: Vec<model_json::model_json::BookmarkNodes> = bookmarks.clone();
    bookmarks_sorted.sort_by(|a, b| a.last_modified().cmp(&b.last_modified()));
    // CSV output, we're assuming that by here, only the "places" nodes are left, so we can just print them out in CSV format
    // either to the stdout or to the output file stream
    let mut csv_writer = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always)   // just easier to just quote everything including numbers
        .from_writer(output_writer);
    for bookmark in bookmarks_sorted {
        // convert the last_modified i64 to datetime - last_modified is encoded as unix epoch time in microseconds
        let last_modified = chrono::NaiveDateTime::from_timestamp_opt(
            bookmark.last_modified() / 1_000_000,
            (bookmark.last_modified() % 1_000_000) as u32,
        )
        .unwrap();
        // output: "title","uri","chapter","last_modified","notes","tags"
        let mut record = csv::StringRecord::new();
        record.push_field(&bookmark.title());
        record.push_field(&bookmark.uri());
        record.push_field("0");
        record.push_field(&last_modified.format("%Y-%m-%dT%H:%M:%S").to_string());
        record.push_field("-");
        record.push_field("#");
        csv_writer.write_record(&record).unwrap();
    }
}
