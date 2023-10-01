pub mod model_csv_manga {
    use csv::{Error, Writer};
    use serde::{Deserialize, Serialize};
    use std::fmt::{self};
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
        // Note: the serde::rename is based off of MangaModel struct field names, in which should also match SQLite3 column names
        #[serde(rename = "title")]
        // Specify the CSV column name so that we can keep fields private
        title: String, // UTF8 encoded

        // Note: all new varaibles have to be Option type AND must be appended to the end of the struct // All variables that are Option type should get serialized to Some("") so that writer can make
        // sure to pack it (i.e. "a",,,"d",,"f") - Option type is only for the sake of missing data
        // on older version of the CSV
        #[serde(rename = "title_romanized")] // Specify the CSV column name so that we can keep fields private
        #[serde(default)] // quite critical that you have this for any/almost-all serde elements that are Option type
        #[serde(deserialize_with = "fn_deserialize_option_string")]
        // this is the custom deserializer for Option<String>
        title_romanized: Option<String>, // is Some() ONLY if title was in Japanese

        #[serde(rename = "url")] // Specify the CSV column name so that we can keep fields private
        url: String, // home page of manga (see impl of to_url and from_url validation)

        #[serde(rename = "url_with_chapter")] // Specify the CSV column name so that we can keep fields private
        #[serde(default)] // quite critical that you have this for any/almost-all serde elements that are Option type
        #[serde(deserialize_with = "fn_deserialize_option_string")]
        // this is the custom deserializer for Option<String>
        url_with_chapter: Option<String>, // last read/updated chapter

        #[serde(rename = "chapter")] // Specify the CSV column name so that we can keep fields private
        #[serde(default)] // quite critical that you have this for any/almost-all serde elements that are Option type
        #[serde(deserialize_with = "fn_deserialize_option_string")]
        // this is the custom deserializer for Option<String>
        chapter: Option<String>, // last read/updated chapter

        #[serde(rename = "last_update")] // Specify the CSV column name so that we can keep fields private
        #[serde(default)] // quite critical that you have this for any/almost-all serde elements that are Option type
        #[serde(deserialize_with = "fn_deserialize_option_string")]
        // this is the custom deserializer for Option<String>
        last_update_yyyymmdd_thhmmss: Option<String>,

        #[serde(rename = "notes")] // Specify the CSV column name so that we can keep fields private
        #[serde(default)] // quite critical that you have this for any/almost-all serde elements that are Option type
        #[serde(deserialize_with = "fn_deserialize_option_string")]
        // this is the custom deserializer for Option<String>
        notes: Option<String>,

        #[serde(rename = "tags")] // Specify the CSV column name so that we can keep fields private
        #[serde(default)] // quite critical that you have this for any/almost-all serde elements that are Option type
        #[serde(deserialize_with = "fn_deserialize_option_string")]
        // this is the custom deserializer for Option<String>
        tags: Option<String>, // i.e. "#アニメ化" ; empty vec[] is same as None

        #[serde(rename = "my_anime_list")] // Specify the CSV column name so that we can keep fields private
        #[serde(default)] // quite critical that you have this for any/almost-all serde elements that are Option type
        #[serde(deserialize_with = "fn_deserialize_option_string")]
        // this is the custom deserializer for Option<String>
        my_anime_list: Option<String>, // provides author and artist
    }

    // NOTE: Display is used for serialization to string-based CSV,
    //       so you must keep it in format of "a,b,c,d" (raw CSV), and at the same
    //       time, we'll route through the getter accessor rather than directly
    //       so that Option based fields can be customized based on desired behaviours
    // Also assuming trait std::fmt::Display overrides CsvMangaModel::to_string()
    // SELECT
    // --  m.id,
    //     m.title,
    //     m.title_romanized,
    //     m.url,
    //     m.url_with_chapter,
    //     m.chapter,
    //     m.last_update,
    //     m.notes,
    //     m.my_anime_list,
    //     (SELECT GROUP_CONCAT(t.tag, ', ')
    //         FROM manga_to_tags_map AS mt
    //         JOIN tags AS t ON mt.tag_id = t.id
    //         WHERE mt.manga_id = m.id) AS tags
    // FROM manga AS m;
    impl fmt::Display for CsvMangaModel {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"",
                self.title(),
                self.title_romanized(),
                self.url(),
                self.url_with_chapter(),
                self.chapter(),
                self.last_update(),
                self.notes(),
                self.tags(),
            )
        }
    }

    impl CsvMangaModel {
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
        pub fn str_to_epoch_micros(time_yyyymmdd_thhmmss: String) -> i64 {
            // convert the last_update i64 to datetime - last_update is encoded as unix epoch time in microseconds
            let timespan_yyyymmdd_thhmmss =
                chrono::NaiveDateTime::parse_from_str(&time_yyyymmdd_thhmmss, "%Y-%m-%dT%H:%M:%S")
                    .unwrap()
                    .timestamp_micros();
            timespan_yyyymmdd_thhmmss
        }
        fn fix_comma_in_string(s: &str) -> String {
            Utils::fix_comma_in_string(s)
        }
        // only exposing this function so that use-depencies of kakasi will be limited to this module only
        // but mainly, also want to preserve at least the UTF8 comma ("、") in the title
        fn romanized(title: &str) -> String {
            CsvMangaModel::fix_comma_in_string(kakasi::convert(title).romaji.as_str())
        }

        pub fn get_last_update(&self) -> i64 {
            CsvMangaModel::str_to_epoch_micros(self.last_update().clone().to_string())
        }
        fn strip_chapter_from_url(url_with_chapters: String) -> (String, String) {
            Utils::strip_chapter_from_url(&url_with_chapters)
        }
        pub fn new(model: &MangaModel) -> Self {
            let bookmark_last_update_epoch_micros = match model.last_update() {
                Some(ref s) => {
                    // convert the last_update i64 to datetime - last_update is encoded as unix epoch time in microseconds
                    CsvMangaModel::str_to_epoch_micros(s.to_string().clone())
                }
                None => 0,
            };

            // convert the last_update i64 to datetime - last_update is encoded as unix epoch time in microseconds
            let last_update = CsvMangaModel::from_epoch_to_str(bookmark_last_update_epoch_micros);
            // output: "uri_stripped_for_sorting","title","uri","chapter","last_update","notes","tags"
            // extract chapter if link indicates so...

            CsvMangaModel {
                title: CsvMangaModel::fix_comma_in_string(model.title()),
                title_romanized: match model.title_romanized() {
                    Some(ref s) => Some(CsvMangaModel::fix_comma_in_string(s.as_str())),
                    None => None,
                },
                url: CsvMangaModel::fix_comma_in_string(&model.url()),
                url_with_chapter: match model.url_with_chapter() {
                    Some(ref s) => Some(CsvMangaModel::fix_comma_in_string(s.as_str())), // pretty sure commas are illegal in URLs, but just in case
                    None => None,
                },
                chapter: match model.chapter() {
                    Some(ref s) => Some(CsvMangaModel::fix_comma_in_string(s.as_str())),
                    None => None,
                },
                last_update_yyyymmdd_thhmmss: Some(last_update.to_string()),
                notes: match model.notes() {
                    Some(ref s) => Some(CsvMangaModel::fix_comma_in_string(s.as_str())),
                    None => None,
                },
                tags: match model.tags().len() > 0 {
                    true => Some(CsvMangaModel::fix_comma_in_string(
                        model.tags().join(";").as_str(),
                    )), // NOTE: Using ';' instead of ',' for tags
                    false => None,
                },
                my_anime_list: match model.my_anime_list() {
                    Some(ref s) => Some(CsvMangaModel::fix_comma_in_string(s.as_str())),
                    None => None,
                },
            }
        }

        fn build_record_header() -> csv::StringRecord {
            let mut str_record = csv::StringRecord::new();
            str_record.push_field("Title");
            str_record.push_field("URL_with_Chapters");
            str_record.push_field("Chapter");
            str_record.push_field("last_update");
            str_record.push_field("Notes");
            str_record.push_field("Tags");
            str_record.push_field("Possible_Romanized_Title");
            str_record.push_field("Romanized_Title");
            str_record
        }
        pub fn build_record_v1(&self) -> csv::StringRecord {
            let mut str_record = csv::StringRecord::new();
            str_record.push_field(self.title());
            str_record.push_field(self.url_with_chapter());
            str_record.push_field(self.chapter());
            str_record.push_field(self.last_update());
            str_record.push_field(self.notes());
            str_record.push_field(self.tags());
            str_record.push_field(self.url()); // note that for v1, it's OK if this is "" empty-tring
            str_record.push_field(self.title_romanized());
            str_record
        }
        pub fn build_record(&self) -> csv::StringRecord {
            let mut str_record = csv::StringRecord::new();
            str_record.push_field(self.title());
            str_record.push_field(self.url_with_chapter());
            str_record.push_field(self.chapter());
            str_record.push_field(self.last_update());
            str_record.push_field(self.notes());
            str_record.push_field(self.tags());
            str_record.push_field(self.url()); // for v2 and up, we want to make sure this is concrete
            str_record.push_field(self.title_romanized());
            str_record
        }
        pub fn build_record_update(&mut self) -> csv::StringRecord {
            let mut str_record = csv::StringRecord::new();
            str_record.push_field(self.title());
            str_record.push_field(self.url_with_chapter());
            str_record.push_field(self.chapter());
            str_record.push_field(self.last_update());
            str_record.push_field(self.notes());
            str_record.push_field(self.tags());

            str_record.push_field(self.url()); // for v2 and up, we want to make sure this is concrete
            str_record.push_field(self.title_romanized());
            str_record
        }
        pub fn build_record_v2(&mut self) -> csv::StringRecord {
            let mut str_record = csv::StringRecord::new();
            str_record.push_field(self.title());
            str_record.push_field(self.url_with_chapter());
            str_record.push_field(self.chapter());
            str_record.push_field(self.last_update());
            str_record.push_field(self.notes());
            str_record.push_field(self.tags());
            str_record.push_field(self.url());
            str_record.push_field(self.title_romanized());
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
            let mut rdr: csv::Reader<&[u8]> = csv::ReaderBuilder::new()
                .has_headers(false) // without this, it'll ignore the first line, let alone if there is only one row, it will become empty record!
                .escape(Some(b'\\')) // rather than ("") ours use (\") to represent embedded quotes
                .comment(Some(b'#')) // allow # to be on first column to indicate comments
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
            // for now, the two that could possibly be used is chapter (as int) and last_update (as datetime)
            // so, we'll check for those two fields
            let binding = csv_model_des.chapter().clone();
            let ch: &str = binding;
            let lm_epoch =
                CsvMangaModel::str_to_epoch_micros(csv_model_des.last_update().clone().to_string());
            let mut ch_removed_extra = -1;
            if ch.contains(".") || ch.contains("-") {
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

            //let model = CsvMangaModel {
            //    title: csv_model_des.title().into(),
            //    url_with_chapters: csv_model_des.url_with_chapters().into(),
            //    chapter: csv_model_des.chapter().into(),
            //    last_update_YYYYmmddTHHMMSS: csv_model_des.last_update().into(),
            //    notes: csv_model_des.notes().into(),
            //    tags: csv_model_des.tags().into(),
            //    url: Some(csv_model_des.url_mut().into()),
            //    romanized_title: Some(csv_model_des.romanized_title_mut().into()),
            //};
            // once deserialize, make it into MangaModel
            match MangaModel::new_from_required_elements(
                csv_model_des.title().clone(),
                csv_model_des.url_with_chapter().clone(),
                model_manga::CASTAGNOLI.checksum(csv_model_des.url_with_chapter().as_bytes()),
            ) {
                Ok(mut model) => {
                    model.set_last_update(Some(csv_model_des.last_update().clone().to_string()));
                    model.set_notes(Some(csv_model_des.notes().clone().to_string()));
                    model.set_tags(
                        csv_model_des
                            .tags()
                            .split(';')
                            .map(|s| s.to_string())
                            .collect(),
                    );

                    //let record = model.build_record();
                    Ok(CsvMangaModel::new(&model))
                }
                Err(e) => {
                    let err_msg = format!("Error: {}", e.to_string());
                    return Err(Box::from(err_msg));
                }
            }
        }

        // accessor to private data
        pub fn title(&self) -> &str {
            &self.title
        }
        pub fn title_romanized(&self) -> &str {
            match self.title_romanized {
                Some(ref s) => s, // found it!
                None => &self.title,
            }
        }
        pub fn url(&self) -> &str {
            &self.url
        }
        pub fn url_with_chapter(&self) -> &str {
            match self.url_with_chapter {
                Some(ref s) => s.as_str(),
                None => &self.url,
            }
        }
        pub fn chapter(&self) -> &str {
            match &self.chapter {
                Some(s) => s.as_str(),
                None => "",
            }
        }
        pub fn last_update(&self) -> &str {
            match &self.last_update_yyyymmdd_thhmmss {
                Some(s) => &s.as_str(),
                None => "",
            }
        }
        pub fn notes(&self) -> &str {
            match &self.notes {
                Some(s) => &s.as_str(),
                None => "",
            }
        }
        pub fn tags(&self) -> &str {
            match &self.tags {
                Some(s) => &s,
                None => "",
            }
        }
        pub fn my_anime_list(&self) -> &str {
            match &self.my_anime_list {
                Some(s) => &s,
                None => "",
            }
        }

        // modifier to private data
        pub fn set_title(&mut self, title: String) {
            self.title = title;
        }
        pub fn set_title_romanized(&mut self, title_romanized: Option<String>) {
            self.title_romanized = title_romanized;
        }
        pub fn set_url(&mut self, url: String) {
            self.url = url;
        }
        pub fn set_url_with_chapters(&mut self, url_with_chapters: Option<String>) {
            self.url_with_chapter = url_with_chapters;
        }
        pub fn set_chapter(&mut self, chapter: Option<String>) {
            self.chapter = chapter;
        }
        pub fn set_last_update(&mut self, last_update: Option<String>) {
            self.last_update_yyyymmdd_thhmmss = last_update;
        }
        pub fn set_notes(&mut self, notes: Option<String>) {
            self.notes = notes;
        }
        pub fn set_tags(&mut self, tags: Option<String>) {
            self.tags = tags;
        }
        pub fn set_my_anime_list(&mut self, my_anime_list: Option<String>) {
            self.my_anime_list = my_anime_list;
        }

        //pub fn romanized_title_mut(&mut self) -> &str {
        //    match self.title_romanized {
        //        Some(ref s) => s, // found it!
        //        None =>
        //        // String::from(""),    // empty string
        //        {
        //            self.set_romanized_title ( Some(CsvMangaModel::romanized(&self.title)));
        //            &self.romanized_title().as_ref().unwrap()
        //        } // alternative
        //    }
        //}
        //pub fn romanized_title(&self) -> &str {
        //    match self.title_romanized {
        //        Some(ref s) => s, // found it!
        //        None => &self.title,
        //    }
        //}
        //// for here, similar to romanized_title(), but for alternative, urll_with_chapters
        //pub fn url_mut(&mut self) -> &str {
        //    match self.url {
        //        Some(ref s) => s,
        //        None =>
        //        //String::from(""), // empty string
        //        {
        //            let (alternative, _) =
        //                CsvMangaModel::strip_chapter_from_url(self.url_with_chapters.clone());
        //            self.url = Some(alternative);
        //            &self.url.as_ref().unwrap() // alternative
        //        }
        //    }
        //}
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CsvMangaModelV1 {
        #[serde(rename = "Title")]
        title: String,

        #[serde(rename = "URL_with_Chapters")]
        url_with_chapters: String,

        #[serde(rename = "Chapter")]
        chapter: String,

        #[serde(rename = "last_update")]
        last_update_yyyymmdd_thhmmss: String, // format is "%Y-%m-%dT%H:%M:%S"

        #[serde(rename = "Notes")]
        notes: String,

        #[serde(rename = "Tags")]
        tags: String,
    }
    impl CsvMangaModelV1 {
        // getter/accessor for the fields
        pub fn title(&self) -> &str {
            &self.title
        }
        pub fn url_with_chapters(&self) -> &str {
            &self.url_with_chapters
        }
        pub fn chapter(&self) -> &str {
            &self.chapter
        }
        pub fn last_update(&self) -> &str {
            &self.last_update_yyyymmdd_thhmmss
        }
        pub fn notes(&self) -> &str {
            &self.notes
        }
        pub fn tags(&self) -> &str {
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

        #[serde(rename = "last_update")]
        last_update_yyyymmdd_thhmmss: String, // format is "%Y-%m-%dT%H:%M:%S"

        #[serde(rename = "Notes")]
        notes: String,

        #[serde(rename = "Tags")]
        tags: String,
    }
    impl CsvMangaModelV2 {
        // getter/accessor for the fields
        pub fn title(&self) -> &str {
            &self.title
        }
        pub fn url_with_chapters(&self) -> &str {
            &self.url_with_chapters
        }
        pub fn chapter(&self) -> &str {
            &self.chapter
        }
        pub fn last_update(&self) -> &str {
            &self.last_update_yyyymmdd_thhmmss
        }
        pub fn notes(&self) -> &str {
            &self.notes
        }
        pub fn tags(&self) -> &str {
            &self.tags
        }
        pub fn romanized_title(&self) -> &str {
            &self.romanized_title
        }
        pub fn url(&self) -> &str {
            &self.url
        }
    }

    pub struct Utils {
        //csv_writer: Writer<Box<dyn Write + 'static>>, // mutable reference to a trait object
        csv_writer: Writer<Box<dyn Write>>, // mutable reference to a trait object

        // iterator for reading each CSV rows that is mutable
        //csv_reader: csv::Reader<Box<dyn std::io::Read + 'static>>,
        csv_reader: csv::Reader<Box<dyn std::io::Read>>,
    }
    impl Drop for Utils {
        fn drop(&mut self) {
            // flush the csv_writer
            self.csv_writer.flush().unwrap();
        }
    }

    impl Utils {
        // pass in writer, such as stdout or file
        pub fn new(
            possible_output_writer: Option<Box<dyn Write>>,
            input_reader: Box<dyn std::io::Read>,
        ) -> Utils {
            // fail immediately if output_writer is not a streamable writer
            Utils {
                csv_writer: match possible_output_writer {
                    Some(output_writer) => csv::WriterBuilder::new()
                        .quote_style(csv::QuoteStyle::Always) // just easier to just quote everything including numbers
                        .from_writer(output_writer),
                    None => csv::WriterBuilder::new()
                        .quote_style(csv::QuoteStyle::Always) // just easier to just quote everything including numbers
                        .from_writer(Box::new(std::io::stdout())), // just dump it out to stdout
                },
                csv_reader: csv::ReaderBuilder::new()
                    .has_headers(false) // without this, it'll ignore the first line, let alone if there is only one row, it will become empty record!
                    .escape(Some(b'\\')) // rather than ("") ours use (\") to represent embedded quotes
                    .comment(Some(b'#')) // allow # to be on first column to indicate comments
                    .from_reader(input_reader),
            }
        }

        // reset iterator by setting new input_reader
        pub fn reset(&mut self, input_reader: Box<dyn std::io::Read>) {
            self.csv_reader = csv::ReaderBuilder::new()
                .has_headers(false) // without this, it'll ignore the first line, let alone if there is only one row, it will become empty record!
                .escape(Some(b'\\')) // rather than ("") ours use (\") to represent embedded quotes
                .comment(Some(b'#')) // allow # to be on first column to indicate comments
                .from_reader(input_reader);
        }

        // iterator rdr to next row for deserializing
        pub fn next(&mut self) -> Option<Result<MangaModel, Error>> {
            let result = self.csv_reader.deserialize().next(); // possibly use csv_reader.records() instead?
                                                               // return as MangaModel IF we've not reached the end of stream (None if end of stream)
            match result {
                Some(Ok(deserialized_record)) => {
                    // by explicitly casting to CsvMangaModel, we can get the deserialized record (very weird approach)
                    let csv_manga_model_record: CsvMangaModel = deserialized_record;
                    #[cfg(debug_assertions)]
                    {
                        println!(">> csv::next: CsvMangaRecord({:?})", csv_manga_model_record);
                    }

                    // push a copy
                    match MangaModel::new_from_required_elements(
                        csv_manga_model_record.title().clone(),
                        csv_manga_model_record.url_with_chapter().clone(),
                        model_manga::CASTAGNOLI
                            .checksum(csv_manga_model_record.url_with_chapter().clone().as_bytes()),
                    ) {
                        Ok(mut m) => {
                            m.set_last_update(Some(
                                csv_manga_model_record.last_update().clone().to_string(),
                            ));
                            m.set_notes(Some(csv_manga_model_record.notes().clone().to_string()));
                            m.set_tags(
                                csv_manga_model_record
                                    .tags()
                                    .clone()
                                    .split(';')
                                    .map(|s| s.trim().to_string())
                                    .collect::<Vec<String>>(),
                            );
                            //m.my_anime_list = csv_manga_model_record.m
                            #[cfg(debug_assertions)]
                            {
                                println!(">>> csv::next: MangaModel({:?})", m);
                            }
                            return Some(Ok(m)); // brute-force bail out immediately here
                        }
                        Err(e) => {
                            println!("Error:read_next(): could not create MangaModel from CSV record - {}", e);
                            return None; // brute-force bail out immediately here
                        }
                    };
                }
                Some(Err(e)) => {
                    eprintln!("Error: {}", e);
                    None
                }
                None => None,
            }
        }

        pub fn fix_comma_in_string(s: &str) -> String {
            // NOTE: cannot have commmas inside strings for MOST CSV utilities fails to know the differences...
            // so, we need to replace all commas with something else, such as "、"
            s.replace(",", "、")
        }

        pub fn strip_chapter_from_url(
            url_with_chapters: &str,
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
            let mut uri_stripped = url_with_chapters.clone().to_string(); //
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
            let mut util = Utils::new(None, input_reader); // should auto-reset to head of stream, so no need to reset()
            let mut mangas: Vec<MangaModel> = Vec::new();
            //mangas.extend(rdr.deserialize::<Manga>());
            //for result in rdr.deserialize() {
            for result in util.next() {
                match result {
                    Ok(result_record) => {
                        let record = CsvMangaModel::new(&result_record);
                        #[cfg(debug_assertions)]
                        {
                            //
                        }

                        // push a copy
                        mangas.push(result_record);
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
            bookmark_last_update_epoch_micros: i64,
            bookmark_uri: &str,
            bookmark_title: &str,
        ) -> Option<CsvMangaModel> {
            match MangaModel::new_from_required_elements(
                bookmark_title.clone(),
                bookmark_uri.clone(),
                model_manga::CASTAGNOLI.checksum(bookmark_uri.as_bytes()),
            ) {
                Ok(mut mm) => {
                    mm.set_last_update(Some(CsvMangaModel::from_epoch_to_str(
                        bookmark_last_update_epoch_micros,
                    )));

                    let m = CsvMangaModel::new(&mm);
                    let mut record = m.build_record();
                    // write it
                    self.csv_writer.write_record(&record).unwrap();
                    Some(m)
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    None
                }
            }
        }

        pub fn record(&mut self, m: &mut MangaModel) {
            let mut c = CsvMangaModel::new(m);
            let r = c.build_record_update();
            // write it
            self.csv_writer.write_record(&r).unwrap();
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::{
            model_csv_manga::model_csv_manga::CsvMangaModel,
            model_manga::{self, model_manga::MangaModel},
        };

        // chose this title for a critical reason that if withinside quotes, there is a UTF8 comma ("、"), which is a problem for CSV if it was converted to ","
        const K_MANGA_TITLE: &str = "ゲート―自衛隊彼の地にて、斯く戦えり";
        // notice that libkakasi converts "ゲート" to "geeto" and knows the difference between the dash in "ゲート" and the dash in "―" (which is a UTF8 dash), sadly it convers the UTF8 comma ("、") to a regular comma (","), in which we force it
        const K_EXPECTED_ROMANIZED_TITLE: &str = "geeto ― jieitai kano chi nite、 kaku tatakae ri"; // NOTE: the UTF8 comma ("、") is converted to a regular comma (",")
        const K_MANGA_URL: &str = "https://example.com/manga/gate/";
        const K_MANGA_URL_WITH_CHAPTERS: &str = "https://example.com/manga/gate-chapter-10/";
        const K_MANGA_CHAPTER: &str = "10";
        const K_MANGA_LAST_UPDATE: &str = "2021-07-22T12:34:56";
        const K_MANGA_NOTES_RAW: &str    = "Notes may have commas, (<- this will be replaced) but they will get replaced with \"、\" - this also tests for double-quotes issues";
        const K_MANGA_NOTES_FIXED: &str = "Notes may have commas、 (<- this will be replaced) but they will get replaced with \"、\" - this also tests for double-quotes issues";
        const K_MANGA_TAGS_SEMICOLON_SEPARATED: &str = "#action; #isekai; #fantasy; #shounen";
        const K_MANGA_MY_ANIME_LIST_LINK: &str = ""; // usually is empty
                                                     // note that this version needs to append "\n" at tail separately
        fn quoted(s: &str) -> String {
            format!("\"{}\"", s)
        }
        fn make_csv_test_string() -> String {
            //const K_MANGA_CSV_RAW_V1: &str = "\"ゲート―自衛隊彼の地にて、斯く戦えり\",\"https://example.com/manga/gate-chapter-10/\",\"10\",\"2021-07-22T12:34:56\",\"Notes may have commas, (<- this will be replaced) but they will get replaced with \"、\"\",\"#action; #isekai; #fantasy; #shounen\"";
            const K_MANGA_CSV_RAW_V2: &str = "\"ゲート―自衛隊彼の地にて、斯く戦えり\",\"geeto ― jieitai kano chi nite, kaku tatakae ri\",https://rawkuma.com/gate-jietai-kare-no-chi-nite-kaku-tatakeri-01/,https://rawkuma.com/gate-jietai-kare-no-chi-nite-kaku-tatakeri-01/,\"\",\" 2023-09-06T13:57:22\",\" -\",\"\",\"アニメ化, +1\"";
            let mut csv = String::new();
            csv.push_str(&quoted(K_MANGA_TITLE));
            csv.push(',');
            csv.push_str(&quoted(K_EXPECTED_ROMANIZED_TITLE));
            csv.push(',');
            csv.push_str(&quoted(K_MANGA_URL));
            csv.push(',');
            csv.push_str(&quoted(K_MANGA_URL_WITH_CHAPTERS));
            csv.push(',');
            csv.push_str(&quoted(K_MANGA_CHAPTER));
            csv.push(',');
            csv.push_str(&quoted(K_MANGA_LAST_UPDATE));
            csv.push(',');
            csv.push_str(&quoted(K_MANGA_NOTES_RAW));
            csv.push(',');
            csv.push_str(&quoted(K_MANGA_MY_ANIME_LIST_LINK));
            csv.push(',');
            csv.push_str(&quoted(K_MANGA_TAGS_SEMICOLON_SEPARATED));

            //quoted(&csv)
            csv
        }
        fn make_csv_fixed_test_string() -> String {
            //const K_MANGA_CSV_FIXED_V1: &str = "\"ゲート―自衛隊彼の地にて、斯く戦えり\",\"https://example.com/manga/gate-chapter-10/\",\"10\",\"2021-07-22T12:34:56\",\"Notes may have commas, (<- this will be replaced) but they will get replaced with \"、\"\",\"#action; #isekai; #fantasy; #shounen\"";
            const K_MANGA_CSV_FIXED_V2: &str = "\"ゲート―自衛隊彼の地にて、斯く戦えり\",\"geeto ― jieitai kano chi nite, kaku tatakae ri\",https://rawkuma.com/gate-jietai-kare-no-chi-nite-kaku-tatakeri-01/,https://rawkuma.com/gate-jietai-kare-no-chi-nite-kaku-tatakeri-01/,\"\",\" 2023-09-06T13:57:22\",\" -\",\"\",\"アニメ化, +1\"";
            let mut csv = String::new();
            csv.push_str(&quoted(K_MANGA_TITLE));
            csv.push(',');
            csv.push_str(&quoted(K_MANGA_URL_WITH_CHAPTERS));
            csv.push(',');
            csv.push_str(&quoted(K_MANGA_CHAPTER));
            csv.push(',');
            csv.push_str(&quoted(K_MANGA_LAST_UPDATE));
            csv.push(',');
            csv.push_str(&quoted(K_MANGA_NOTES_FIXED)); // NOTE: for unit-test validations, we use the fixed version instead of calling fix_comma_in_string() since that invalidates the purpose of "unit" testing
            csv.push(',');
            csv.push_str(&quoted(K_MANGA_TAGS_SEMICOLON_SEPARATED));

            // for V2, addeing romanized_title and url
            csv.push(',');
            csv.push_str(&quoted(K_EXPECTED_ROMANIZED_TITLE));
            csv.push(',');
            csv.push_str(&quoted(K_MANGA_URL));

            //quoted(&csv)
            csv
        }

        // Model based on constants defined:
        fn make_default_model() -> (MangaModel, CsvMangaModel) {
            let mut manga_model = MangaModel::new_from_required_elements(
                K_MANGA_TITLE,
                K_MANGA_URL_WITH_CHAPTERS,
                model_manga::CASTAGNOLI.checksum(K_MANGA_URL_WITH_CHAPTERS.as_bytes()),
            )
            .unwrap(); // for unit-tests, assume that it's always valid
            manga_model.set_url_with_chapter(Some(K_MANGA_URL_WITH_CHAPTERS.to_string()));
            manga_model.set_chapter(Some(K_MANGA_CHAPTER.to_string()));
            manga_model.set_last_update(Some(K_MANGA_LAST_UPDATE.to_string()));
            manga_model.set_notes(Some(K_MANGA_NOTES_FIXED.to_string()));
            manga_model.set_tags(MangaModel::csv_to_tags(
                String::from(K_MANGA_TAGS_SEMICOLON_SEPARATED).as_str(),
            ));
            assert!(manga_model.title_romanized().clone().is_some());
            assert_ne!(
                manga_model.title_romanized().clone().unwrap(), // because regular version mismatches on UTF8 comma ("、"), it's not the same as the expected
                K_EXPECTED_ROMANIZED_TITLE
            );

            let mut csv_manga_model = CsvMangaModel::new(&manga_model.clone());
            assert_eq!(
                csv_manga_model.title_romanized().clone(), // Unlike regular version, CSV verion should have UTF8 comma ("、") intact
                K_EXPECTED_ROMANIZED_TITLE
            );
            (manga_model, csv_manga_model)
        }

        #[test]
        fn test_emedded_quotes_and_invalid_column_counts() {
            // Output: 'this has embedded "quotes" in it'
            let my_csv_string = "
this has embedded \"quotes\" in it,column2
column1,column2 with newline that should get skipped

 column1 starting with space,column2
column1, column2 with space in front
expect 2 errors to follow,one for 3 fields and one for 1 field
c1,because this line has more than 2 columns, the ENTIRE line will get ignored
This line only has one column
#comment line are ignored
# note that the when a column are surrounded by quotes (not in middle, but at ends), it will get trimmed out
\"column1 with quotes\",\"column2 with quotes\"

\"column1 with quotes\",column2 \"without\" quotes at ends

,column2: column1 is empty
column1: column2 is empty,
THE,END
";

            let mut reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .escape(Some(b'\\'))
                .comment(Some(b'#'))
                .from_reader(my_csv_string.as_bytes());

            // WARNINGS:
            // * Usage of 'reader.records()' consumes the reader, so you can only use it once!
            //   Meaning, reader.records().count() will return 4, but the 2nd time you call it, it'll return 0
            //   in which for(result in reader.records()) will not be called at all (since it's already consumed)
            //      // println!("There are {} records", reader.records().count());  // DO NOT DO THIS
            // * First row defines number of expected columns to follow for the REST ofthe rows, and whenever
            //   the reader encounters a row (a row is defined by new-line) with less columns than the first row,
            //   it will return an error BUT it will allow to continue reading the rest of the rows
            for result in reader.records() {
                match result {
                    Ok(record) => {
                        for field in record.iter() {
                            print!("'{}', ", field);
                        }
                        println!("");
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }

        #[test]
        fn test_title() {
            let (manga, csv_manga) = make_default_model();
            assert_eq!(csv_manga.title(), K_MANGA_TITLE);
        }

        #[test]
        fn test_romanized_title() {
            let (manga, csv_manga) = make_default_model();
            assert_eq!(
                csv_manga.clone().title_romanized(),
                K_EXPECTED_ROMANIZED_TITLE
            );
            assert_eq!(
                csv_manga.clone().title_romanized(),
                K_EXPECTED_ROMANIZED_TITLE
            );
        }

        // test serialization to CSV
        #[test]
        fn test_to_csv() {
            let (manga, csv_manga) = make_default_model();
            let csv = csv_manga.to_csv();
            // note that to_csv() appends "\n" at tail
            //assert_eq!(&csv, &(K_MANGA_CSV.to_owned() + "\n"));
            assert_eq!(&csv, &make_csv_fixed_test_string());
        }

        // test deserialization from CSV
        #[test]
        fn test_from_csv() {
            let test_string = make_csv_test_string();
            println!("# K_MANGA_CSV:\n#\t{}", test_string);

            let manga = CsvMangaModel::from_csv(&test_string).unwrap();
            println!("# manga:\n#\t{}\n#", manga);

            assert_eq!(manga.notes(), K_MANGA_NOTES_FIXED);
            assert_eq!(manga.title(), K_MANGA_TITLE);
            assert_eq!(manga.title_romanized(), K_EXPECTED_ROMANIZED_TITLE);
            assert_eq!(manga.url_with_chapter(), K_MANGA_URL_WITH_CHAPTERS);
            assert_eq!(manga.url(), K_MANGA_URL);
            assert_eq!(manga.chapter(), K_MANGA_CHAPTER);
            assert_eq!(manga.last_update(), K_MANGA_LAST_UPDATE);
            assert_eq!(manga.tags(), K_MANGA_TAGS_SEMICOLON_SEPARATED);
        }
    }
}
