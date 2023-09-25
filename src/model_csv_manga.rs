use crate::model_manga;

pub mod model_csv_manga {
    use chrono::ParseError;
    use core::num;
    use csv::{DeserializeError, Error, Writer};
    use serde::{Deserialize, Serialize};
    use std::f32::consts::E;
    use std::fmt::{self, Debug, Display};
    use std::io::Write;

    use crate::model_manga;
    use crate::model_manga::model_manga::MangaModel;

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

    // NOTE: Display is used for serialization to string-based CSV,
    //       so you must keep it in format of "a,b,c,d" (raw CSV), and at the same
    //       time, we'll route through the getter accessor rather than directly
    //       so that Option based fields can be customized based on desired behaviours
    // Also assuming trait std::fmt::Display overrides CsvMangaModel::to_string()
    impl fmt::Display for CsvMangaModelV1 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
                self.title(),
                self.url_with_chapters(),
                self.chapter(),
                self.last_modified(),
                self.notes(),
                self.tags()
            )
        }
    }
    impl fmt::Display for CsvMangaModelV2 {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
                self.title(),
                self.url_with_chapters(),
                self.chapter(),
                self.last_modified(),
                self.notes(),
                self.tags(),
                self.romanized_title(),
                self.url(),
            )
        }
    }

    impl fmt::Display for CsvMangaModel {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
                self.title(),
                self.url_with_chapters(),
                self.chapter(),
                self.last_modified(),
                self.notes(),
                self.tags(),
                self.romanized_title(),
                self.url()
            )
        }
    }

    impl CsvMangaModel {
        pub fn from_epoch_to_str(epoch: i64) -> String {
            // convert the last_modified i64 to datetime - last_modified is encoded as unix epoch time in microseconds
            let from_epoch_timespan = chrono::NaiveDateTime::from_timestamp_opt(
                epoch / 1_000_000,
                (epoch % 1_000_000) as u32,
            )
            .unwrap();
            let last_modified_YYYYmmddTHHMMSS =
                from_epoch_timespan.format("%Y-%m-%dT%H:%M:%S").to_string();
            last_modified_YYYYmmddTHHMMSS
        }
        pub fn str_to_epoch_micros(time_YYYYmmddTHHMMSS: String) -> i64 {
            // convert the last_modified i64 to datetime - last_modified is encoded as unix epoch time in microseconds
            let timespan_YYYYmmddTHHMMSS = chrono::NaiveDateTime::parse_from_str(
                &time_YYYYmmddTHHMMSS,
                "%Y-%m-%dT%H:%M:%S",
            )
            .unwrap()
            .timestamp_micros();
            timespan_YYYYmmddTHHMMSS
        }
        pub fn new(model: &MangaModel) -> Self {
            let bookmark_last_modified_epoch_micros = match model.last_update {
                Some(ref s) => {
                    // convert the last_modified i64 to datetime - last_modified is encoded as unix epoch time in microseconds
                    CsvMangaModel::str_to_epoch_micros(s.clone())
                }
                None => 0,
            };

            // convert the last_modified i64 to datetime - last_modified is encoded as unix epoch time in microseconds
            let last_modified = CsvMangaModel::from_epoch_to_str(bookmark_last_modified_epoch_micros);
            // output: "uri_stripped_for_sorting","title","uri","chapter","last_modified","notes","tags"
            // extract chapter if link indicates so...

            let rom_title = match model.title_romanized {
                Some(ref s) => Some(CsvMangaModel::fix_comma_in_string(s.clone().as_str())),
                None => None,
            };
            CsvMangaModel {
                title: CsvMangaModel::fix_comma_in_string(model.title.as_str()),
                romanized_title: rom_title,
                url: Some(model.url.clone()),
                url_with_chapters: match model.url_with_chapter {
                    Some(ref s) => CsvMangaModel::fix_comma_in_string(s.clone().as_str()),
                    None => String::from(""),
                },
                chapter: match model.chapter {
                    Some(ref s) => CsvMangaModel::fix_comma_in_string(s.clone().as_str()),
                    None => String::from(""),
                },
                last_modified_YYYYmmddTHHMMSS: last_modified,
                notes: match model.notes {
                    Some(ref s) => CsvMangaModel::fix_comma_in_string(s.clone().as_str()),
                    None => String::from(""),
                },
                tags: match model.tags.len() > 0 {
                    true => CsvMangaModel::fix_comma_in_string(model.tags.join(",").as_str()),
                    false => String::from(""),
                },
            }
        }

        fn build_record_header() -> csv::StringRecord {
            let mut str_record = csv::StringRecord::new();
            str_record.push_field("Title");
            str_record.push_field("URL_with_Chapters");
            str_record.push_field("Chapter");
            str_record.push_field("Last_Modified");
            str_record.push_field("Notes");
            str_record.push_field("Tags");
            str_record.push_field("Possible_Romanized_Title");
            str_record.push_field("Romanized_Title");
            str_record
        }
        pub fn build_record_v1(&self) -> csv::StringRecord {
            let mut str_record = csv::StringRecord::new();
            str_record.push_field(self.title());
            str_record.push_field(self.url_with_chapters());
            str_record.push_field(self.chapter());
            str_record.push_field(self.last_modified());
            str_record.push_field(self.notes());
            str_record.push_field(self.tags());
            str_record.push_field(self.url()); // note that for v1, it's OK if this is "" empty-tring
            str_record.push_field(self.romanized_title());
            str_record
        }
        pub fn build_record(&self) -> csv::StringRecord {
            let mut str_record = csv::StringRecord::new();
            str_record.push_field(self.title());
            str_record.push_field(self.url_with_chapters());
            str_record.push_field(self.chapter());
            str_record.push_field(self.last_modified());
            str_record.push_field(self.notes());
            str_record.push_field(self.tags());
            str_record.push_field(self.url()); // for v2 and up, we want to make sure this is concrete
            str_record.push_field(self.romanized_title());
            str_record
        }
        pub fn build_record_update(&mut self) -> csv::StringRecord {
            let mut str_record = csv::StringRecord::new();
            str_record.push_field(self.title());
            str_record.push_field(self.url_with_chapters());
            str_record.push_field(self.chapter());
            str_record.push_field(self.last_modified());
            str_record.push_field(self.notes());
            str_record.push_field(self.tags());

            str_record.push_field(self.url_mut()); // for v2 and up, we want to make sure this is concrete
            str_record.push_field(self.romanized_title_mut());
            str_record
        }
        pub fn build_record_v2(&mut self) -> csv::StringRecord {
            let mut str_record = csv::StringRecord::new();
            str_record.push_field(self.title());
            str_record.push_field(self.url_with_chapters());
            str_record.push_field(self.chapter());
            str_record.push_field(self.last_modified());
            str_record.push_field(self.notes());
            str_record.push_field(self.tags());
            str_record.push_field(self.url_mut());
            str_record.push_field(self.romanized_title_mut());
            str_record
        }
        // serialize to CSV string (same as to_string()), only here
        // to accomodate the matching from_csv() function, but ideally,
        // better to just call CsvMangaModel::to_string() instead
        pub fn to_csv(&self) -> String /*csv*/ {
            self.to_string()
        }

        // desrialize from CSV string
        pub fn from_csv(csv: &str) -> Result<CsvMangaModel, Box<dyn std::error::Error>> {
            // a bit tricky on how to deserialize from CSV string, because order matters
            // we we'll have to guess the order of the fields and then deserialize
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(csv.as_bytes());

            let mut csv_model_des: CsvMangaModel = match rdr.deserialize().next() {
                Some(Ok(result_record)) => result_record,
                Some(Err(csv_error)) => {
                    let err_msg = format!("Error: {}", csv_error.to_string());
                    return Err(Box::from(err_msg));
                }
                None => {
                    let err_msg = "Error: could not deserialize".to_string();
                    return Err(Box::from(err_msg));
                }
            };
            #[cfg(debug_assertions)]
            {
                // using Debug {:?} will render Option::None as None
                println!("\n>> {:?}", csv_model_des);
                //println!(">> {}\n", record);
            }
            // fail immediately if the CSV is not in the right format
            // for now, the two that could possibly be used is chapter (as int) and last_modified (as datetime)
            // so, we'll check for those two fields
            let binding = csv_model_des.chapter().clone();
            let ch: &str = binding.as_str();
            let lm_epoch = CsvMangaModel::str_to_epoch_micros(csv_model_des.last_modified().clone());
            let mut ch_removed_extra = -1;
            if (ch.contains(".") || ch.contains("-")) {
                // strip or keep all chars only up to "." or "-" so we can parse it as int
                let ch = ch
                    .split('.')
                    .next()
                    .unwrap()
                    .split('-')
                    .next()
                    .unwrap()
                    .to_string();
                // now  parse and make sure it's an integer, if not error out
                ch_removed_extra = match ch.parse::<i32>() {
                    Ok(num) => num,
                    Err(parse_error) => {
                        let err_msg = format!("Error: {}", parse_error.to_string());
                        return Err(Box::from(err_msg));
                    }
                };
            }

            // assume format is probably fine (for now)
            let model = CsvMangaModel {
                title: csv_model_des.title().into(),
                url_with_chapters: csv_model_des.url_with_chapters().into(),
                chapter: csv_model_des.chapter().into(),
                last_modified_YYYYmmddTHHMMSS: csv_model_des.last_modified().into(),
                notes: csv_model_des.notes().into(),
                tags: csv_model_des.tags().into(),
                url: Some(csv_model_des.url_mut().into()),
                romanized_title: Some(csv_model_des.romanized_title_mut().into()),
            };
            //let record = model.build_record();
            Ok(model)
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
            CsvMangaModel::str_to_epoch_micros( self.last_modified_YYYYmmddTHHMMSS.clone())
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
    impl CsvMangaModelV1 {
        // getter/accessor for the fields
        pub fn title(&self) -> &String {
            &self.title
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
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CsvMangaModelV2 {
        // the two new fields for sroting are at the head so that post-sorting, you can 'cut -d',' -f3-' to return back to v1
        // Note that on this model, url and romanize_title are concrete rather than Some(s) since it's NEVER assumed to be None
        #[serde(rename = "URL")]
        #[serde(default)]
        // quite critical that you have this for any/almost-all serde elements that are Option type
        url: String, // URL without the chapter (mainly for sorting purposes)

        #[serde(rename = "Romanized_Title")]
        #[serde(default)]
        // quite critical that you have this for any/almost-all serde elements that are Option type
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
    impl CsvMangaModelV2 {
        // getter/accessor for the fields
        pub fn title(&self) -> &String {
            &self.title
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
        pub fn romanized_title(&self) -> &String {
            &self.romanized_title
        }
        pub fn url(&self) -> &String {
            &self.url
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
        pub fn read_csv(input_reader: Box<dyn std::io::Read>) -> Vec<MangaModel> {
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(input_reader);

            let mut mangas: Vec<MangaModel> = Vec::new();
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
                        let model: MangaModel = MangaModel::new_from_required_elements(
                            record.title,
                            record.url_with_chapters.clone(),
                            model_manga::CASTAGNOLI
                                .checksum(record.url_with_chapters.clone().as_bytes()),
                        );
                        mangas.push(model);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            mangas
        }

        pub fn write_csv(&mut self, mangas: &Vec<MangaModel>) -> Result<(), csv::Error> {
            for manga in mangas {
                let csv_manga_model = CsvMangaModel::new(manga);
                let record = csv_manga_model.build_record();
                self.csv_writer.write_record(&record)?;
            }
            Ok(())
        }

        pub fn write_csv_header(&mut self) -> Result<(), csv::Error> {
            let mut record = CsvMangaModel::build_record_header();
            self.csv_writer.write_record(&record)
        }

        pub fn record_bookmark(
            &mut self,
            bookmark_last_modified_epoch_micros: i64,
            bookmark_uri: &String,
            bookmark_title: &String,
        ) -> CsvMangaModel {
            let mut mm: MangaModel = MangaModel::new_from_required_elements(
                bookmark_title.clone(),
                bookmark_uri.clone(),
                model_manga::CASTAGNOLI.checksum(bookmark_uri.as_bytes()),
            );
            mm.last_update = Some(CsvMangaModel::from_epoch_to_str(
                bookmark_last_modified_epoch_micros,
            ));

            let m = CsvMangaModel::new(&mm);
            let mut record = m.build_record();
            // write it
            self.csv_writer.write_record(&record).unwrap();
            m
        }

        //      ""2023-09-21T18:54:30"https://rawkuma.com/overlord-chapter-1/"2023-09-21T18:54:30", "1/"2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30", " "2023-09-21T18:54:30"https://rawkuma.com/overlord", "oobaaroodo"
        //      ""2023-09-21T18:54:30"https://rawkuma.com/overlord-chapter-1/"2023-09-21T18:54:30", "1/"2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30", " "2023-09-21T18:54:30"https://rawkuma.com/overlord", "Overlord"
        //      ""2023-09-21T18:54:30"https://rawkuma.com/overlord-chapter-2/"2023-09-21T18:54:30", "2/"2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30", " "2023-09-21T18:54:30"https://rawkuma.com/overlord", "Overlord"
        //      "https://rawkuma.com/overlord-chapter-1/", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "#", "https://rawkuma.com/overlord/", "Overlord"
        //      "https://rawkuma.com/overlord-chapter-2/", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "#", "https://rawkuma.com/overlord/", "Overlord"
        //      " ""https://rawkuma.com/overlord","oobaaroodo","オーバーロード"," ""https://rawkuma.com/overlord-chapter-1/""","1/""","","",""
        //      "https://rawkuma.com/overlord/","Overlord Chapter 1 Raw - Rawkuma","https://rawkuma.com/overlord-chapter-1/","1","2023-09-16T03:00:05","","#"
        //      "https://rawkuma.com/overlord/","Overlord Chapter 1 Raw - Rawkuma","Overlord Chapter 1 Raw - Rawkuma","https://rawkuma.com/overlord-chapter-1/","1","2023-09-16T03:00:05","","#"
        //      "https://rawkuma.com/overlord/","Overlord Chapter 2 Raw - Rawkuma","https://rawkuma.com/overlord-chapter-2/","2","2023-09-16T02:57:37","","#"
        //      "https://rawkuma.com/overlord/","Overlord Chapter 2 Raw - Rawkuma","Overlord Chapter 2 Raw - Rawkuma","https://rawkuma.com/overlord-chapter-2/","2","2023-09-16T02:57:37","","#"
        //      " ""https://rawkuma.com/overlord","Overlord","Overlord"," ""https://rawkuma.com/overlord-chapter-1/""","1/""","","",""
        //      " ""https://rawkuma.com/overlord","Overlord","Overlord"," ""https://rawkuma.com/overlord-chapter-2/""","2/""","","",""
        //      "oobaaroodo", "オーバーロード", " "2023-09-21T18:54:30"https://rawkuma.com/overlord-chapter-1/"2023-09-21T18:54:30", "1/"2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30"
        //      "Overlord Chapter 1 Raw - Rawkuma","https://rawkuma.com/overlord-chapter-1/","1","2023-09-16T03:00:05","","#","https://rawkuma.com/overlord/","Overlord Chapter 1 Raw - Rawkuma"
        //      "Overlord Chapter 2 Raw - Rawkuma","https://rawkuma.com/overlord-chapter-2/","2","2023-09-16T02:57:37","","#","https://rawkuma.com/overlord/","Overlord Chapter 2 Raw - Rawkuma"
        //      "Overlord"," ""https://rawkuma.com/overlord-chapter-1/""","1/""","","",""," ""https://rawkuma.com/overlord","Overlord"
        //      "Overlord", "https://rawkuma.com/overlord-chapter-1/", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "#"
        //      "Overlord", "https://rawkuma.com/overlord-chapter-2/", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "#"
        //      "Overlord"," ""https://rawkuma.com/overlord-chapter-2/""","2/""","","",""," ""https://rawkuma.com/overlord","Overlord"
        //      "Overlord", "Overlord", " "2023-09-21T18:54:30"https://rawkuma.com/overlord-chapter-1/"2023-09-21T18:54:30", "1/"2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30"
        //      "Overlord", "Overlord", " "2023-09-21T18:54:30"https://rawkuma.com/overlord-chapter-2/"2023-09-21T18:54:30", "2/"2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30"
        //      "Overlord", "Overlord", "https://rawkuma.com/overlord-chapter-1/", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "#"
        //      "Overlord", "Overlord", "https://rawkuma.com/overlord-chapter-2/", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "2023-09-21T18:54:30", "#"
        //      "オーバーロード"," ""https://rawkuma.com/overlord-chapter-1/""","1/""","","",""," ""https://rawkuma.com/overlord","oobaaroodo"
        pub fn record(&mut self, m: &mut MangaModel) {
            let mut c = CsvMangaModel::new(m);
            let r = c.build_record_update();
            // write it
            self.csv_writer.write_record(&r).unwrap();
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
            let csv = manga.to_csv();
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
