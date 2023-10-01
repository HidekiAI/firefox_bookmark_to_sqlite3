pub const CASTAGNOLI: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISCSI);

pub mod model_manga {

    use serde::{Deserialize, Serialize};
    use url::Url;

    use crate::my_utils::{make_none_if_empty, trim_quotes};

    // Create a trait so that we ONLY allow either String or &str generic types

    // Each serde based system (i.e. CSV, JSON, SQLite) will have its own
    // model, specifcally because the deserialization (read) may directly
    // write to data-model bypassing the impl methods.
    // This is the "common" model, which will get passed around to
    // other data-model systems (i.e. SQLite, JSON, CSV, etc.) in order
    // to be specify the required elements for each system to be non-Option types.
    // also to assure valid URL are used, it will use url::Url instead of String.
    // There are other anomalies, such as for CSV, strings with commas will be
    // have issues even if they are quoted, so CSV internally will translate
    // a string with commas to something internally legal, hence there may be
    // cases where we will encounter String that are using UTF8 commas (i.e. "，")
    // instead of ASCII commas (i.e. ",").
    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
    pub struct MangaModel {
        // NOTE: almost (if not) all fields are private, and enforces usage of accessor methods
        // Also, because we do not wish to have it mutate, we probably want it as &str but
        // that would require impl<'a> for MangaModel<'a> and we don't want to do at the moment
        // due to complexities of traits such as clone and display...
        id: u32,                          // primary key - either prune or ignore if id is 0
        title: String,                    // UTF8 encoded
        possible_title_romanized: Option<String>,  // is Some() ONLY if title was in Japanese
        url: String, // home page of manga (see impl of to_url and from_url validation)
        possible_url_with_chapter: Option<String>, // last read/updated chapter
        possible_chapter: Option<String>, // last read/updated chapter
        possible_last_update: Option<String>,
        possible_notes: Option<String>,
        tags: Vec<String>, // i.e. "#アニメ化" ; empty vec[] is same as None
        possible_my_anime_list: Option<String>, // provides author and artist
    }

    // Display trait for pretty printing and to_string()
    impl std::fmt::Display for MangaModel {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                f,
                "id: '{}', title: '{}', title_romanized: '{:?}', url: '{}', url_with_chapter: '{:?}', chapter: '{:?}', last_update: '{:?}', notes: '{:?}', tags: '{:?}', my_anime_list: '{:?}'",
                self.id(),
                self.title(),
                match self.title_romanized().clone() {
                    Some(s) => match s.trim().trim_end_matches('"').is_empty() {
                        false => Some(s),
                        true => None
                    },
                    None => None
                },
                self.url(),
                match self.possible_url_with_chapter.clone() {
                    Some(s) => match s.trim().trim_end_matches('"').is_empty() {
                        false => Some(s),
                        true => None
                    },
                    None => None
                },
                match self.possible_chapter.clone() {
                    Some(s) => match s.trim().trim_end_matches('"').is_empty() {
                        false => Some(s),
                        true => None
                    },
                    None => None
                },
                match self.possible_last_update.clone() {
                    Some(s) => match s.trim().trim_end_matches('"').is_empty() {
                        false => Some(s),
                        true => None
                    },
                    None => None
                },
                match self.possible_notes.clone() {
                    Some(s) => match s.trim().trim_end_matches('"').is_empty() {
                        false => Some(s),
                        true => None
                    },
                    None => None
                },
                self.tags.iter().filter_map(|tag| {
                    match tag.trim().trim_end_matches('"').is_empty() {
                        false => Some(tag.to_owned()),
                        true => None
                    }
                }).collect::<Vec<String>>(),
                match self.possible_my_anime_list.clone() {
                    Some(s) => match s.trim().trim_end_matches('"').is_empty() {
                        false => Some(s),
                        true => None
                    },
                    None => None
                },
            )
        }
    } // Display

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MangaList {
        pub data: Vec<MangaModel>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MangaResponse {
        pub data: MangaModel,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MangaRequest {
        pub title: String,
        pub title_romanized: Option<String>,
        pub url: String,
        pub url_with_chapter: Option<String>,
        pub chapter: Option<String>,
        pub last_update: Option<String>,
        pub notes: Option<String>,
        pub tags: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MangaUpdateRequest {
        pub title: Option<String>,
        pub title_romanized: Option<String>,
        pub url: Option<String>,
        pub url_with_chapter: Option<String>,
        pub chapter: Option<String>,
        pub last_update: Option<String>,
        pub notes: Option<String>,
        pub tags: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MangaDeleteRequest {
        pub id: i32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MangaDeleteResponse {
        pub id: i32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MangaUpdateResponse {
        pub id: i32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MangaCreateResponse {
        pub id: i32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MangaCreateRequest {
        pub title: String,
        pub title_romanized: Option<String>,
        pub url: String,
        pub url_with_chapter: Option<String>,
        pub chapter: Option<String>,
        pub last_update: Option<String>,
        pub notes: Option<String>,
        pub tags: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MangaSearchRequest {
        pub title: Option<String>,
        pub title_romanized: Option<String>,
        pub url: Option<String>,
        pub url_with_chapter: Option<String>,
        pub chapter: Option<String>,
        pub last_update: Option<String>,
        pub notes: Option<String>,
        pub tags: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MangaSearchResponse {
        pub data: Vec<MangaModel>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MangaSearchByTitleRequest {
        pub title: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct MangaSearchByTitleResponse {
        pub data: Vec<MangaModel>,
    }

    // Default implementation of the with_values method that sets validate_url to false
    impl Default for MangaModel {
        fn default() -> Self {
            MangaModel::with_values(
                0,
                String::new(),
                None,
                String::new(),
                None,
                None,
                None,
                None,
                Vec::new(),
                None,
                //false, // Set validate_url to false by default
            )
        }
    }

    impl MangaModel {
        // Allow both String and &str to be passed in with magic of AsRef<T> and s.as_ref() combination
        //fn trim_quotes<T: AsRef<str>>(s: T) -> String {
        //    let mut s = s.as_ref().trim().trim_end_matches('"').to_string();
        //    if s.starts_with('"') || s.ends_with('"') || s.starts_with(' ') || s.ends_with(' ') {
        //        trim_quotes(&s[1..s.len() - 1])
        //    } else {
        //        s.to_string()
        //    }
        //}
        //// turn Some("") into None - NOTE: We do NOT want to return `Option<&'static str>` static lifetime, so we return Option<String>
        //fn make_none<T: AsRef<str>>(s: Option<T>) -> Option<String> {
        //    // no need to transform 's' since trim_quotes() will do it for us
        //    match s {
        //        Some(s) => match trim_quotes(s).is_empty() {
        //            false => Some(trim_quotes(s)),
        //            true => None,
        //        },
        //        None => None,
        //    }
        //}

        pub fn romanize_title_self(&mut self) {
            let title_romanized = match kakasi::is_japanese(self.title.as_str()) {
                kakasi::IsJapanese::True => {
                    Some(trim_quotes(kakasi::convert(self.title.as_str()).romaji))
                }
                _ => None,
            };
            self.possible_title_romanized = title_romanized;
        }

        pub fn romanize_title(title: &str) -> Option<String> {
            match kakasi::is_japanese(title) {
                kakasi::IsJapanese::True => Some(trim_quotes(kakasi::convert(title).romaji)),
                _ => None,
            }
        }

        pub fn csv_to_tags(csv: &str) -> Vec<String> {
            let mut tags: Vec<String> = Vec::new();
            for tag in csv.split(",") {
                tags.push(trim_quotes(tag));
            }
            tags
        }

        pub fn url_and_chapter(
            url_parsed: Url, // because Url does not validate if it is empty string, we can assume to_str() will never be empty string
        ) -> (
            String,         /*url_as_is*/
            Option<String>, /*base_url*/
            Option<String>, /*chapter*/
        ) {
            let url_parsed_as_str = url_parsed.as_str();
            /*
                        UPDATE manga SET
                            last_update = REPLACE(last_update, '"', ''),
                            notes = REPLACE(notes, '"', ''),

                            tags = REPLACE(tags, '"', '')  ;

            .mode csv
            ;

            .output manga.out.csv
            ;

            SELECT
            --  m.id,
                m.title,
                m.title_romanized,
                m.url,
                m.url_with_chapter,
                m.chapter,
                m.last_update,
                m.notes,
                m.my_anime_list,
                (SELECT GROUP_CONCAT(t.tag, ', ')
                    FROM manga_to_tags_map AS mt
                    JOIN tags AS t ON mt.tag_id = t.id
                    WHERE mt.manga_id = m.id) AS tags
            FROM manga AS m;
            ;
                        */
            // i.e. "https://some.example.com/tsuki-ga-michibiku-isekai-douchuu-chapter-12-1/"
            // url_as_is = "https://some.example.com/tsuki-ga-michibiku-isekai-douchuu-chapter-12-1/"
            // base_url = "https://some.example.com/tsuki-ga-michibiku-isekai-douchuu/"
            // chapter = Some("12.1")
            // i.e. "https://some.example.com/tsuki-ga-michibiku-isekai-douchuu/"
            // url_as_is = "https://some.example.com/tsuki-ga-michibiku-isekai-douchuu/"
            // base_url = "https://some.example.com/tsuki-ga-michibiku-isekai-douchuu/"
            // chapter = None
            let (url_as_is, base_url, chapter) = {
                let mut path_segments = url_parsed.path_segments().unwrap().collect::<Vec<_>>();
                // if last segment is empty, then pop it off
                if path_segments.last().unwrap().is_empty() {
                    path_segments.pop();
                }
                if let Some(last_segment) = path_segments.last_mut() {
                    // IF the url contains string "-chapter", then extract chapter number AND base_url
                    if last_segment.contains("-chapter-") {
                        // I have "https://some.example.com/mymanga-chapter-12-1/"
                        // strip off "-chapter-" and replace "-" with "." (if any)
                        // first, locate substring "-chapter-" and split to left and right
                        // i.e. "mymanga-chapter-12-1" => "mymanga" "12-1"
                        // now replace "-" with "." (if any)
                        // i.e. "12-1" => "12.1"
                        // return left as url and right as float:
                        // i.e. (Some("https://some.example.com/mymanga/"), Some(12.1))
                        let mut last_segment_split = last_segment.split("-chapter-"); // iterator of 2 elements
                        let base_url_leftside = last_segment_split.next().unwrap(); // first element is left side
                        let chapter = last_segment_split.next().unwrap().replace("-", "."); // second element is right side

                        path_segments.pop(); // remove last segment
                        path_segments.push(base_url_leftside); // append leftside of "-chapter-" to path_segments
                                                               // format base_url to have the schema, Domain, port if any, and path_segments
                                                               // i.e. "https://some.example.com/mymanga/"
                        let base_url = url_parsed.scheme().to_string()
                            + "://"
                            + url_parsed.domain().unwrap()
                            + url_parsed
                                .port()
                                .map_or("".to_string(), |port| format!(":{}", port))
                                .as_str()
                            + "/"
                            + path_segments.join("/").as_str(); // joins together each segments of the paths with "/" and makes it into a string

                        // if base_url does not end with "/", then append it
                        let base_url = match base_url.ends_with("/") {
                            true => base_url,
                            false => base_url + "/",
                        };

                        // return tuple of url_as_is, base_url, and chapter
                        (
                            trim_quotes(url_parsed_as_str),     // url_as_is
                            make_none_if_empty(Some(base_url)), // base_url
                            make_none_if_empty(Some(chapter)),  // chapter
                        )
                    } else {
                        (
                            trim_quotes(url_parsed_as_str),              // url_as_is
                            make_none_if_empty(Some(url_parsed_as_str)), // base_url
                            None,                                        // chapter
                        )
                    }
                } else {
                    (
                        trim_quotes(url_parsed_as_str), // url_as_is
                        None,                           // base_url
                        None,                           // chapter
                    )
                }
            };
            // if base_url does not end with "/", then append it
            let possible_base_url = match base_url.clone() {
                Some(str_base_url) => match str_base_url.ends_with("/") {
                    true => Some(trim_quotes(str_base_url)),
                    false => Some(trim_quotes(String::from(str_base_url) + "/")),
                },
                None => None,
            };

            #[cfg(debug_assertions)]
            {
                //println!(
                //    "## MangaModel::url_and_chapter: {:?}\n#\turl_parsed='{}'\n#\turl_as_is='{}', base_url='{:?}', chapter='{:?}'",
                //    url_parsed,
                //    url_parsed.as_str(),
                //    url_as_is,
                //    base_url,
                //    chapter,
                //);
            }
            (
                trim_quotes(url_as_is),
                make_none_if_empty(possible_base_url),
                make_none_if_empty(chapter),
            )
        }

        // Private constructor that constructs a MangaModel object with default values
        fn new() -> MangaModel {
            MangaModel {
                id: 0,
                title: String::new(),
                possible_title_romanized: None,
                url: String::new(),
                possible_url_with_chapter: None,
                possible_chapter: None,
                possible_last_update: None,
                possible_notes: None,
                tags: Vec::new(),
                possible_my_anime_list: None,
            }
        }
        // Public constructor that constructs a MangaModel object with the given values
        pub fn with_values(
            id: u32,
            title: String,
            title_romanized: Option<String>,
            url: String,
            url_with_chapter: Option<String>,
            chapter: Option<String>,
            last_update: Option<String>,
            notes: Option<String>,
            tags: Vec<String>,
            my_anime_list: Option<String>,
        ) -> MangaModel {
            // ideally, rather than setting ID=0, use CRC32 of title as ID to make it unique prior to calling this...
            if title.trim().trim_end_matches('"').is_empty() {
                panic!("title is empty");
            }
            if url.trim().trim_end_matches('"').is_empty() {
                panic!("url is empty");
            }
            if id == 0 {
                panic!("id is 0");
            }

            // validate url passed as string is valid url via url::Url, similar to above tests,
            // we'll panic if url is invalid
            let url_parsed = match Url::parse(url.as_str()) {
                Ok(validated_url) => validated_url,
                Err(e) => panic!("Error parsing url ({:?}): {:?}", url, e),
            };

            MangaModel {
                id: id,       // primary key - either prune or ignore if id is 0
                title: title, // UTF8 encoded, uniqueness based on this and/or url
                possible_title_romanized: make_none_if_empty(title_romanized),
                url: trim_quotes(url_parsed), // validated via url::Url
                possible_url_with_chapter: make_none_if_empty(url_with_chapter),
                possible_chapter: make_none_if_empty(chapter),
                possible_last_update: make_none_if_empty(last_update),
                possible_notes: make_none_if_empty(notes),
                tags: tags,
                possible_my_anime_list: make_none_if_empty(my_anime_list),
            }
        }

        // disallow empty title, url, or id; note that id passed is commonly/usually from
        // other system such as SQLite, so it is not checked for validity for uniqueness as
        // primary key.
        pub fn new_from_required_elements(
            title_possibly_in_kanji: &str,
            url_with_possible_chapter: &str,
            id: u32,
        ) -> Result<MangaModel, Box<dyn std::error::Error>> {
            let possible_title_romanized =
                Self::romanize_title(&trim_quotes(title_possibly_in_kanji));

            // validate url
            let url_parsed = match Url::parse(&trim_quotes(url_with_possible_chapter)) {
                Ok(validated_url) => Ok(validated_url),
                Err(e) => Err(format!(
                    "Error:new_from_required_elements({}): {}\n\tRaw: '{:?}'",
                    trim_quotes(url_with_possible_chapter),
                    e,
                    url_with_possible_chapter
                )),
            };

            match url_parsed {
                Ok(parsed) => {
                    // NOTE: url_and_chapter() trims, removes quotes, and makes Some("") to None
                    let (url_as_is, possible_base_url, possible_chapter) =
                        Self::url_and_chapter(parsed);
                    #[cfg(debug_assertions)]
                    {
                        println!("\n# MangaModel::new_from_required_elements: title='{}', url='{}', id='{}'\n\ttitle_romanized='{:?}', url_with_chapter='{:?}', chapter='{:?}'\n",
                                        title_possibly_in_kanji,
                                        url_with_possible_chapter,
                                        id,
                                        possible_title_romanized,
                                        url_with_possible_chapter,
                                        possible_chapter,
                                    );
                    }

                    Ok(Self::with_values(
                        id,
                        trim_quotes(title_possibly_in_kanji),
                        make_none_if_empty(possible_title_romanized),
                        // assume possible_base_url is trimmed, quotes removed, and Some("") is None
                        match possible_base_url {
                            Some(base_url) => base_url.to_string(),
                            None => url_as_is.clone(),
                        },
                        make_none_if_empty(Some(url_as_is.clone())),
                        possible_chapter.clone(), // assume possible_chapter is trimmed, quotes removed, and Some("") is None
                        None,
                        None,
                        Vec::new(), // empty vec[] is same as None
                        None,
                    ))
                }
                Err(str_err) => Err(str_err.into()),
            }
        }

        pub fn id(&self) -> u32 {
            self.id
        }
        pub fn title(&self) -> &str {
            self.title.as_str()
        }
        pub fn title_romanized(&self) -> Option<String> {
            // Note that for Option<String> based, it's not possible to return Option<String> since
            // the lifetime of the Option<String> is not the same as the lifetime of the Option<String>
            // so we return Option<String> instead.
            make_none_if_empty(self.possible_title_romanized.as_ref())
        }
        pub fn url(&self) -> &str {
            self.url.as_str()
        }
        pub fn url_with_chapter(&self) -> Option<String> {
            make_none_if_empty(self.possible_url_with_chapter.as_ref())
        }
        pub fn chapter(&self) -> Option<String> {
            make_none_if_empty(self.possible_chapter.as_ref())
        }
        pub fn last_update(&self) -> Option<String> {
            make_none_if_empty(self.possible_last_update.as_ref())
        }
        pub fn notes(&self) -> Option<String> {
            make_none_if_empty(self.possible_notes.as_ref())
        }

        pub fn tags(&self) -> Vec<&str> {
            self.tags
                .iter()
                .filter_map(|tag| match tag.trim().trim_end_matches('"').is_empty() {
                    false => Some(tag.as_str()),
                    true => None,
                })
                .collect::<Vec<&str>>()
        }
        pub fn my_anime_list(&self) -> Option<String> {
            make_none_if_empty(self.possible_my_anime_list.as_ref())
        }

        pub fn set_id(&mut self, id: u32) {
            self.id = id;
        }
        pub fn set_title(&mut self, title: String) {
            self.title = title;
        }
        pub fn set_title_romanized(&mut self, title_romanized: Option<String>) {
            self.possible_title_romanized = title_romanized.map(|s| trim_quotes(s));
        }
        pub fn set_url(&mut self, url: String) {
            self.url = url;
        }
        pub fn set_url_with_chapter(&mut self, url_with_chapter: Option<String>) {
            self.possible_url_with_chapter = url_with_chapter.map(|s| trim_quotes(s));
        }
        pub fn set_chapter(&mut self, chapter: Option<String>) {
            self.possible_chapter = chapter.map(|s| trim_quotes(s));
        }
        pub fn set_last_update(&mut self, last_update: Option<String>) {
            self.possible_last_update = last_update.map(|s| trim_quotes(s));
        }
        pub fn set_notes(&mut self, notes: Option<String>) {
            self.possible_notes = notes.map(|s| trim_quotes(s));
        }
        pub fn set_tags(&mut self, tags: Vec<String>) {
            self.tags = tags;
        }
        pub fn set_my_anime_list(&mut self, my_anime_list: Option<String>) {
            self.possible_my_anime_list = my_anime_list.map(|s| trim_quotes(s));
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_manga_model() {
            let mut manga = MangaModel::with_values(
                1,
                "My Manga".to_owned(),
                Some(String::from("My Romanized Manga")),
                "https://example.com/manga".to_owned(),
                Some("https://example.com/manga/1".to_string()),
                Some("1".to_string()),
                Some("2021-01-01".to_string()),
                Some("My notes".to_string()),
                vec!["tag1".to_owned(), "tag2".to_owned()],
                Some("https://myanimelist.net/manga/1".to_string()),
            );

            assert_eq!(manga.id(), 1);
            assert_eq!(manga.title(), "My Manga");
            assert_eq!(
                manga.title_romanized(),
                Some("My Romanized Manga".to_string())
            );
            assert_eq!(manga.url(), "https://example.com/manga");
            assert_eq!(
                manga.url_with_chapter(),
                Some("https://example.com/manga/1".to_string())
            );
            assert_eq!(manga.chapter(), Some("1".to_string()));
            assert_eq!(manga.last_update(), Some("2021-01-01".to_string()));
            assert_eq!(manga.notes(), Some("My notes".to_string()));
            assert_eq!(manga.tags(), vec!["tag1".to_owned(), "tag2".to_owned()]);
            assert_eq!(
                manga.my_anime_list(),
                Some("https://myanimelist.net/manga/1".to_string())
            );

            // update notes
            manga.set_notes(Some("My updated notes".to_string()));
            assert!(manga.notes().is_some());
            assert_eq!(manga.notes(), Some("My updated notes".to_string()));

            // update tags
            manga.set_tags(vec!["tag3".to_owned(), "tag4".to_owned()]);
            assert!(manga.tags().is_empty() == false);
            assert_eq!(manga.tags(), vec!["tag3".to_owned(), "tag4".to_owned()]);
        }
    }
}
