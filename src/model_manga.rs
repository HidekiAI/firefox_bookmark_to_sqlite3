use kakasi;
use url::{ParseError, Url};

use crc::{Algorithm, Crc, CRC_32_ISCSI};
pub const CASTAGNOLI: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISCSI);

pub mod model_manga {
    use serde::{Deserialize, Serialize};
    use url::Url;

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
        pub id: u32,                          // primary key - either prune or ignore if id is 0
        pub title: String,                    // UTF8 encoded
        pub title_romanized: Option<String>,  // is Some() ONLY if title was in Japanese
        pub url: String, // home page of manga (see impl of to_url and from_url validation)
        pub url_with_chapter: Option<String>, // last read/updated chapter
        pub chapter: Option<String>, // last read/updated chapter
        pub last_update: Option<String>,
        pub notes: Option<String>,
        pub tags: Vec<String>, // i.e. "#アニメ化" ; empty vec[] is same as None
        pub my_anime_list: Option<String>, // provides author and artist
    }

    // Display trait for pretty printing and to_string()
    impl std::fmt::Display for MangaModel {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                f,
                "id: '{}', title: '{}', title_romanized: '{:?}', url: '{}', url_with_chapter: '{:?}', chapter: '{:?}', last_update: '{:?}', notes: '{:?}', tags: '{:?}', my_anime_list: '{:?}'",
                self.id,
                self.title,
                self.title_romanized,
                self.url,
                self.url_with_chapter,
                self.chapter,
                self.last_update,
                self.notes,
                self.tags,
                self.my_anime_list,
            )
        }
    }

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

    impl MangaModel {
        pub fn romanize_title_self(&mut self) {
            let title_romanized = match kakasi::is_japanese(self.title.as_str()) {
                kakasi::IsJapanese::True => Some(kakasi::convert(self.title.as_str()).romaji),
                _ => None,
            };
            self.title_romanized = title_romanized;
        }
        pub fn romanize_title(title: &str) -> Option<String> {
            match kakasi::is_japanese(title) {
                kakasi::IsJapanese::True => Some(kakasi::convert(title).romaji),
                _ => None,
            }
        }

        pub fn csv_to_tags(csv: &str) -> Vec<String> {
            let mut tags: Vec<String> = Vec::new();
            for tag in csv.split(",") {
                tags.push(tag.to_string());
            }
            tags
        }
        pub fn url_and_chapter(
            url_parsed: Url,
        ) -> (
            String,         /*url_as_is*/
            Option<String>, /*base_url*/
            Option<String>, /*chapter*/
        ) {
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
                            url_parsed.as_str().to_string(), // url_as_is
                            Some(base_url),                  // base_url
                            Some(chapter),                   // chapter
                        )
                    } else {
                        (
                            url_parsed.as_str().to_string(),       // url_as_is
                            Some(url_parsed.as_str().to_string()), // base_url
                            None,                                  // chapter
                        )
                    }
                } else {
                    (
                        url_parsed.as_str().to_string(), // url_as_is
                        None,                            // base_url
                        None,                            // chapter
                    )
                }
            };
            // if base_url does not end with "/", then append it
            let base_url = match base_url.clone() {
                Some(base_url) => match base_url.ends_with("/") {
                    true => Some(base_url),
                    false => Some(base_url + "/"),
                },
                None => None,
            };

            #[cfg(debug_assertions)]
            {
                println!(
                    "## MangaModel::url_and_chapter: {:?}\n#\turl_parsed='{}'\n#\turl_as_is='{}', base_url='{:?}', chapter='{:?}'",
                    url_parsed,
                    url_parsed.as_str(),
                    url_as_is,
                    base_url,
                    chapter,
                );
            }
            (url_as_is, base_url, chapter)
        }

        // disallow empty title, url, or id; note that id passed is commonly/usually from
        // other system such as SQLite, so it is not checked for validity for uniqueness as
        // primary key.
        pub fn new_from_required_elements(
            title_possibly_in_kanji: String,
            url_with_possible_chapter: String,
            id: u32,
        ) -> MangaModel {
            let title_romanized = MangaModel::romanize_title(&title_possibly_in_kanji);

            // validate url
            let url_parsed = match Url::parse(&url_with_possible_chapter) {
                Ok(validated_url) => validated_url,
                Err(e) => panic!("Error parsing url: {}", e),
            };

            let (url_as_is, base_url, chapter) = MangaModel::url_and_chapter(url_parsed.clone());
            #[cfg(debug_assertions)]
            {
                println!("# MangaModel::new_from_required_elements: title='{}', url='{}', id='{}'\n\ttitle_romanized='{:?}', url_with_chapter='{:?}', chapter='{:?}'",
                    title_possibly_in_kanji,
                    url_with_possible_chapter,
                    id,
                    title_romanized,
                    url_with_possible_chapter,
                    chapter,
                );
            }

            MangaModel {
                id: id,
                title: title_possibly_in_kanji,
                title_romanized: title_romanized,
                url: match base_url {
                    Some(base_url) => base_url,
                    None => url_as_is.clone(),
                },
                url_with_chapter: Some(url_as_is.clone()),
                chapter: chapter.clone(),
                last_update: None,
                notes: None,
                tags: Vec::new(), // empty vec[] is same as None
                my_anime_list: None,
            }
        }
    }
}
