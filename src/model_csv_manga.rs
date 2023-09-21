//use kakasi;

// manga csv format
// "title","url", "url_with_chapters", "chapter","last_modified","notes", "tags"
pub mod model_csv_manga {
    use csv::Writer;
    use serde::{Deserialize, Serialize};
    use std::fmt::{self, Debug, Display};
    use std::io::Write;

    // Custom deserialization function for Option<String>
    fn fn_deserialize_option_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: Result<String, _> = Deserialize::deserialize(deserializer);
        Ok(s.ok())
    }
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct CsvMangaModel {
        title: String,
        url_with_chapters: String,
        chapter: String,
        last_modified_YYYYmmddTHHMMSS: String,
        notes: String,
        tags: String,

        // Note: all new varaibles have to be Option type AND must be appended to the end of the struct
        // All variables that are Option type should get serialized to Some("") so that writer can make
        // sure to pack it (i.e. "a",,,"d",,"f") - Option type is only for the sake of missing data
        // on older version of the CSV
        #[serde(default)] // quite critical that you have this for any/almost-all serde elements that are Option type
        #[serde(deserialize_with = "fn_deserialize_option_string")]
        // this is the custom deserializer for Option<String>
        romanized_title: Option<String>, // for V2

        #[serde(default)] // quite critical that you have this for any/almost-all serde elements that are Option type
        #[serde(deserialize_with = "fn_deserialize_option_string")]
        // this is the custom deserializer for Option<String>
        url: Option<String>, // for V2
    }

    impl fmt::Display for CsvMangaModelV1 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                //"Title: {}\nRomanized Title: {}\nURL: {}\nURL with Chapters: {}\nChapter: {}\nLast Modified: {}\nNotes: {}\nTags: {}",
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
                self.title,
                self.url_with_chapters,
                self.chapter,
                self.last_modified_YYYYmmddTHHMMSS,
                self.notes,
                self.tags
            )
        }
    }
    impl fmt::Display for CsvMangaModelV2 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                //"Title: {}\nRomanized Title: {}\nURL: {}\nURL with Chapters: {}\nChapter: {}\nLast Modified: {}\nNotes: {}\nTags: {}",
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
                self.title,
                self.url_with_chapters,
                self.chapter,
                self.last_modified_YYYYmmddTHHMMSS,
                self.notes,
                self.tags,
                self.romanized_title,
                self.url,
            )
        }
    }

    impl fmt::Display for CsvMangaModel {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                //"Title: {}\nRomanized Title: {}\nURL: {}\nURL with Chapters: {}\nChapter: {}\nLast Modified: {}\nNotes: {}\nTags: {}",
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
                self.title,
                self.url_with_chapters,
                self.chapter,
                self.last_modified_YYYYmmddTHHMMSS,
                self.notes,
                self.tags,
                (match self.romanized_title {
                    Some(ref s) => s,
                    None => "",
                }),
                (match self.url {
                    Some(ref s) => s,
                    None => "",
                })
            )
        }
    }

    impl CsvMangaModel {
        pub fn new(title: String) -> Self {
            let romanized_title = CsvMangaModel::romanized(&title);
            CsvMangaModel {
                title,
                romanized_title: Some(CsvMangaModel::fix_comma_in_string(romanized_title.as_str())),
                url: None,
                url_with_chapters: String::from(""),
                chapter: String::from(""),
                last_modified_YYYYmmddTHHMMSS: String::from(""),
                notes: String::from(""),
                tags: String::from(""),
            }
        }

        pub fn new_with_url(title: String, url_with_chapters: String) -> Self {
            let romanized_title = CsvMangaModel::romanized(&title);
            let (uri_stripped, chapter) =
                CsvMangaModel::strip_chapter_from_url(url_with_chapters.clone());

            CsvMangaModel {
                title,
                romanized_title: Some(CsvMangaModel::fix_comma_in_string(romanized_title.as_str())),
                url: Some(uri_stripped.clone()),
                url_with_chapters,
                chapter: chapter.clone(),
                last_modified_YYYYmmddTHHMMSS: String::from(""),
                notes: String::from(""),
                tags: String::from(""),
            }
        }

        pub fn new_with_url_and_chapter(
            title: String,
            url_with_chapters: String,
            chapter: String,
        ) -> Self {
            let romanized_title = CsvMangaModel::romanized(&title);
            let (uri_stripped, _) =
                CsvMangaModel::strip_chapter_from_url(url_with_chapters.clone());

            CsvMangaModel {
                title,
                romanized_title: Some(CsvMangaModel::fix_comma_in_string(romanized_title.as_str())),
                url: Some(uri_stripped.clone()),
                url_with_chapters,
                chapter: chapter.clone(),
                last_modified_YYYYmmddTHHMMSS: String::from(""),
                notes: String::from(""),
                tags: String::from(""),
            }
        }

        pub fn new_from_bookmark(
            bookmark_last_modified_epoch_micros: i64,
            bookmark_uri: &String,
            bookmark_title: &String,
        ) -> Self {
            // convert the last_modified i64 to datetime - last_modified is encoded as unix epoch time in microseconds
            let last_modified = chrono::NaiveDateTime::from_timestamp_opt(
                bookmark_last_modified_epoch_micros / 1_000_000,
                (bookmark_last_modified_epoch_micros % 1_000_000) as u32,
            )
            .unwrap();
            // output: "uri_stripped_for_sorting","title","uri","chapter","last_modified","notes","tags"
            // extract chapter if link indicates so...

            let (uri_stripped, chapter) =
                CsvMangaModel::strip_chapter_from_url(bookmark_uri.into());

            CsvMangaModel {
                title: CsvMangaModel::fix_comma_in_string(bookmark_title.clone().as_str()),
                romanized_title: Some(CsvMangaModel::romanized(&bookmark_title.clone())),
                url: Some(uri_stripped.clone()),
                url_with_chapters: bookmark_uri.clone(),
                chapter,
                last_modified_YYYYmmddTHHMMSS: last_modified
                    .format("%Y-%m-%dT%H:%M:%S")
                    .to_string(),
                notes: String::from(""),
                tags: String::from("#"),
            }
        }

        // serialize to CSV string
        pub fn to_csv(&self, for_sorting: bool) -> String {
            // Q: is there way to do this without using csv::WriterBuilder?
            let record = csv::StringRecord::new();

            let mut wtr = csv::WriterBuilder::new()
                .quote_style(csv::QuoteStyle::Always) // quote everything
                .has_headers(false)
                .from_writer(vec![]);
            if for_sorting {
                // NOTE: this serializer does NOT render URL without chapter
                wtr.serialize((
                    (match &self.url {
                        // special case where all columns will shift left by 1
                        Some(s) => s,
                        None => &self.url_with_chapters,
                    }),
                    &self.title,
                    &self.url_with_chapters,
                    &self.chapter,
                    &self.last_modified_YYYYmmddTHHMMSS,
                    &self.notes,
                    &self.tags,
                    // v2
                    (match &self.romanized_title {
                        Some(s) => s,
                        None => &self.title,
                    }),
                ))
                .unwrap();
            } else {
                wtr.serialize((
                    &self.title,
                    &self.url_with_chapters,
                    &self.chapter,
                    &self.last_modified_YYYYmmddTHHMMSS,
                    &self.notes,
                    &self.tags,
                    // v2
                    (match &self.url {
                        Some(s) => s,
                        None => &self.url_with_chapters,
                    }),
                    (match &self.romanized_title {
                        Some(s) => s,
                        None => &self.title,
                    }),
                ))
                .unwrap();
            }
            String::from_utf8(wtr.into_inner().unwrap()).unwrap()
        }
        // desrialize from CSV string
        pub fn from_csv(csv: &str) -> Result<CsvMangaModel, csv::Error> {
            // Q: Is there way to do this without using csv::ReaderBuilder?
            let record = csv::StringRecord::new();

            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(csv.as_bytes());

            let mut record: CsvMangaModel = rdr.deserialize().next().unwrap()?;
            #[cfg(debug_assertions)]
            {
                println!("\n>> {:?}", record);  // using Debug {:?} will render Option::None as None
                //println!(">> {}\n", record);
            }
            Ok(CsvMangaModel {
                title: record.title().into(),
                url_with_chapters: record.url_with_chapters().into(),
                chapter: record.chapter().into(),
                last_modified_YYYYmmddTHHMMSS: record.last_modified().into(),
                notes: record.notes().into(),
                tags: record.tags().into(),
                url: Some(record.url_mut().into()),
                romanized_title: Some(record.romanized_title_mut().into()),
            })
        }

        pub fn title(&self) -> &String {
            &self.title
        }

        // still trying to figure out whether to:
        // * return Option<String> as-is
        // * return String with "" if None
        // * return alternative String Title if None
        pub fn romanized_title_mut(&mut self) -> &String {
            match self.romanized_title {
                Some(ref s) => s, // found it!
                None =>
                // String::from(""),    // empty string
                {
                    self.romanized_title = Some(CsvMangaModel::romanized(&self.title));
                    &self.romanized_title.as_ref().unwrap()
                } // alternative
            }
        }
        pub fn romanized_title(&self) -> &String {
            match self.romanized_title {
                Some(ref s) => s, // found it!
                None => &self.title,
            }
        }
        // for here, similar to romanized_title(), but for alternative, urll_with_chapters
        pub fn url_mut(&mut self) -> &String {
            match self.url {
                Some(ref s) => s,
                None =>
                //String::from(""), // empty string
                {
                    let (alternative, _) =
                        CsvMangaModel::strip_chapter_from_url(self.url_with_chapters.clone());
                    self.url = Some(alternative);
                    &self.url.as_ref().unwrap() // alternative
                }
            }
        }
        pub fn url(&self) -> &String {
            match self.url {
                Some(ref s) => s,
                None => &self.url_with_chapters,
            }
        }
        pub fn url_with_chapters(&self) -> &String {
            &self.url_with_chapters
        }
        pub fn chapter(&self) -> &String {
            &self.chapter
        }
        pub fn last_modified(&self) -> &String {
            &self.last_modified_YYYYmmddTHHMMSS
        }
        pub fn notes(&self) -> &String {
            &self.notes
        }
        pub fn tags(&self) -> &String {
            &self.tags
        }

        fn fix_comma_in_string(s: &str) -> String {
            Utils::fix_comma_in_string(s)
        }
        // only exposing this function so that use-depencies of kakasi will be limited to this module only
        // but mainly, also want to preserve at least the UTF8 comma ("、") in the title
        fn romanized(title: &String) -> String {
            CsvMangaModel::fix_comma_in_string(kakasi::convert(title).romaji.as_str())
        }

        pub fn get_last_modified(&self) -> i64 {
            let timestampe_epoch_micros = chrono::NaiveDateTime::parse_from_str(
                &self.last_modified_YYYYmmddTHHMMSS,
                "%Y-%m-%dT%H:%M:%S",
            )
            .unwrap()
            .timestamp_micros();

            // convert the last_modified i64 to datetime - last_modified is encoded as unix epoch time in microseconds
            chrono::NaiveDateTime::from_timestamp_opt(
                timestampe_epoch_micros / 1_000_000,
                (timestampe_epoch_micros % 1_000_000) as u32,
            )
            .unwrap()
            .timestamp_micros()
        }
        fn strip_chapter_from_url(url_with_chapters: String) -> (String, String) {
            Utils::strip_chapter_from_url(&url_with_chapters)
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CsvMangaModelV1 {
        #[serde(rename = "Title")]
        title: String,

        #[serde(rename = "URL_with_Chapters")]
        url_with_chapters: String,

        #[serde(rename = "Chapter")]
        chapter: String,

        #[serde(rename = "Last_Modified")]
        last_modified_YYYYmmddTHHMMSS: String, // format is "%Y-%m-%dT%H:%M:%S"

        #[serde(rename = "Notes")]
        notes: String,

        #[serde(rename = "Tags")]
        tags: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CsvMangaModelV2 {
        // the two new fields for sroting are at the head so that post-sorting, you can 'cut -d',' -f3-' to return back to v1
        // Note that on this model, url and romanize_title are concrete rather than Some(s) since it's NEVER assumed to be None
        #[serde(rename = "URL")]
        url: String, // URL without the chapter (mainly for sorting purposes)

        #[serde(rename = "Romanized_Title")]
        romanized_title: String, // this too for sorting purposes

        #[serde(rename = "Title")]
        title: String,

        #[serde(rename = "URL_with_Chapters")]
        url_with_chapters: String,

        #[serde(rename = "Chapter")]
        chapter: String,

        #[serde(rename = "Last_Modified")]
        last_modified_YYYYmmddTHHMMSS: String, // format is "%Y-%m-%dT%H:%M:%S"

        #[serde(rename = "Notes")]
        notes: String,

        #[serde(rename = "Tags")]
        tags: String,
    }

    pub struct Utils {
        csv_writer: Writer<Box<dyn Write + 'static>>, // mutable reference to a trait object
    }
    impl Drop for Utils {
        fn drop(&mut self) {
            // flush the csv_writer
            self.csv_writer.flush().unwrap();
        }
    }

    impl Utils {
        // pass in writer, such as stdout or file
        pub fn new(output_writer: Box<dyn Write>) -> Utils {
            // fail immediately if output_writer is not a streamable writer
            Utils {
                csv_writer: csv::WriterBuilder::new()
                    .quote_style(csv::QuoteStyle::Always) // just easier to just quote everything including numbers
                    .from_writer(output_writer),
            }
        }

        pub fn fix_comma_in_string(s: &str) -> String {
            // NOTE: cannot have commmas inside strings for MOST CSV utilities fails to know the differences...
            // so, we need to replace all commas with something else, such as "、"
            s.replace(",", "、")
        }

        pub fn strip_chapter_from_url(
            url_with_chapters: &String,
        ) -> (String /*url_stripped*/, String /*chapter*/) {
            let target_string = "chapter";
            let chapter = match url_with_chapters
                .to_lowercase()
                .contains(&target_string.to_lowercase())
            {
                false => "0".to_string(),
                true => {
                    // get substring past the string "chapter" from the URI
                    // i.e. "http://mydomain.tld/title-chapter-10", "http://mydomain.tld/title-chapter-10-1", "http://mydomain.tld/title-chapter-10/", "http://mydomain.tld/title-chapter-10-1/"
                    // we want to extract as "10", "10.1" - note that trailing "/" needs to be removed and "-" needs to be replaced with "."
                    let mut chapter_number = url_with_chapters.to_lowercase();
                    chapter_number = chapter_number
                        .split_off(chapter_number.find(&target_string.to_lowercase()).unwrap());
                    chapter_number = chapter_number
                        .split_off(target_string.len())
                        .trim_start_matches('-')
                        .to_string();
                    // strip off trailing "/" if any
                    if chapter_number.ends_with('/') {
                        chapter_number.pop();
                    }
                    // substitute "-" with "." for chapter number
                    chapter_number.replace('-', ".")
                }
            };
            let mut uri_stripped = url_with_chapters.clone();
            if uri_stripped.to_lowercase().contains(target_string) {
                // remove trailing "/" if any first before popping other stuffs
                let has_closing_slash = uri_stripped.ends_with('/');

                // keep the string all the way up to "-chapter" and strip off "-chapter" and the rest to the end
                // split_off() moves the 2nd half as return value (updates the original string in-place),
                // since we don't need 2nd part, just ignore the return value
                let _ = uri_stripped.split_off(
                    uri_stripped
                        .to_lowercase()
                        .find(&target_string.to_lowercase())
                        .unwrap(),
                );
                // in case the "title-chapter" stripped off and is left as "title-", strip off the trailing "-"
                if uri_stripped.ends_with('-') {
                    uri_stripped.pop();
                }

                // finally, if there was a trailing "/", add it back
                if has_closing_slash {
                    uri_stripped.push('/');
                }
            }
            (uri_stripped, chapter)
        }

        // read the CSV file (either as a file stream or stdin stream) and convert to Vec<Manga> (uses Manga::from_csv() methods for each rows read)
        pub fn read_csv(input_reader: Box<dyn std::io::Read>) -> Vec<CsvMangaModel> {
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(input_reader);

            let mut mangas: Vec<CsvMangaModel> = Vec::new();
            //mangas.extend(rdr.deserialize::<Manga>());
            for result in rdr.deserialize() {
                match result {
                    Ok(result_record) => {
                        let record: CsvMangaModel = result_record;
                        #[cfg(debug_assertions)]
                        {
                            //
                        }

                        // push a copy
                        mangas.push(CsvMangaModel::new_with_url(
                            record.title,
                            record.url_with_chapters,
                        ));
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            mangas
        }

        pub fn write_csv(&mut self, mangas: &Vec<CsvMangaModel>) -> Result<(), csv::Error> {
            for manga in mangas {
                let mut record = csv::StringRecord::new();
                record.push_field(&manga.title);
                record.push_field(match &manga.romanized_title {
                    Some(s) => s,
                    None => "",
                });
                record.push_field(match &manga.url {
                    Some(s) => s,
                    None => "",
                });
                record.push_field(&manga.url_with_chapters);
                record.push_field(&manga.chapter);
                record.push_field(&manga.last_modified_YYYYmmddTHHMMSS);
                record.push_field(&manga.notes);
                record.push_field(&manga.tags);
                self.csv_writer.write_record(&record)?;
            }
            Ok(())
        }

        pub fn write_csv_header(&mut self) -> Result<(), csv::Error> {
            let mut record = csv::StringRecord::new();
            record.push_field("Title");
            record.push_field("Possible_Romanized_Title");
            record.push_field("Romanized_Title");
            record.push_field("Possible_URL");
            record.push_field("URL_with_Chapters");
            record.push_field("Chapter");
            record.push_field("Last_Modified");
            record.push_field("Notes");
            record.push_field("Tags");
            self.csv_writer.write_record(&record)
        }

        pub fn record_bookmark(
            &mut self,
            bookmark_last_modified_epoch_micros: i64,
            bookmark_uri: &String,
            bookmark_title: &String,
        ) -> CsvMangaModel {
            let m = CsvMangaModel::new_from_bookmark(
                bookmark_last_modified_epoch_micros,
                bookmark_uri,
                bookmark_title,
            );
            let mut record = csv::StringRecord::new();
            record.push_field(match &m.url {
                Some(s) => s,
                None => "",
            });
            record.push_field(match &m.romanized_title {
                Some(s) => s,
                None => "",
            });
            record.push_field(&m.title);
            record.push_field(&m.url_with_chapters);
            record.push_field(&m.chapter);
            record.push_field(&m.last_modified_YYYYmmddTHHMMSS);
            record.push_field(&m.notes);
            record.push_field(&m.tags);
            // write it
            self.csv_writer.write_record(&record).unwrap();
            m
        }
        pub fn record(&mut self, m: &CsvMangaModel) {
            let mut record = csv::StringRecord::new();
            record.push_field(match &m.url {
                Some(s) => s,
                None => "",
            });
            record.push_field(match &m.romanized_title {
                Some(s) => s,
                None => "",
            });
            record.push_field(&m.title);
            record.push_field(&m.url_with_chapters);
            record.push_field(&m.chapter);
            record.push_field(&m.last_modified_YYYYmmddTHHMMSS);
            record.push_field(&m.notes);
            record.push_field(&m.tags);
            // write it
            self.csv_writer.write_record(&record).unwrap();
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::model_csv_manga::model_csv_manga::CsvMangaModel;

        // chose this title for a critical reason that if withinside quotes, there is a UTF8 comma ("、"), which is a problem for CSV if it was converted to ","
        const K_MANGA_TITLE: &str = "ゲート―自衛隊彼の地にて、斯く戦えり";
        // notice that libkakasi converts "ゲート" to "geeto" and knows the difference between the dash in "ゲート" and the dash in "―" (which is a UTF8 dash), sadly it convers the UTF8 comma ("、") to a regular comma (","), in which we force it
        const K_EXPECTED_ROMANIZED_TITLE: &str = "geeto ― jieitai kano chi nite、 kaku tatakae ri";
        const K_MANGA_URL_WITH_CHAPTERS: &str = "https://example.com/manga/gate-chapter-10/";
        const K_MANGA_CHAPTER: &str = "10";
        const K_MANGA_LAST_MODIFIED: &str = "2021-07-22T12:34:56";
        const K_MANGA_NOTES: &str = "Notes may have commas, but they will get replaced with \"、\"";
        const K_MANGA_TAGS: &str = "#action; #isekai; #fantasy; #shounen";
        // note that this version needs to append "\n" at tail separately
        const K_MANGA_CSV_SORT_BY_URL : &str = "\"https://example.com/manga/gate-chapter-10/\",\"ゲート―自衛隊彼の地にて、斯く戦えり\",\"https://example.com/manga/gate-chapter-10/\",\"10\",\"2021-07-22T12:34:56\",\"Notes may have commas, but they will get replaced with \"\"、\"\"\",\"#action; #isekai; #fantasy; #shounen\",\"geeto ― jieitai kano chi nite、 kaku tatakae ri\"" ;
        const K_MANGA_CSV: &str = "\"ゲート―自衛隊彼の地にて、斯く戦えり\",\"https://example.com/manga/gate-chapter-10/\",\"10\",\"2021-07-22T12:34:56\",\"Notes may have commas, but they will get replaced with \"\"、\"\"\",\"#action; #isekai; #fantasy; #shounen\"";

        // Model based on constants defined:
        fn make_default_model() -> CsvMangaModel {
            CsvMangaModel {
                title: String::from(K_MANGA_TITLE),
                romanized_title: Some(String::from(K_EXPECTED_ROMANIZED_TITLE)),
                url: Some(String::from(K_MANGA_URL_WITH_CHAPTERS)),
                url_with_chapters: String::from(K_MANGA_URL_WITH_CHAPTERS),
                chapter: String::from(K_MANGA_CHAPTER),
                last_modified_YYYYmmddTHHMMSS: String::from(K_MANGA_LAST_MODIFIED),
                notes: String::from(K_MANGA_NOTES),
                tags: String::from(K_MANGA_TAGS),
            }
        }

        #[test]
        fn test_title() {
            let manga = make_default_model();
            assert_eq!(manga.title(), &K_MANGA_TITLE);
        }

        #[test]
        fn test_romanized_title() {
            let mut manga = make_default_model();
            assert_eq!(manga.romanized_title(), &K_EXPECTED_ROMANIZED_TITLE);
            assert_eq!(manga.romanized_title_mut(), &K_EXPECTED_ROMANIZED_TITLE);
        }

        // test serialization to CSV
        #[test]
        fn test_to_csv() {
            let manga = make_default_model();
            let csv = manga.to_csv(true);
            // note that to_csv() appends "\n" at tail
            assert_eq!(&csv, &(K_MANGA_CSV_SORT_BY_URL.to_owned() + "\n"));
        }

        // test deserialization from CSV
        #[test]
        fn test_from_csv() {
            println!("K_MANGA_CSV:\n\t{}", K_MANGA_CSV);

            let manga = CsvMangaModel::from_csv(&K_MANGA_CSV).unwrap();
            println!("manga:\n\t{}", manga);

            assert_eq!(manga.title(), &K_MANGA_TITLE);
            assert_eq!(manga.romanized_title(), &K_EXPECTED_ROMANIZED_TITLE);
            assert_eq!(manga.url_with_chapters(), &K_MANGA_URL_WITH_CHAPTERS);
            assert_eq!(manga.chapter(), &K_MANGA_CHAPTER);
            assert_eq!(manga.last_modified(), &K_MANGA_LAST_MODIFIED);
            assert_eq!(manga.notes(), &K_MANGA_NOTES);
            assert_eq!(manga.tags(), &K_MANGA_TAGS);
        }
    }
}
