mod model_csv_manga;
mod model_json_mozilla_bookmarks;
mod model_manga; // this is the same as `mod model_json; pub use model_json::*;`
mod model_sqlite3_manga;

use std::io::{self, BufRead, BufReader, Write};

use firefox_bookmark_to_csv::my_libs;
use json_to_csv::upsert_db;

use model_json_mozilla_bookmarks::model_json_mozilla_bookmarks::{
    BookmarkNodes, BookmarkRootFolder,
};
use model_manga::model_manga::MangaModel;

mod json_to_csv {
    //use serde_json::Value;

    use std::{
        env,
        fs::File,
        io::{self, BufRead, BufReader, BufWriter, Write},
    };

    #[allow(unused_imports)]
    // following are actually used, but compiler is complaining that it's not used
    use crate::model_json_mozilla_bookmarks::model_json_mozilla_bookmarks::{
        BookmarkNodes, BookmarkRootFolder,
    };
    use crate::{model_csv_manga, model_manga::model_manga::MangaModel, model_sqlite3_manga};

    pub fn upsert_db(
        db_full_paths: &str,
        manga: &MangaModel,
        continue_on_error: bool,
        debug_flag: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        //let upsert_result = model_sqlite3_manga::model_sqlite3_manga::upsert_manga(
        //    db_full_paths,
        //    &manga, // need to clone so that we do not steal/borrow the ownership of possible_csv_row/result
        //);
        let insert_result = model_sqlite3_manga::model_sqlite3_manga::insert_manga(
            db_full_paths,
            &manga, // need to clone so that we do not steal/borrow the ownership of possible_csv_row/result
        );
        match insert_result {
            Ok(upsert_row_returned) => {
                // do nothing (for now) if successfully inserted
                if debug_flag {
                    println!("> inserted_row_model SUCCESS: {}", upsert_row_returned)
                }
            }
            Err(insert_or_update_error) => {
                // for now, panic!() if it was an error based on unique constraint (because it's a programmer bug rather than actual error)
                if insert_or_update_error
                    .to_string()
                    .contains("UNIQUE constraint failed")
                {
                    println!("\n");
                    panic!( "Programmer (logic) Error - UNIEQUE constraint should have ben handled elsewhere: Attempting to write CSV row\n>\t{:?}:\n>\t{}", manga, insert_or_update_error);
                }
                // if the error is based on unique constraint, UPSERT should have taken care of it, so
                // assume that it's some other error and print it out
                println!(
                    "ERROR upsert_db(): Error writing CSV row: {}\n\t{:#?}\n",
                    insert_or_update_error, manga,
                );

                if continue_on_error == false {
                    return Err(Box::new(insert_or_update_error));
                }
            }
        }
        Ok(())
    }

    // read existing CSV file and deserialize each row, we'll directly
    // pass/transfer it down to SQLite
    pub fn read_csv_and_update_sqlite(
        input_reader: Box<dyn std::io::Read>,
        db_full_paths: &str,
        debug_flag: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // deserialize - from_reader() method needs to access io::Read::bytes() method
        let mut csv_util = model_csv_manga::model_csv_manga::Utils::new(None, input_reader);
        // whether table already exists or not, we'll create it in case it does not exist
        let _table_created = model_sqlite3_manga::model_sqlite3_manga::create_tables(db_full_paths);

        // iterate through each row via csv_util.next() (it will deserialize it to MangaModel) and write it to SQLite
        let mut line_count = 0; // starting with 0, so that if first line returned is None, then we'll know that there is no line to process
        let mut update_count = 0;
        let mut possible_csv_row = csv_util.next();

        // NOTE: we do not return or panic!() inside this while loop, instead we'll
        //       just print out the error and continue on to the next row
        //       but track all the errors and return it at the end
        let mut ret_errors: Vec<Result<(), Box<dyn std::error::Error>>> = Vec::new(); // vec![];
        while possible_csv_row.is_some() {
            // write to SQLite
            #[cfg(debug_assertions)]
            {
                println!("# csv_row (raw, from file): {:?}", possible_csv_row);
            }
            line_count += 1;

            // TODO: Make this match into map (i.e. possible_csv_row.map(|file_csv_row| { ... })) for it'll make it more cleaner to read
            match possible_csv_row {
                Some(result) => {
                    match result {
                        Ok(csv_row) => {
                            #[cfg(debug_assertions)]
                            {
                                println!("#\tcsv_row (parsed): {:?}", &csv_row);
                            }
                            // write to SQLite - the model from DB SHOULD have correct Manga.ID
                            match upsert_db(db_full_paths, &csv_row, true, debug_flag) {
                                Ok(_) => {
                                    update_count += 1;
                                }
                                Err(e) => {
                                    println!("ERROR read_csv_and_update_sqlite(): Error writing CSV row {:?}:\n>\t{}\n", &csv_row, &e);
                                    ret_errors.push(Err(e));
                                }
                            }
                        }
                        Err(csv_error) => {
                            // do nothing, let it
                            ret_errors.push(Err(Box::new(csv_error)));
                        }
                    }
                }
                None => {
                    // NOTE: While() loop should have prevented from ever hitting this case, but just in case...
                    // CSV reader could not read the row, so we'll log message that we're done and bail out of this while loop
                    println!(
                        "CSV reader could not read the row, assuming we are adone reading CSV file"
                    );
                }
            }
            possible_csv_row = csv_util.next(); // should return if there is no more row to read (EOF:w
        }

        // print some stats on completions (success of fail) of filename and number of rows (lines) processed
        println!("CSV file: {}", db_full_paths);
        println!("CSV file lines read: {}", line_count);
        println!("Rows (lines) upserted: {}", update_count);
        match ret_errors.len() {
            0 => {
                // special case, when line_count is 0, then we'll return error of "no rows processed"
                if line_count == 0 {
                    println!("Error: No rows processed");
                    // safe to bail out here with a return
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "No rows processed",
                    )));
                }
                // no errors
                Ok(())
            }
            _ => {
                println!("{} errors found while reading CSV stream", ret_errors.len());
                // append all errors as single string and return it
                let mut ret_error_str = String::new();
                for ret_error in ret_errors {
                    ret_error_str.push_str(&format!("{:?}\n", ret_error)); // is it considered a hack to use {:?} instead of {}?
                }
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    ret_error_str,
                )) as Box<dyn std::error::Error>)
            }
        }
    }

    pub fn parse_args(
        args: Vec<String>,
    ) -> Result<
        (
            String,                           // SQLite3 database full path
            Box<dyn BufRead + 'static>, // -i: either stdin or input file-stream of JSON (bookmak) file (NOTE: special case of using 'static)
            Option<Box<dyn Write + 'static>>, // -o: either stdout or output file-stream of CSV file
            bool,                       // -D debug flag
        ),
        Box<dyn std::error::Error>,
    > {
        #[cfg(debug_assertions)]
        {
            println!("args: {:?}", args);
        }
        let mut has_input_file = false;
        let mut has_output_file = false;
        let mut has_possible_input_csv_file = false;
        let mut has_db_file = false;
        let mut input_filepaths_bookmark_json = String::new();
        let mut output_filepaths_csv = String::new();
        let mut possible_last_csv: Option<String> = None;
        let mut db_full_paths = String::new();
        let mut debug_flag = false;
        let mut i = 0;
        while i < args.len() {
            println!("arg[{}]: {}", i, args[i]);
            if args[i] == "-i" {
                has_input_file = true;
                input_filepaths_bookmark_json = args[i + 1].clone();
                i += 2; // increment by 2 to skip the next argument
            } else if args[i] == "-o" {
                has_output_file = true;
                output_filepaths_csv = args[i + 1].clone();
                i += 2; // increment by 2 to skip the next argument
            } else if args[i] == "-c" {
                has_possible_input_csv_file = true;
                possible_last_csv = Some(args[i + 1].clone());
                i += 2; // increment by 2 to skip the next argument
            } else if args[i] == "-d" {
                has_db_file = true;
                db_full_paths = args[i + 1].clone();
                i += 2; // increment by 2 to skip the next argument
            } else if args[i] == "-D" {
                // debug mode
                debug_flag = true;
                i += 1; // increment by 1 to skip the next argument
            } else if args[i] == "-h" || args[i] == "--help" {
                println!("Usage: {} [-i <bookmark.json>] [-o <output.csv>] [-c <last.csv>] [-d <db.sqlite3>] [-D]", args[0]);
                println!("-i is not specified, then stdin will be used");
                println!("-d (required): SQLite3 database file");
                println!("-o (optional): if not specified, then NO CSV will be written");
                println!("-c (optional): if not specified, then it will be ignored");
                println!("-D (optional): Debug outpupt");
            } else {
                println!("Unknown argument: '{}'", args[i]);
                // throw error
                //return Err(format!("Unknown argument: {}", args[i]));
                i += 1; // increment by 1 to skip the next argument
            }
        }

        println!("DB_file (SQLite3): {} '{}'", has_db_file, db_full_paths);
        println!(
            "Input_file (bookmark JSON): {} '{}'",
            has_input_file, input_filepaths_bookmark_json
        );
        println!(
            "Output_file (CSV): {} '{}'",
            has_output_file, output_filepaths_csv
        );
        println!(
            "Possible Input_csv_file (previously persisted CSV): {} '{:?}'",
            has_possible_input_csv_file, possible_last_csv
        );
        println!("NOTE: If Input_csv_file is optional if SQLite3 is up-to-date, and the CSV is basically human-readable version of SQLite3");
        println!("If there are conflicts between CSV and SQLite3, then the CSV will take precedence over SQLite3 and updates will be written to SQLite3");
        println!(
            "This way, one can hand-edit and update CSV file and then re-import it into SQLite3"
        );

        // first, read all available data and build database from both CSV (i.e. 漫画.csv) and JSON (i.e. bookmark.json)
        // into SQLite3 database 漫画.sqlite3
        if db_full_paths.is_empty() {
            // locate to see if '漫画.csv' exists in current directory
            let db_full_paths = String::from("漫画.sqlite3");
            if !std::path::Path::new(&db_full_paths).exists() {
                // exit application via panic!()
                panic!("Error: DB file '{}' does not exist", db_full_paths);
            }
        } else {
            // make sure that the DB file exists (accessible)
            if !std::path::Path::new(&db_full_paths).exists() {
                // exit application
                panic!("Error: DB file '{}' does not exist", db_full_paths);
            }
        }

        // append/read (deserialize) from input CSV file (if it exists)
        // note that when deserializing to the SQLite3, we'll just overwrite the existing entry
        // via upsert() method so ideally, caller should backup the existing database file
        if has_possible_input_csv_file && possible_last_csv.is_some() {
            match possible_last_csv {
                Some(last_csv) => {
                    // make sure that the CSV file exists (accessible)
                    if !std::path::Path::new(&last_csv).exists() {
                        // No need to panic, just ignore and use the SQLite3 file
                        println!("Error: CSV file '{}' does not exist", last_csv);
                    }
                    // open stream for csv file
                    match File::open(last_csv.clone()) {
                        Ok(input_csv_file) => {
                            // read CSV file and deserialize each row, we'll directly
                            // pass/transfer it down to SQLite
                            match read_csv_and_update_sqlite(
                                Box::new(input_csv_file),
                                &db_full_paths.clone(),
                                debug_flag,
                            ) {
                                Ok(()) => {
                                    // read line and written/updated to sqlite3...
                                }
                                Err(e) => {
                                    println!(
                                        "Error reading CSV file '{}' to be updating '{}': {}",
                                        last_csv, db_full_paths, e
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            // file doesn't exist, just log error and continue on
                            println!("Error opening CSV file '{}': {}", last_csv.clone(), e);
                        }
                    };
                }
                None => (), // no-op, assume SQLite3 is up-to-date
            }
        }

        // now depending on stdin or firefox bookmark JSON file, we'll create a stream
        let input_reader_bookmark_json: Box<dyn BufRead + 'static> = if has_input_file {
            // open stream for input file
            match File::open(input_filepaths_bookmark_json.clone()) {
                Ok(input_file) => {
                    Box::new(BufReader::new(input_file)) as Box<dyn BufRead + 'static>
                }
                Err(e) => {
                    // file doesn't exist, just log error and continue on
                    panic!(
                        "Error opening input file '{}': {}",
                        input_filepaths_bookmark_json.clone(),
                        e
                    );
                }
            }
        } else {
            // use stdin
            Box::new(BufReader::new(io::stdin())) as Box<dyn BufRead + 'static>
        };

        // next, create a stream for CSV output (either csv file or stdout)
        let output_writer_csv: Option<Box<dyn Write + 'static>> = if has_output_file {
            // open stream for output file
            match File::create(output_filepaths_csv.clone()) {
                Ok(output_file) => {
                    Some(Box::new(BufWriter::new(output_file)) as Box<dyn Write + 'static>)
                }
                Err(e) => {
                    // file doesn't exist, just log error and continue on
                    panic!(
                        "Error opening output file '{}': {}",
                        output_filepaths_csv.clone(),
                        e
                    );
                }
            }
        } else {
            //Some(Box::new(BufWriter::new(io::stdout())) as Box<dyn Write + 'static>)  // stdout - legacy method auto generated CSV to stdout...
            // Unlike missing of -i input, for output, if '-o' is missing, assume we do not generate output CSV file
            None
        };

        let ret_tuple = (
            db_full_paths.clone(),
            input_reader_bookmark_json, // -i
            output_writer_csv,          // -o
            debug_flag,
        );

        Ok(ret_tuple)
    }

    #[test]
    fn test_parse_args() {
        let tmp_dir = String::from(env::temp_dir().as_os_str().to_str().unwrap_or_default()); // sadly, Windows will fail on "/dev/shm/" so we'll have to use temp_dir()

        // Test with input file and output file
        let args = vec![
            String::from("-i"),
            String::from("samples/input.json"),
            String::from("-o"),
            String::from(tmp_dir.clone() + "output.csv"),
            String::from("-c"),
            String::from(tmp_dir.clone() + "current_list.csv"),
            String::from("-d"),
            String::from(tmp_dir.clone() + "parse_args.sqlite3"),
        ];

        // prior to entering the test, we want to make sure db file exists because parse_args() will ASSUME that it exists
        let db_full_paths = tmp_dir.clone() + "parse_args.sqlite3".into();
        if !std::path::Path::new(&db_full_paths).exists() {
            // create it
            match model_sqlite3_manga::model_sqlite3_manga::create_tables(&db_full_paths) {
                Ok(_) => {}
                Err(e) => {
                    panic!("Error creating DB file: {}", e);
                }
            }
        }

        match parse_args(args) {
            Ok((_db_full_paths, _input_json, possible_output_csv, _)) => {
                // clean up and close
                match possible_output_csv {
                    Some(mut output_csv) => {
                        output_csv.flush().unwrap();
                    }
                    None => (),
                }
            }
            Err(e) => {
                panic!("{}", e);
            }
        }

        let tmp_dir = String::from(env::temp_dir().as_os_str().to_str().unwrap_or_default()); // sadly, Windows will fail on "/dev/shm/" so we'll have to use temp_dir()

        // read test JSON files and attempt to deserialize it
        let args = vec![
            String::from("-i"),
            String::from("samples/input.json"),
            String::from("-d"),
            String::from(tmp_dir.clone() + "parse_args.sqlite3"),
        ];
        match parse_args(args) {
            Ok((_db_paths, input_json, possible_output_csv, _)) => {
                // deserialize - from_reader() method needs to access io::Read::bytes() method
                let bookmark_folders: BookmarkRootFolder =
                    serde_json::from_reader(input_json).unwrap();

                // for test, just recursively traverse down each children and print the title and lastModified and the type
                fn traverse_children(children: &Vec<BookmarkNodes>) {
                    for child in children {
                        println!(
                            "title: {}, lastModified: {}, uri: {:#?}",
                            child.title(),
                            child.last_modified(),
                            child.uri()
                        );
                        if let Some(children) = &child.possible_children() {
                            traverse_children(children);
                        }
                    }
                }
                traverse_children(bookmark_folders.children());

                // clean up and close
                match possible_output_csv {
                    Some(mut output_csv) => {
                        output_csv.flush().unwrap();
                    }
                    None => (),
                }
            }
            Err(e) => {
                panic!("{}", e);
            }
        }
    }
}

fn get_args() -> (
    String,                           // db_full_paths*/
    Box<dyn BufRead + 'static>,       // input_reader_bookmark_json*/
    Option<Box<dyn Write + 'static>>, // output_writer_csv*/
    bool,                             // debug_flag
) {
    let args: Vec<String> = std::env::args().collect();

    // read in JSON either from stdin or file
    let (db_full_paths, input_reader_bookmark_json, possible_output_writer_csv, debug_flag) =
        match json_to_csv::parse_args(args) {
            Ok((db_full_paths, input_reader_json, output_writer_csv, debug_flag)) => (
                db_full_paths,
                input_reader_json,
                output_writer_csv,
                debug_flag,
            ),
            Err(e) => {
                // do we want to just panic?
                panic!("{}", e);
            }
        };
    (
        db_full_paths,
        input_reader_bookmark_json,
        possible_output_writer_csv,
        debug_flag,
    )
}

fn read_bookmarks_into_manga<'a>(
    result_bookmark_folders: &Result<BookmarkRootFolder, serde_json::Error>,
) -> Result<Vec<MangaModel>, Box<dyn std::error::Error + '_>> {
    let bookmarks_raw: Vec<BookmarkNodes> = match result_bookmark_folders {
        Ok(bookmark_folders) => {
            // recursively visit each child and return Some tuple if it is bookmark, else return None for containers and separators
            fn traverse_children(children: &Vec<BookmarkNodes>) -> Vec<BookmarkNodes> {
                let mut bookmarks: Vec<BookmarkNodes> = Vec::new();
                for child in children {
                    if child.is_bookmark() {
                        bookmarks.push(child.clone());
                    } else if let Some(children) = &child.possible_children() {
                        bookmarks.append(&mut traverse_children(children));
                    }
                    // else, it's a separator, so we'll ignore it...
                }
                bookmarks
            }
            traverse_children(bookmark_folders.children())
        }
        Err(e) => {
            let err = e.clone();
            // pretty much, if we cannot read the JSON, then this app is useless, so just panic!() at the caller level
            // just opt-out early and bail out of this function
            println!("Error deserializing JSON: {}", err);
            let ret = Box::new(err);
            return Err(ret);
        }
    };

    // now that we've got it as data-model, we will just travese down each child and print out the title, URI, and last modified date, sorted by last modified date
    let mut bookmarks_sorted: Vec<BookmarkNodes> = bookmarks_raw.clone();
    //bookmarks_sorted.sort_by(|a, b| a.last_modified().cmp(&b.last_modified()));   // sort by date-column
    bookmarks_sorted.sort_by(|a, b| a.uri().cmp(&b.uri())); // sort by URI

    // CSV output, we're assuming that by here, only the "places" nodes are left, so we can just print them out in CSV format
    // either to the stdout or to the output file stream
    //let mut csv_writer = csv::WriterBuilder::new()
    //    .quote_style(csv::QuoteStyle::Always) // just easier to just quote everything including numbers
    //    .from_writer(output_writer);
    let mut mangas_mut = Vec::new();
    for bookmark in bookmarks_sorted {
        // convert the last_modified i64 to datetime - last_modified is encoded as unix epoch time in microseconds
        let str_last_modified = my_libs::from_epoch_to_str(*bookmark.last_modified());
        let mut mm: MangaModel = match MangaModel::new_from_required_elements(
            bookmark.title(),
            bookmark.uri().clone().as_str(),
            model_manga::CASTAGNOLI.checksum(bookmark.uri().clone().as_bytes()),
        ) {
            Ok(mm) => mm,
            Err(e) => {
                // let the caller handle panic, here we'll just opt out early and return error
                println!("Error creating MangaModel: {}", e);
                return Err(e);
            }
        };
        mm.set_last_update(Some(str_last_modified));
        mangas_mut.push(mm);
    }

    // now that new and old are merged, sort by last_modified and print out the CSV
    mangas_mut.sort_by(|a, b| a.url().cmp(&b.url()));
    Ok(mangas_mut)
}

fn main() {
    // read in JSON either from stdin or file
    let (db_full_paths, input_reader_bookmark_json, possible_output_writer_csv, debug_flag) =
        get_args();

    // read in JSON and deserialize it as Bookmark structure
    let bookmark_folders: Result<BookmarkRootFolder, serde_json::Error> =
        serde_json::from_reader(input_reader_bookmark_json);

    let _mut_csv_writer_util = model_csv_manga::model_csv_manga::Utils::new(
        possible_output_writer_csv,
        Box::new(BufReader::new(io::stdin())),
    );
    // read in json (firefox bookmarks) and deserialize it into MangaModel - pass writer by ref
    let mangas_mut = read_bookmarks_into_manga(&bookmark_folders).unwrap(); // let's panic if it fails

    // update local sqlite database with mangas_mut (Vec<MangaModel> list)
    for manga in &mangas_mut {
        if debug_flag {
            //println!("manga: {:#?}", manga);
            //println!("manga: {:?}", manga);
            println!("manga => {}", manga); // since Display is impl'ed for MangaModel, we can just print it out
        }
        let _db_result = upsert_db(&db_full_paths, manga, true, debug_flag);
    }
}
