//use kakasi;

// manga csv format
// "title","url", "url_with_chapters", "chapter","last_modified","notes", "tags"
pub mod model_csv_manga {
    use csv::Writer;
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;
    use std::io::Write;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Manga {
        #[serde(rename = "Title")]
        title: String,

        #[serde(rename = "Romanized Title")]
        romanized_title: String,

        #[serde(rename = "URL with Chapters")]
        url_with_chapters: String,

        #[serde(rename = "Chapter")]
        chapter: String,

        #[serde(rename = "Last Modified")]
        last_modified: String,

        #[serde(rename = "Notes")]
        notes: String,

        #[serde(rename = "Tags")]
        tags: String,
    }

    //impl Default for Manga {
    //    fn default() -> Self {
    //        Manga {
    //            title: String::from(""),
    //            romanized_title: String::from(""),
    //            url_with_chapters: String::from(""),
    //            chapter: String::from(""),
    //            last_modified: String::from(""),
    //            notes: String::from(""),
    //            tags: String::from(""),
    //        }
    //    }
    //}

    impl Manga {
        pub fn new(title: String) -> Self {
            let romanized_title = Manga::romanized_url_with_chapters(&title);
            Manga {
                title,
                romanized_title,
                url_with_chapters: String::from(""),
                chapter: String::from(""),
                last_modified: String::from(""),
                notes: String::from(""),
                tags: String::from(""),
            }
        } // new(title: String)

        pub fn new_with_url(title: String, url_with_chapters: String) -> Self {
            let romanized_title = Manga::romanized_url_with_chapters(&title);
            Manga {
                title,
                romanized_title,
                url_with_chapters,
                chapter: String::from(""),
                last_modified: String::from(""),
                notes: String::from(""),
                tags: String::from(""),
            }
        } // new_with_url(title: String, url_with_chapters: String)

        pub fn to_csv(&self) -> String {
            let mut wtr = csv::WriterBuilder::new()
                .quote_style(csv::QuoteStyle::Always) // quote everything
                .has_headers(false)
                .from_writer(vec![]);
            wtr.serialize((
                &self.title,
                &self.romanized_title,
                &self.url_with_chapters,
                &self.chapter,
                &self.last_modified,
                &self.notes,
                &self.tags,
            ))
            .unwrap();
            String::from_utf8(wtr.into_inner().unwrap()).unwrap()
        }

        pub fn from_csv(csv: &str) -> Result<Manga, csv::Error> {
            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(csv.as_bytes());

            let record: Manga = rdr.deserialize().next().unwrap()?;
            Ok(Manga {
                title: record.title,
                romanized_title: record.romanized_title,
                url_with_chapters: record.url_with_chapters,
                chapter: record.chapter,
                last_modified: record.last_modified,
                notes: record.notes,
                tags: record.tags,
            })
        }

        pub fn title(&self) -> &String {
            &self.title
        }
        pub fn romanized_title(&self) -> &String {
            &self.romanized_title
        }
        pub fn url_with_chapters(&self) -> &String {
            &self.url_with_chapters
        }
        pub fn chapter(&self) -> &String {
            &self.chapter
        }
        pub fn last_modified(&self) -> &String {
            &self.last_modified
        }
        pub fn notes(&self) -> &String {
            &self.notes
        }
        pub fn tags(&self) -> &String {
            &self.tags
        }
        // only exposing this function so that use-depencies of kakasi will be limited to this module only
        // but mainly, also want to preserve at least the UTF8 comma ("、") in the title
        pub fn romanized_url_with_chapters(title: &String) -> String {
            kakasi::convert(title).romaji.replace(",", "、")
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

        pub fn record(
            &mut self,
            bookmark_last_modified_epoch_micros: i64,
            bookmark_uri: &String,
            bookmark_title: &String,
        ) -> Result<(), csv::Error> {
            // convert the last_modified i64 to datetime - last_modified is encoded as unix epoch time in microseconds
            let last_modified = chrono::NaiveDateTime::from_timestamp_opt(
                bookmark_last_modified_epoch_micros / 1_000_000,
                (bookmark_last_modified_epoch_micros % 1_000_000) as u32,
            )
            .unwrap();
            // output: "uri_stripped_for_sorting","title","uri","chapter","last_modified","notes","tags"
            // extract chapter if link indicates so...
            let target_string = "chapter";
            let chapter = match bookmark_uri
                .to_lowercase()
                .contains(&target_string.to_lowercase())
            {
                false => "0".to_string(),
                true => {
                    // get substring past the string "chapter" from the URI
                    // i.e. "http://mydomain.tld/title-chapter-10", "http://mydomain.tld/title-chapter-10-1", "http://mydomain.tld/title-chapter-10/", "http://mydomain.tld/title-chapter-10-1/"
                    // we want to extract as "10", "10.1" - note that trailing "/" needs to be removed and "-" needs to be replaced with "."
                    let mut chapter_number = bookmark_uri.to_lowercase();
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
            let mut uri_stripped = bookmark_uri.clone();
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

            let mut record = csv::StringRecord::new();
            record.push_field(&uri_stripped);
            record.push_field(&bookmark_title);
            record.push_field(&bookmark_uri);
            record.push_field(&chapter);
            record.push_field(&last_modified.format("%Y-%m-%dT%H:%M:%S").to_string());
            record.push_field("-");
            record.push_field("#");
            self.csv_writer.write_record(&record)
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::model_csv_manga::model_csv_manga::Manga;

        #[test]
        fn test_title() {
            let manga = Manga {
                title: String::from("One Piece"),
                romanized_title: String::from("Wan Pīsu"),
                url_with_chapters: String::from("https://example.com/manga/one-piece/"),
                chapter: String::from("1000"),
                last_modified: String::from("2021-07-22T12:34:56"),
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
            let manga = Manga {
                title: k_manga_title.clone(),
                romanized_title: Manga::romanized_url_with_chapters(&k_manga_title.clone()),
                url_with_chapters: String::from("https://example.com/manga/gate/"),
                chapter: String::from("1000"),
                last_modified: String::from("2021-07-22T12:34:56"),
                notes: String::from(""),
                tags: String::from("action; adventure; comedy; drama; fantasy; shounen"), // NOTE: cannot have commmas inside strings for MOST CSV utilities fails to know the differences...
            };
            assert_eq!(manga.romanized_title(), &k_expected_romanized_title);
        }

        // test serialization to CSV
        #[test]
        fn test_to_csv() {
            let manga = Manga {
                title: String::from("One Piece"),
                romanized_title: Manga::romanized_url_with_chapters(&String::from("One Piece")),
                url_with_chapters: String::from("https://example.com/manga/one-piece/"),
                chapter: String::from("1000"),
                last_modified: String::from("2021-07-22T12:34:56"),
                notes: String::from(""),
                tags: String::from("action; adventure; comedy; drama; fantasy; shounen"), // NOTE: cannot have commmas inside strings for MOST CSV utilities fails to know the differences...
            };
            let csv = manga.to_csv();
            assert_eq!(
                csv,
                "\"One Piece\",\"One Piece\",\"https://example.com/manga/one-piece/\",\"1000\",\"2021-07-22T12:34:56\",\"\",\"action; adventure; comedy; drama; fantasy; shounen\"\n"
            );
        }

        // test deserialization from CSV
        #[test]
        fn test_from_csv() {
            let csv = "\"One Piece\",\"One Piece\",\"https://example.com/manga/one-piece/\",\"1000\",\"2021-07-22T12:34:56\",\"\",\"action; adventure; comedy; drama; fantasy; shounen\"\n";
            let manga = Manga::from_csv(csv).unwrap();
            assert_eq!(manga.title(), "One Piece");
            assert_eq!(
                manga.romanized_title(),
                &Manga::romanized_url_with_chapters(&String::from("One Piece"))
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
