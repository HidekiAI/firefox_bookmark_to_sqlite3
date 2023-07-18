//extern crate serde_derive;
// data model (schema) for json serde
pub mod model_json {
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;

    /// Generated by https://quicktype.io

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct BookmarkRootFolder {
        #[serde(rename = "guid")]
        guid: String,

        #[serde(rename = "title")]
        title: String,

        #[serde(rename = "index")]
        index: i64,

        #[serde(rename = "dateAdded")]
        date_added: i64,

        #[serde(rename = "lastModified")]
        last_modified: i64,

        #[serde(rename = "id")]
        id: i64,

        #[serde(rename = "typeCode")]
        type_code: i64,

        #[serde(rename = "type")]
        bookmark_type: Type, // almost most likely it will always be "text/x-moz-place-container" here

        #[serde(rename = "root")]
        root: String,

        #[serde(rename = "children")]
        children: Vec<BookmarkNodes>, // this is the root folder container, so we may have children here
    }

    impl BookmarkRootFolder {
        pub fn children(&self) -> &Vec<BookmarkNodes> {
            &self.children
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct BookmarkNodes {
        #[serde(rename = "guid")]
        guid: String,

        #[serde(rename = "title")]
        title: String,

        #[serde(rename = "index")]
        index: i64,

        #[serde(rename = "dateAdded")]
        date_added: i64,

        #[serde(rename = "lastModified")]
        last_modified: i64,

        #[serde(rename = "id")]
        id: i64,

        #[serde(rename = "typeCode")]
        type_code: i64,

        #[serde(rename = "type")]
        child_type: Type, // we only care about "text/x-moz-place"

        #[serde(rename = "root")]
        root: Option<String>,

        #[serde(rename = "children")]
        children: Option<Vec<BookmarkNodes>>, // if this was "text/x-moz-place-container", we'd have children here...

        #[serde(rename = "uri")]
        uri: Option<String>,
    }

    impl BookmarkNodes {
        pub fn is_bookmark(&self) -> bool {
            self.child_type == Type::TextXMozPlace
        }

        pub fn title(&self) -> &String {
            &self.title
        }

        pub fn uri(&self) -> String {
            // return asn empty string if it is not a bookmark and/or is None
            if self.child_type != Type::TextXMozPlace {
                return String::from("");
            }
            match &self.uri {
                Some(uri) => uri.clone(),
                None => String::from(""),
            }
        }

        pub fn last_modified(&self) -> &i64 {
            &self.last_modified
        }

        pub fn children(&self) -> &Option<Vec<BookmarkNodes>> {
            &self.children
        }
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
    pub enum Type {
        #[serde(rename = "text/x-moz-place")]
        TextXMozPlace,

        #[serde(rename = "text/x-moz-place-container")]
        TextXMozPlaceContainer,

        #[serde(rename = "text/x-moz-place-separator")]
        TextXMozPlaceSeparator,
    }

    #[test]
    fn test() {
        let json_data = r#"
{ "guid": "root________", "title": "", "index": 0, "dateAdded": 1687548918712000, "lastModified": 1689519935422000, "id": 1, "typeCode": 2, "type": "text/x-moz-place-container", "root": "placesRoot", "children": [ { "guid": "menu________", "title": "menu", "index": 0, "dateAdded": 1687548918712000, "lastModified": 1688395173395000, "id": 2, "typeCode": 2, "type": "text/x-moz-place-container", "root": "bookmarksMenuFolder", "children": [ { "guid": "A8NUOjpsRO1f", "title": "", "index": 0, "dateAdded": 1687548920094000, "lastModified": 1687548920094000, "id": 15, "typeCode": 3, "type": "text/x-moz-place-separator" } ] }, { "guid": "toolbar_____", "title": "toolbar", "index": 1, "dateAdded": 1687548918712000, "lastModified": 1689519935422000, "id": 3, "typeCode": 2, "type": "text/x-moz-place-container", "root": "toolbarFolder", "children": [ { "guid": "Npno2qvkXy1F", "title": "Downloads", "index": 0, "dateAdded": 1688676588125000, "lastModified": 1688676595137000, "id": 19, "typeCode": 1, "type": "text/x-moz-place", "uri": "about:downloads" }, { "guid": "EvEy7VW_sMTG", "title": "ゆるキャン△", "index": 1, "dateAdded": 1689519634292000, "lastModified": 1689519634292000, "id": 20, "typeCode": 1, "type": "text/x-moz-place", "uri": "https://some-site/page-of-this-manga" } ] }, { "guid": "unfiled_____", "title": "unfiled", "index": 3, "dateAdded": 1687548918712000, "lastModified": 1687548919979000, "id": 5, "typeCode": 2, "type": "text/x-moz-place-container", "root": "unfiledBookmarksFolder" }, { "guid": "mobile______", "title": "mobile", "index": 4, "dateAdded": 1687548918955000, "lastModified": 1687548919979000, "id": 6, "typeCode": 2, "type": "text/x-moz-place-container", "root": "mobileFolder" } ] }
        "#;
        // deserialize
        let bookmark: BookmarkRootFolder = serde_json::from_str(json_data).unwrap();
        // for test, just recursively traverse down each children and print the title and lastModified and the type
        fn traverse_children(children: &Vec<BookmarkNodes>) {
            for child in children {
                println!(
                    "title: {}, type: {:#?}, lastModified: {}, uri: {:#?}",
                    child.title, child.child_type, child.last_modified, child.uri
                );
                if let Some(children) = &child.children {
                    traverse_children(children);
                }
            }
        }
        traverse_children(&bookmark.children);
    }
}
