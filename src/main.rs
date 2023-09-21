use model_csv_manga::model_csv_manga::CsvMangaModel;

mod model_csv_manga;
mod model_json_mozilla_bookmarks; // this is the same as `mod model_json; pub use model_json::*;`

mod json_to_csv {
    //use serde_json::Value;

    use std::{
        fs::File,
        io::{self, BufRead, BufReader, BufWriter, Write},
    };

    use crate::{
        model_csv_manga,
        model_json_mozilla_bookmarks::model_json_mozilla_bookmarks::{
            BookmarkNodes, BookmarkRootFolder,
        },
    };

    pub fn parse_args(
        args: Vec<String>,
    ) -> Result<
        (
            Box<dyn BufRead + 'static>,
            Box<dyn Write + 'static>,
            Option<Vec<model_csv_manga::model_csv_manga::CsvMangaModel>>,
        ),
        String,
    > {
        let mut input_file = false;
        let mut output_file = false;
        let mut input_csv_file = false;
        let mut input = String::new();
        let mut output = String::new();
        let mut input_csv = String::new();
        for i in 0..args.len() {
            if args[i] == "-i" {
                input_file = true;
                input = args[i + 1].clone();
            }
            if args[i] == "-o" {
                output_file = true;
                output = args[i + 1].clone();
            }
            if args[i] == "-c" {
                input_csv_file = true;
                input_csv = args[i + 1].clone();
            }
        }

        println!("\nInput_file: {} '{}'", input_file, input);
        println!("Output_file: {} '{}'", output_file, output);
        println!("Input_csv_file: {} '{}'", input_csv_file, input_csv);

        // append/read (deserialize) from input CSV file (if it exists)
        let mut last_csv_file: Option<Vec<model_csv_manga::model_csv_manga::CsvMangaModel>> = None;
        if input_csv_file {
            match File::open(&input_csv) {
                Ok(file) => {
                    println!("Input CSV file: {}", input_csv);
                    let input_reader = Box::new(BufReader::new(file)); // NOTE: file is NOT std::io::stdin(), or can  it be?
                    let mangas = model_csv_manga::model_csv_manga::Utils::read_csv(input_reader);
                    last_csv_file = Some(mangas);
                }
                Err(e) => {
                    // just log that we cannot find it, but it's no big deal (keep last_csv_file=None)
                    println!("Error opening CSV file '{}': {}", input_csv, e)
                }
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
                                return Ok((input_reader, output_writer, last_csv_file));
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
                        return Ok((input_reader, output_writer, last_csv_file));
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
                        return Ok((input_reader, output_writer, last_csv_file));
                    }
                    Err(e) => {
                        return Err(format!("Error creating output file: {}", e));
                    }
                }
            } else {
                println!("Output file: stdout");
                let output_writer = Box::new(BufWriter::new(io::stdout()));
                return Ok((input_reader, output_writer, last_csv_file));
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
            String::from("-c"),
            String::from("/dev/shm/current_list.csv"),
        ];
        match parse_args(args) {
            Ok((_input, mut output, _possible_mangas)) => {
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
            Ok((input, mut output, _possible_mangas)) => {
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
    let (input_reader, output_writer, possible_mangas) = match json_to_csv::parse_args(args) {
        Ok((input_reader, output_writer, possible_mangas)) => {
            (input_reader, output_writer, possible_mangas)
        }
        Err(e) => {
            println!("{}", e);
            return;
        }
    };

    // read in JSON and deserialize it as Bookmark structure
    let bookmark_folders: Result<
        model_json_mozilla_bookmarks::model_json_mozilla_bookmarks::BookmarkRootFolder,
        _,
    > = serde_json::from_reader(input_reader);
    let bookmarks: Vec<model_json_mozilla_bookmarks::model_json_mozilla_bookmarks::BookmarkNodes> =
        match bookmark_folders {
            Ok(bookmark_folders) => {
                // recursively visit each child and return Some tuple if it is bookmark, else return None for containers and separators
                fn traverse_children(
                    children: &Vec<
                        model_json_mozilla_bookmarks::model_json_mozilla_bookmarks::BookmarkNodes,
                    >,
                ) -> Vec<model_json_mozilla_bookmarks::model_json_mozilla_bookmarks::BookmarkNodes>
                {
                    let mut bookmarks: Vec<
                        model_json_mozilla_bookmarks::model_json_mozilla_bookmarks::BookmarkNodes,
                    > = Vec::new();
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
    let mut bookmarks_sorted: Vec<
        model_json_mozilla_bookmarks::model_json_mozilla_bookmarks::BookmarkNodes,
    > = bookmarks.clone();
    //bookmarks_sorted.sort_by(|a, b| a.last_modified().cmp(&b.last_modified()));   // sort by date-column
    bookmarks_sorted.sort_by(|a, b| a.uri().cmp(&b.uri())); // sort by URI

    // CSV output, we're assuming that by here, only the "places" nodes are left, so we can just print them out in CSV format
    // either to the stdout or to the output file stream
    //let mut csv_writer = csv::WriterBuilder::new()
    //    .quote_style(csv::QuoteStyle::Always) // just easier to just quote everything including numbers
    //    .from_writer(output_writer);
    let mut mut_csv_writer = model_csv_manga::model_csv_manga::Utils::new(output_writer);
    let mut mangas_mut = possible_mangas.unwrap_or(Vec::new());
    for bookmark in bookmarks_sorted {
        // convert the last_modified i64 to datetime - last_modified is encoded as unix epoch time in microseconds
        let last_modified = chrono::NaiveDateTime::from_timestamp_opt(
            bookmark.last_modified() / 1_000_000,
            (bookmark.last_modified() % 1_000_000) as u32,
        )
        .unwrap();
        //let m = mut_csv_writer.record(
        //    last_modified.timestamp_micros(),
        //    &bookmark.uri(),
        //    bookmark.title(),
        //);
        let m = model_csv_manga::model_csv_manga::CsvMangaModel::new_from_bookmark(
            last_modified.timestamp_micros(),
            &bookmark.uri(),
            bookmark.title(),
        );
        mangas_mut.push(m);
    }

    // now that new and old are merged, sort by last_modified and print out the CSV
    mangas_mut.sort_by(|a, b| a.url().cmp(&b.url()));

    // lastly, want to split into 2 files, one that is sorted but unique URL in which
    // if there are duplicates in which one isin JA_JP and another in ROMAJI for title,
    // then we want to keep the JA_JP one and dump the ROMAJI one into the "duplicates" file
    // in another case, if there are multiple rows (JA_JP and ROMAJI) for the same URL,
    // then we want to keep the one with the latest last_modified date OR bigger chapter number
    // and dump the other one into the "duplicates" file
    // see also Linux tool 'uniq -c' (note that 'uniq' requires sorted input for it tests against
    // ADJACENT lines, so we need to sort it first)

    // easiest is to create map (dictionary) based on what will consider as "unique" key
    // We'll create multiple maps, one for each key
    let mut url_map: std::collections::HashMap<
        String,
        Vec<model_csv_manga::model_csv_manga::CsvMangaModel>,
    > = std::collections::HashMap::new();
    let mut romaji_title_map: std::collections::HashMap<
        String,
        Vec<model_csv_manga::model_csv_manga::CsvMangaModel>,
    > = std::collections::HashMap::new();
    for mut manga_mut in mangas_mut {
        // if URL is empty st, then we'll use the title as the url_with_chapters as fallback
        let key_url = manga_mut.url_mut().to_string();
        if url_map.contains_key(&key_url) {
            url_map.get_mut(&key_url).unwrap().push(manga_mut.clone());
        } else {
            url_map.insert(key_url, vec![manga_mut.clone()]);
        }

        let key_romaji_title = manga_mut.romanized_title_mut().to_string();
        if romaji_title_map.contains_key(&key_romaji_title) {
            romaji_title_map
                .get_mut(&key_romaji_title)
                .unwrap()
                .push(manga_mut.clone());
        } else {
            romaji_title_map.insert(key_romaji_title, vec![manga_mut.clone()]);
        }
    }

    // because maps are auto-sorted by key, there are no sort_by() method, we'll
    // create a new map that has only one entry per key (can do vec, but it's nice to
    // have a list presoted by keys)
    let mut url_map_unique: std::collections::HashMap<
        String,
        model_csv_manga::model_csv_manga::CsvMangaModel,
    > = std::collections::HashMap::new();
    let mut romaji_title_map_unique: std::collections::HashMap<
        String,
        model_csv_manga::model_csv_manga::CsvMangaModel,
    > = std::collections::HashMap::new();
    for (key, mangas) in &url_map {
        // if only one entry, then add to url_map_unique
        if mangas.len() == 1 {
            // since len==1, first() is the only element...
            url_map_unique.insert(key.into(), mangas.first().unwrap().clone());
        }
    }
    // remove all unique entries from url_map so that we're left with ones that have duplicates
    for (key, _) in &url_map_unique {
        url_map.remove(key);
    }

    // now that we have url_map that have dupes, we'll generate two maps
    // do the same for romaji_title_map
    for (key, mangas) in &romaji_title_map {
        // if only one entry, then add to romaji_title_map_unique
        if mangas.len() == 1 {
            romaji_title_map_unique.insert(key.into(), mangas.first().unwrap().clone());
        }
    }
    // remove unique entries from romaji_title_map
    for (key, _) in &romaji_title_map_unique {
        romaji_title_map.remove(key);
    }

    // now merge the two uniques into single merged_unique_map,
    // rather than checking whther key already exists or not, we'll just
    // insert it and let the HashMap overwrite the existing entry
    // because of that characteristic nature, it's important that
    // we'd iterate the more important map last
    let mut merged_unique_map: std::collections::HashMap<
        String,
        model_csv_manga::model_csv_manga::CsvMangaModel,
    > = std::collections::HashMap::new();
    for (key, manga) in url_map_unique {
        merged_unique_map.insert(key, manga);
    }
    for (key, manga) in romaji_title_map_unique {
        merged_unique_map.insert(key, manga);
    }

    // let's double check one last time, to make sure if the merged_unique_map
    // does not have a key in the url_map or romaji_title_map
    for (key, _) in &url_map {
        // if key exists in url_map, then it should not be in merged_unique_map
        if merged_unique_map.contains_key(key) {
            // remove it
            merged_unique_map.remove(key);
        }
    }
    for (key, _) in &romaji_title_map {
        // if key exists in romaji_title_map, then it should not be in merged_unique_map
        if merged_unique_map.contains_key(key) {
            // remove it
            merged_unique_map.remove(key);
        }
    }

    // now we are absolutely sure that merged_unique_map has only unique entries
    // and we can dump it to the output file
    for (_, manga) in &merged_unique_map {
        #[cfg(debug_assertions)]
        {
            //println!("{}", manga);
        }
        mut_csv_writer.record(&manga.clone());
    }

    // add a MARKER to indicate that this is the end of the unique list and what are to follow are duplicates
    let marker_manga =
        CsvMangaModel::new_from_bookmark(0, &String::from("MARKER"), &String::from("MARKER"));
    mut_csv_writer.record(&marker_manga);

    // url_map and romaji_title_map are basically bookmarks that needs to be narrowed down to
    // single URL but because it has same URL but differs by title, or same title but differs by URL
    // (i.e. due to one URL is chapter-1 and the other is chapter-2)
    let mut merged_duplicates_map: Vec<model_csv_manga::model_csv_manga::CsvMangaModel> =
        Vec::new();
    for (_, mangas) in &url_map {
        let mut mangas_for_update = mangas.clone();
        // Iterate through each elements in mangas for its romaji_title (key) of romaji_title_map add locate any
        // elements that does not match the key and add/update it to the mangas list
        for m in mangas.clone() {
            // search if m.url() is found in the URL map...
            if romaji_title_map.contains_key(m.url()) {
                // found url as key, move all the elements here into mangas list
                for m2 in romaji_title_map.get(m.url()).unwrap() {
                    // make sure this CsvMangaModel isn't already in the list
                    if !mangas_for_update.contains(m2) {
                        // not found, add it to the url key
                        mangas_for_update.push(m2.clone());
                    }
                }

                // remove the key from romaji_title_map
                romaji_title_map.remove(m.url()).unwrap();
            }
        }

        // and then add it as merged list
        merged_duplicates_map.append(&mut mangas_for_update);
    }
    // now append (to merged_duplicates_map) any remaining elements in romaji_title_map
    for (_, mangas) in romaji_title_map {
        let mut mangas_for_update = mangas.clone();
        // Iterate through each elements in mangas for its url (key) of url_map add locate any
        // elements that does not match the key and add/update it to the mangas list
        for m in mangas.clone() {
            // search if m.romanized_title() is found in the URL map...
            // and if so, grabe it, and remove it from url_map
            if url_map.contains_key(m.romanized_title()) {
                // found url as key, move all the elements here into mangas list
                for m2 in url_map.get(m.romanized_title()).unwrap() {
                    // make sure this CsvMangaModel isn't already in the list
                    if !mangas_for_update.contains(m2) {
                        mangas_for_update.push(m2.clone());
                    }
                }

                // remove the key from url_map
                // probably not needed, but just in case this block is moved/reordered...
                url_map.remove(m.romanized_title()).unwrap();
            }
        }

        // and then add it as merged list
        merged_duplicates_map.append(&mut mangas_for_update);
    }

    // final sorting by URL
    merged_duplicates_map.sort_by(|a, b| a.url().cmp(&b.url()));

    // now that we've merged the url_map and romaji_title_map, we'll just dump it to the output file
    for manga in merged_duplicates_map {
        #[cfg(debug_assertions)]
        {
            //println!("{}", manga);
        }
        mut_csv_writer.record(&manga.clone());
    }

    // just in case, let's also dump url_map in case it has something in it still (it should be presorted by key:URL)
    // so it'll be blocks of duplicates with same URL
    for (_, mangas) in url_map {
        for manga in mangas {
            #[cfg(debug_assertions)]
            {
                //println!("{}", manga);
            }
            mut_csv_writer.record(&manga.clone());
        }
    }
}
