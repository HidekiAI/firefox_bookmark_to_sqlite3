//use kakasi;

// manga csv format
// "title","url", "url_with_chapters", "chapter","last_modified","notes", "tags"
pub mod model_csv_manga {
    use csv::Writer;
    use serde::{Deserialize, Serialize};
    use std::fmt::{self, Debug, Display};
    use std::io::Write;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CsvMangaModel {
        title: String,
        romanized_title: Option<String>,    // for V2
        url: Option<String>,            // for V2
        url_with_chapters: String,
        chapter: String,
        last_modified_YYYYmmddTHHMMSS: String,
        notes: String,
        tags: String,
    }

    impl fmt::Display for CsvMangaModel {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
            f,
            "Title: {}\nRomanized Title: {}\nURL: {}\nURL with Chapters: {}\nChapter: {}\nLast Modified: {}\nNotes: {}\nTags: {}",
            self.title,
            (match &self.romanized_title {
                Some(s) => s,
                None => "",
            }),
            (match &self.url {
                Some(s) => s,
                None => "",
            }),
            self.url_with_chapters,
            self.chapter,
            self.last_modified_YYYYmmddTHHMMSS,
            self.notes,
            self.tags,
        )
        }
    }

    impl CsvMangaModel {
        pub fn new(title: String) -> Self {
            let romanized_title = CsvMangaModel::romanized_url_with_chapters(&title);
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
            let romanized_title = CsvMangaModel::romanized_url_with_chapters(&title);
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
            let romanized_title = CsvMangaModel::romanized_url_with_chapters(&title);
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
                romanized_title: Some(CsvMangaModel::romanized_url_with_chapters( &bookmark_title.clone() ) ),
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
                    &self.url, // it's first-column so it' can be easily sorted
                    &self.title,
                    &self.romanized_title,
                    &self.url_with_chapters,
                    &self.chapter,
                    &self.last_modified_YYYYmmddTHHMMSS,
                    &self.notes,
                    &self.tags,
                ))
                .unwrap();
            } else {
                wtr.serialize((
                    &self.title,
                    &self.romanized_title,
                    &self.url_with_chapters,
                    &self.chapter,
                    &self.last_modified_YYYYmmddTHHMMSS,
                    &self.notes,
                    &self.tags,
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

            let record: CsvMangaModel = rdr.deserialize().next().unwrap()?;
            Ok(
                CsvMangaModel {
                title: record.title,
                romanized_title: record.romanized_title,
                url: record.url,
                url_with_chapters: record.url_with_chapters,
                chapter: record.chapter,
                last_modified_YYYYmmddTHHMMSS: record.last_modified_YYYYmmddTHHMMSS,
                notes: record.notes,
                tags: record.tags, 
            }
        )
        }

        pub fn title(&self) -> &String {
            &self.title
        }
        pub fn romanized_title(&self) -> &String {
            match self.romanized_title {
                Some(ref s) => s,
                None => &self.title,
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
        fn romanized_url_with_chapters(title: &String) -> String {
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

    impl fmt::Display for CsvMangaModelV1 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                //"Title: {}\nRomanized Title: {}\nURL: {}\nURL with Chapters: {}\nChapter: {}\nLast Modified: {}\nNotes: {}\nTags: {}",
                "Title:{}; URL with Chapters:{}; Chapter:{}; Last Modified:{}; Notes:{}; Tags:{}",
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
                "Title:{}; Romanized Title:{}; URL:{}; URL with Chapters:{}; Chapter:{}; Last Modified:{}; Notes:{}; Tags:{}",
                self.title,
                self.romanized_title,
                self.url,
                self.url_with_chapters,
                self.chapter,
                self.last_modified_YYYYmmddTHHMMSS,
                self.notes,
                self.tags
            )
        }
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
                        mangas.push(
                            CsvMangaModel::new_with_url(
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
                    None => &manga.title,
                });
                record.push_field(match &manga.url {
                    Some(s) => s,
                    None => &manga.url_with_chapters,
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
                None => &m.url_with_chapters,
            });
            record.push_field(match &m.romanized_title {
                Some(s) => s,
                None => &m.title,
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
                None => &m.url_with_chapters,
            });
            record.push_field(match &m.romanized_title {
                Some(s) => s,
                None => &m.title,
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

        #[test]
        fn test_title() {
            let manga = CsvMangaModel {
                title: String::from("One Piece"),
                romanized_title: Some(String::from("Wan Pīsu")),
                url: Some(String::from("https://example.com/manga/one-piece/")),
                url_with_chapters: String::from("https://example.com/manga/one-piece/"),
                chapter: String::from("1000"),
                last_modified_YYYYmmddTHHMMSS: String::from("2021-07-22T12:34:56"),
                notes: String::from(""),
                tags: String::from("action; adventure; comedy; drama; fantasy; shounen"), // NOTE: cannot have commmas inside strings for MOST CSV utilities fails to know the differences...
            };
            assert_eq!(manga.title(), "One Piece");
        }

        #[test]
        fn test_romanized_title() {
            // chose this title for a critical reason that if withinside quotes, there is a UTF8 comma ("、"), which is a problem for CSV if it was converted to ","
            let k_manga_title = String::from("ゲート―自衛隊彼の地にて、斯く戦えり");
            // notice that libkakasi converts "ゲート" to "geeto" and knows the difference between the dash in "ゲート" and the dash in "―" (which is a UTF8 dash), sadly it convers the UTF8 comma ("、") to a regular comma (","), in which we force it
            let k_expected_romanized_title =
                String::from("geeto ― jieitai kano chi nite、 kaku tatakae ri");
            let manga = CsvMangaModel {
                title: k_manga_title.clone(),
                romanized_title: Some(CsvMangaModel::romanized_url_with_chapters(&k_manga_title.clone())),
                url: Some(String::from("https://example.com/manga/one-piece/")),
                url_with_chapters: String::from("https://example.com/manga/gate/"),
                chapter: String::from("1000"),
                last_modified_YYYYmmddTHHMMSS: String::from("2021-07-22T12:34:56"),
                notes: String::from(""),
                tags: String::from("action; adventure; comedy; drama; fantasy; shounen"), // NOTE: cannot have commmas inside strings for MOST CSV utilities fails to know the differences...
            };
            assert_eq!(manga.romanized_title(), &k_expected_romanized_title);
        }

        // test serialization to CSV
        #[test]
        fn test_to_csv() {
            let manga = CsvMangaModel {
                title: String::from("One Piece"),
                romanized_title: Some(CsvMangaModel::romanized_url_with_chapters(&String::from( "One Piece"))),
                url: Some(String::from("https://example.com/manga/one-piece/")),
                url_with_chapters: String::from("https://example.com/manga/one-piece/"),
                chapter: String::from("1000"),
                last_modified_YYYYmmddTHHMMSS: String::from("2021-07-22T12:34:56"),
                notes: String::from(""),
                tags: String::from("action; adventure; comedy; drama; fantasy; shounen"), // NOTE: cannot have commmas inside strings for MOST CSV utilities fails to know the differences...
            };
            let csv = manga.to_csv(true);
            assert_eq!(
                csv,
                // note that there are no sppace between the fields (comma)
                "\"https://example.com/manga/one-piece/\",\"One Piece\",\"One Piece\",\"https://example.com/manga/one-piece/\",\"1000\",\"2021-07-22T12:34:56\",\"\",\"action; adventure; comedy; drama; fantasy; shounen\"\n"
            );
        }

        // test deserialization from CSV
        #[test]
        fn test_from_csv() {
            let csv = "\"One Piece\",\"One Piece\",\"https://example.com/manga/one-piece/\",\"https://example.com/manga/one-piece/\",\"1000\",\"2021-07-22T12:34:56\",\"\",\"action; adventure; comedy; drama; fantasy; shounen\"\n";
            let manga = CsvMangaModel::from_csv(csv).unwrap();
            assert_eq!(manga.title(), "One Piece");
            assert_eq!(
                manga.romanized_title(),
                &CsvMangaModel::romanized_url_with_chapters(&String::from("One Piece"))
            );
            assert_eq!(
                manga.url_with_chapters(),
                "https://example.com/manga/one-piece/"
            );
            assert_eq!(manga.chapter(), "1000");
            assert_eq!(manga.last_modified(), "2021-07-22T12:34:56");
            assert_eq!(manga.notes(), "");
            assert_eq!(
                manga.tags(),
                "action; adventure; comedy; drama; fantasy; shounen"
            );
        }
    }
}
