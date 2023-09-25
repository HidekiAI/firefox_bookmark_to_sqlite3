use crate::model_manga;
use rusqlite;

// SQLite3 Manga data model
// TABLE manga_no_id:
// id, title, title_romanized, url, chapter, url_with_chapter, last_update, notes, tags
mod model_sqlite3_manga {
    use rusqlite::{Connection, Result, Row};
    use std::{path::Path, error::Error};

    use crate::model_manga::model_manga::MangaModel;


    // NOTE: Unlike CSV and JSON, because SQLite3 is not meant as serde, we do not need
    // to define data-model (schema) for SQLite3, and we'll directly use the model_manga_no_id::MangaModel
    // struct as the data model (schema)
    // Only thing that may differ (for now)

    // create 3 tables, manga, tag_map, and tags (tags field in manga table is a foreign key to tag_map table, and tag_map table is a foreign key to tags)
    // manga table:
    // crucial that the order defined here NEVER changes
    // because almost all queries are based on the sequentially ordered column-index
    // i.e. 'row.get(5)?' is chapter
    // 0: id (PRIMARY KEY)
    // 1: title (NOT NULL)
    // 2: title_romanized
    // 3: url (NOT NULL)
    // 4: url_with_chapter
    // 5: chapter
    // 6: last_update
    // 7: notes
    // 8: tags - foreign key to tag_group_maps table
    // 9: my_anime_list
    // append new columns to the end of the list, never between
    // Schemas:
    // CREATE TABLE manga (
    //     id INTEGER PRIMARY KEY AUTOINCREMENT,
    //     title TEXT NOT NULL,
    //     ...
    // );
    //
    // CREATE TABLE tag (
    //     id INTEGER PRIMARY KEY AUTOINCREMENT,
    //     description TEXT
    // );
    //
    // CREATE TABLE tag_groups (    -- uniqueness of this table is based on the pair maind_id and tag_id, so no need for primary key
    //     main_id INTEGER,
    //     tag_id INTEGER,
    //     FOREIGN KEY(main_id) REFERENCES main(id),
    //     FOREIGN KEY(tag_id) REFERENCES tag(id)
    // );

    fn create_manga_table(db_full_paths: &str) -> Result<()> {
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS manga (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                title_romanized TEXT,
                url TEXT NOT NULL,
                url_with_chapter TEXT,
                chapter TEXT,
                last_update TEXT,
                notes TEXT,
                tags TEXT,  -- just preserve the tags that may have come from original
                my_anime_list TEXT
            )",
            [],
        )?;

        Ok(())
    }

    // table wich has foreign key to manga table and tags table, and is the intermediary table
    fn create_manga_to_tags_map_table(db_full_paths: &str) -> Result<()> {
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS manga_to_tags_map (
                manga_id INTEGER,
                tag_id INTEGER,
                FOREIGN KEY(manga_id) REFERENCES manga(id),
                FOREIGN KEY(tag_id) REFERENCES tags(id)
            )",
            [],
        )?;

        Ok(())
    }

    fn create_tags_table(db_full_paths: &str) -> Result<()> {
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tag TEXT NOT NULL UNIQUE
            )",
            [],
        )?;

        Ok(())
    }

    pub fn create_tables(db_full_paths: &str) -> Result<()> {
        create_manga_table(db_full_paths)?;
        create_manga_to_tags_map_table(db_full_paths)?;
        create_tags_table(db_full_paths)?;

        Ok(())
    }

    // Insert MangaModel (without id field, id=0) and associate tags if any, and return new MangaModel with real/valid id
    pub fn insert_manga(db_full_paths: &str, manga_no_id: &MangaModel) -> Result<MangaModel> {
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        // Option based vars needs to become concrete before we can use them in query
        // first, insert MangaModel so that we can get the id
        conn.execute(
            "INSERT INTO manga (title, title_romanized, url, url_with_chapter, chapter, last_update, notes, tags, my_anime_list) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            &[
                &manga_no_id.title, 
                match &manga_no_id.title_romanized { Some(t) => t, None => ""   }, 
                &manga_no_id.url ,
                match &manga_no_id.url_with_chapter { Some(t) => t, None => ""   },
                match &manga_no_id.chapter { Some(t) => t, None => ""   },
                match &manga_no_id.last_update { Some(t) => t, None => ""   },
                match &manga_no_id.notes { Some(t) => t, None => ""   },
                &manga_no_id.tags.join(","), 
                match &manga_no_id.my_anime_list { Some(t) => t, None => ""   },
                ],
        )?;

        // update MangaModel with the id
        let id = conn.last_insert_rowid() as u32;
        let mut manga = manga_no_id.clone();
        manga.id = id;

        // second, insert tags if any
        if manga.tags.len() > 0 {
            for tag in manga.tags.clone() {
                // insert tag if not exists (case insensitive)
                conn.execute("INSERT OR IGNORE INTO tags (tag) VALUES (?1)", &[&tag])?;
                // get tag id
                let mut stmt = conn.prepare("SELECT id FROM tags WHERE tag = ?1")?;
                let mut tag_iter = stmt.query_map(&[&tag], |row| Ok(row.get(0)?))?;

                let tag_id = tag_iter.next().unwrap().unwrap();
                // insert tag id and manga id into manga_to_tags_map table if the pair does not yet exists (shouldn't exists, but just in case)
                conn.execute(
                    "INSERT OR IGNORE INTO manga_to_tags_map (manga_id, tag_id) VALUES (?1, ?2)",
                    &[&id, &tag_id],
                )?;
            }
        }

        Ok(manga.clone())
    }

    // update based on id field
    pub fn update_manga(db_full_paths: &str, manga: &MangaModel) -> Result<(), Box<dyn std::error::Error + 'static>> {
        // fail if id (u32) is 0
        if manga.id == 0 {
            return Err("id cannot be 0".into());
        }

        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        // if we cannot locate id in manga table during update, return error (most likely got deleted)
        // query for id and title (just in case we need to return the title)
        let mut stmt = conn.prepare("SELECT id FROM manga WHERE id = ?1")?; // returns either 0 or 1 row
        let manga_iter = stmt.query_map(
            &[&manga.id], 
            |row| { 
                row.get::<usize, i32>(0)    // id is i32 type...
            }
        )?;
        if manga_iter.count() == 0 {
            return Err("id not found".into());
        }

        // OK, id exists, so proceed with update
        conn.execute(
            "UPDATE manga SET title = ?1, title_romanized = ?2, url = ?3, url_with_chapter = ?4, chapter = ?5, last_update = ?6, notes = ?7, tags = ?8, my_anime_list = ?9 WHERE id = ?10",
            &[
                &manga.title, 
                match &manga.title_romanized { Some(t) => t, None => ""   }, 
                &manga.url ,
                match &manga.url_with_chapter { Some(t) => t, None => ""   },
                match &manga.chapter { Some(t) => t, None => ""   },
                match &manga.last_update { Some(t) => t, None => ""   },
                match &manga.notes { Some(t) => t, None => ""   },
                &manga.tags.join(","), 
                match &manga.my_anime_list { Some(t) => t, None => ""   },
                &manga.id.to_string(),
                ],
        )?;

        Ok(())
    }

    // delete the row based on id field
    pub fn delete_manga(db_full_paths: &str, id: u32) -> Result<bool> {
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        // if we cannot locate id in manga table during delete, just return Ok(false) (most likely got deleted)
        // should only return single row since we're using id as primary key
        // haven't had time to investigate, but if I just return single column (SELECT id) I'd get an error, so I'm returning 2 columns (id and title) and just ignore the title
        let mut stmt = conn.prepare("SELECT id FROM manga WHERE id = ?1")?; // returns either 0 or 1 row
        let manga_iter = stmt.query_map(
            &[&id], 
            |row| { 
                row.get::<usize, i32>(0)    // id is i32 type...
            }
        )?;
        if manga_iter.count() == 0 {
            return Ok(false);   // just bail out with a warning...
        }

        // if here, id existed, so proceed with delete
        conn.execute("DELETE FROM manga WHERE id = ?1", &[&id])?;

        // also prune tags group table (again, if cannot find, it's OK)
        conn.execute("DELETE FROM manga_to_tags_map WHERE manga_id = ?1", &[&id])?;

        Ok((true))
    }

    // return in manga struct based on ID
    pub fn select_manga(db_full_paths: &str, id: u32) -> Result<MangaModel> {
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        // SELECT from manga_to_tags table for tags field based off of manga.id
        // and then JOIN with manga table
        // based on ONLY rows that matches manga.id == id passed in
        // SELECT manga.id, manga.title, manga.title_romanized, manga.url, manga.url_with_chapter, manga.chapter, manga.last_update, 
        //      manga.notes, manga.my_anime_list, 
        //      GROUP_CONCAT(tags.tag, ',') AS tags 
        //          FROM manga_to_tags_map 
        //          INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id 
        //          INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id 
        //          WHERE manga.id = 1 
        //          GROUP BY manga.id
        //      WHERE manga.id = ?1
        let mut stmt = conn.prepare("SELECT manga.id, manga.title, manga.title_romanized, manga.url, manga.url_with_chapter, manga.chapter, manga.last_update, manga.notes, GROUP_CONCAT(tags.tag, ',') AS tags, manga.my_anime_list FROM manga_to_tags_map INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id WHERE manga.id = ?1 GROUP BY manga.id")?;

        let manga_iter = stmt.query_map(&[&id], |row| {
            Ok(MangaModel {
                id: row.get(0)?,
                title: row.get(1)?,
                title_romanized: row.get(2)?,
                url: row.get(3)?,
                url_with_chapter: row.get(4)?,
                chapter: row.get(5)?,
                last_update: row.get(6)?,
                notes: row.get(7)?,
                tags: match row.get::<usize, String>(8) {
                    Ok(t) => t.split(",").map(|s| s.to_string()).collect(), // alternatively, we can query manga_to_tags_map table directly...
                    Err(_) => Vec::new(),
                },
                my_anime_list: row.get(9)?,
            })
        })?;

        for manga in manga_iter {
            return Ok(manga.unwrap());
        }

        Err(rusqlite::Error::QueryReturnedNoRows)
    }

    // return in manga struct array
    pub fn select_all_manga(db_full_paths: &str) -> Result<Vec<MangaModel>> {
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;
        // SELECT * from manga table and JOIN with manga_to_tags_map table matching manga.id == manga_to_tags_map.manga_id
        // SELECT manga.id, manga.title, manga.title_romanized, manga.url, \
        //          manga.url_with_chapter, manga.chapter, manga.last_update,   \
        //          manga.notes, manga.my_anime_list,                           \
        //          GROUP_CONCAT(tags.tag, ',') AS tags                        \
        //      FROM manga_to_tags_map                                         \
        //      INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id       \
        //      INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id          \
        //      GROUP BY manga.id
        let mut stmt = conn.prepare(
            "SELECT manga.id, manga.title, manga.title_romanized,               \
                        manga.url, manga.url_with_chapter, manga.chapter,           \
                        manga.last_update, manga.notes,                             \
                        GROUP_CONCAT(tags.tag, ',') AS tags,                        \
                            manga.my_anime_list FROM manga_to_tags_map              \
                            INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id   \
                            INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id       \
                            GROUP BY manga.id")?;

        let manga_iter = stmt.query_map([], |row| {
            Ok(MangaModel {
                id: row.get(0)?,
                title: row.get(1)?,
                title_romanized: row.get(2)?,
                url: row.get(3)?,
                url_with_chapter: row.get(4)?,
                chapter: row.get(5)?,
                last_update: row.get(6)?,
                notes: row.get(7)?,
                tags: match row.get::<usize, String>(8) {
                    Ok(t) => t.split(",").map(|s| s.to_string()).collect(),
                    Err(_) => Vec::new(),
                },
                my_anime_list: row.get(9)?,
            })
        })?;

        let mut manga_vec = Vec::new();
        for manga in manga_iter {
            manga_vec.push(manga.unwrap());
        }

        Ok(manga_vec)
    }

    pub fn select_all_manga_by_title(db_full_paths: &str, title: &str) -> Result<Vec<MangaModel>> {
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        // SELECT * from manga table and JOIN with manga_to_tags_map table matching manga.id == manga_to_tags_map.manga_id
        // and manga.title is LIKE %title% that is passed in
        // make sure title can match based on case-insensitive and partial match (i.e. if title passed in is "manga", it can match titles "manga 1", "the other manga 2", etc.)
        // SELECT manga.id, manga.title, manga.title_romanized, manga.url,    \
        //        manga.url_with_chapter, manga.chapter, manga.last_update,   \
        //        manga.notes, manga.my_anime_list,                           \
        //        GROUP_CONCAT(tags.tag, ',') AS tags                         \
        //          FROM manga_to_tags_map                                    \
        //          INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id \
        //          INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id     \
        //      WHERE manga.title LIKE '% || ?1 || %'
        let mut stmt = conn.prepare(
            "SELECT manga.id, manga.title, manga.title_romanized, manga.url,        \
                         manga.url_with_chapter, manga.chapter, manga.last_update,      \
                         manga.notes,                                                   \
                         GROUP_CONCAT(tags.tag, ',') AS tags,                           \
                            manga.my_anime_list FROM manga_to_tags_map                  \
                            INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id   \
                            INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id       \
                    WHERE manga.title LIKE '%' || ?1 || '%' ")?;
        let manga_iter = stmt.query_map(&[&title], |row| {
            Ok(MangaModel {
                id: row.get(0)?,
                title: row.get(1)?,
                title_romanized: row.get(2)?,
                url: row.get(3)?,
                url_with_chapter: row.get(4)?,
                chapter: row.get(5)?,
                last_update: row.get(6)?,
                notes: row.get(7)?,
                tags: match row.get::<usize, String>(8) {
                    Ok(t) => t.split(",").map(|s| s.to_string()).collect(),
                    Err(_) => Vec::new(),
                },
                my_anime_list: row.get(9)?,
            })
        })?;

        let mut manga_vec = Vec::new();
        for manga in manga_iter {
            manga_vec.push(manga.unwrap());
        }

        Ok(manga_vec)
    }

    // locate/search manga where romanized title are similar
    // search string can start with, end with, or contain romanized title in any part of the string
    // sample search string: "a", "a%", "%a", "%a%"
    pub fn select_all_manga_by_title_romanized(
        db_full_paths: &str,
        title_romanized: &str,
    ) -> Result<Vec<MangaModel>> {
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        let title_romanized_like = format!("%{}%", title_romanized);

        let mut stmt = conn.prepare(
            "SELECT manga.id, manga.title, manga.title_romanized, manga.url,        \
                         manga.url_with_chapter, manga.chapter, manga.last_update,      \
                         manga.notes,                                                   \
                         GROUP_CONCAT(tags.tag, ',') AS tags,                           \
                            manga.my_anime_list FROM manga_to_tags_map                  \
                            INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id   \
                            INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id       \
                    WHERE manga.title_romanized LIKE '%' || ?1 || '%' ")?;
        let manga_iter = stmt.query_map(&[&title_romanized_like], |row| {
            Ok(MangaModel {
                id: row.get(0)?,
                title: row.get(1)?,
                title_romanized: row.get(2)?,
                url: row.get(3)?,
                url_with_chapter: row.get(4)?,
                chapter: row.get(5)?,
                last_update: row.get(6)?,
                notes: row.get(7)?,
                tags: match row.get::<usize, String>(8) {
                    Ok(t) => t.split(",").map(|s| s.to_string()).collect(),
                    Err(_) => Vec::new(),
                },
                my_anime_list: row.get(9)?,
            })
        })?;

        let mut manga_vec = Vec::new();
        for manga in manga_iter {
            manga_vec.push(manga.unwrap());
        }

        Ok(manga_vec)
    }

    pub fn select_all_manga_by_url(db_full_paths: &str, url: &str) -> Result<Vec<MangaModel>> {
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        let mut stmt = conn.prepare(
            "SELECT manga.id, manga.title, manga.title_romanized, manga.url,        \
                         manga.url_with_chapter, manga.chapter, manga.last_update,      \
                         manga.notes,                                                   \
                         GROUP_CONCAT(tags.tag, ',') AS tags,                           \
                            manga.my_anime_list FROM manga_to_tags_map                  \
                            INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id   \
                            INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id       \
                    WHERE manga.url LIKE '%' || ?1 || '%' ")?;
        let manga_iter = stmt.query_map(&[&url], |row| {
            Ok(MangaModel {
                id: row.get(0)?,
                title: row.get(1)?,
                title_romanized: row.get(2)?,
                url: row.get(3)?,
                url_with_chapter: row.get(4)?,
                chapter: row.get(5)?,
                last_update: row.get(6)?,
                notes: row.get(7)?,
                tags: match row.get::<usize, String>(8) {
                    Ok(t) => t.split(",").map(|s| s.to_string()).collect(),
                    Err(_) => Vec::new(),
                },
                my_anime_list: row.get(9)?,
            })
        })?;

        let mut manga_vec = Vec::new();
        for manga in manga_iter {
            manga_vec.push(manga.unwrap());
        }

        Ok(manga_vec)
    }

    pub fn select_all_manga_by_url_with_chapter(
        db_full_paths: &str,
        url_with_chapter: &str,
    ) -> Result<Vec<MangaModel>> {
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        let mut stmt = conn.prepare(
            "SELECT manga.id, manga.title, manga.title_romanized, manga.url,        \
                         manga.url_with_chapter, manga.chapter, manga.last_update,      \
                         manga.notes,                                                   \
                         GROUP_CONCAT(tags.tag, ',') AS tags,                           \
                            manga.my_anime_list FROM manga_to_tags_map                  \
                            INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id   \
                            INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id       \
                    WHERE manga.url_with_chapter LIKE '%' || ?1 || '%' ")?;
        let manga_iter = stmt.query_map(&[&url_with_chapter], |row| {
            Ok(MangaModel {
                id: row.get(0)?,
                title: row.get(1)?,
                title_romanized: row.get(2)?,
                url: row.get(3)?,
                url_with_chapter: row.get(4)?,
                chapter: row.get(5)?,
                last_update: row.get(6)?,
                notes: row.get(7)?,
                tags: match row.get::<usize, String>(8) {
                    Ok(t) => t.split(",").map(|s| s.to_string()).collect(),
                    Err(_) => Vec::new(),
                },
                my_anime_list: row.get(9)?,
            })
        })?;

        let mut manga_vec = Vec::new();
        for manga in manga_iter {
            manga_vec.push(manga.unwrap());
        }

        Ok(manga_vec)
    }


    // locate in groups, all the rows that matches the same base url (without chapter) and return
    // rows (in groups based on base url) that has more than one rows (duplicates) per group
    // mainly for the purposes of purging duplicates
    // (see select_all_manga_by_romanized_title)
    pub fn select_all_manga_by_base_url(db_full_paths: &str) -> Result<Vec<MangaModel>> {
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        // SELECT manga.id, manga.title, manga.title_romanized, manga.url, manga.url_with_chapter, manga.chapter, manga.last_update, 
        //      manga.notes, manga.my_anime_list, 
        //      GROUP_CONCAT(tags.tag, ',') AS tags 
        //          FROM manga_to_tags_map 
        //          INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id 
        //          INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id 
        //          WHERE manga.id = 1 
        //          GROUP BY manga.id
        let mut stmt = conn.prepare("SELECT manga.id, manga.title, manga.title_romanized, manga.url, manga.url_with_chapter, manga.chapter, manga.last_update, manga.notes, GROUP_CONCAT(tags.tag, ',') AS tags, manga.my_anime_list FROM manga_to_tags_map INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id WHERE manga.id = ?1 GROUP BY manga.id")?;
        let mut stmt2 = conn.prepare("SELECT id, title, title_romanized, url, url_with_chapter, chapter, last_update, \
                                                            notes, tags, my_anime_list FROM manga GROUP BY url HAVING COUNT(*) > 1")?;
        let manga_iter = stmt.query_map([], |row| {
            Ok(MangaModel {
                id: row.get(0)?,
                title: row.get(1)?,
                title_romanized: row.get(2)?,
                url: row.get(3)?,
                url_with_chapter: row.get(4)?,
                chapter: row.get(5)?,
                last_update: row.get(6)?,
                notes: row.get(7)?,
                tags: match row.get::<usize, String>(8) {
                    Ok(t) => t.split(",").map(|s| s.to_string()).collect(),
                    Err(_) => Vec::new(),
                },
                my_anime_list: row.get(9)?,
            })
        })?;

        let mut manga_vec = Vec::new();
        for manga in manga_iter {
            manga_vec.push(manga.unwrap());
        }

        Ok(manga_vec)
    }

    // locate in groups, all the rows that matches the same romanized title (but differs in url) and return
    // rows (in groups based on romanized title) that has more than one rows (duplicates) per group
    // mainly for the purposes of purging duplicates
    // (see select_all_manga_by_base_url)
    pub fn select_all_manga_by_romanized_title(db_full_paths: &str) -> Result<Vec<MangaModel>> {
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        // SELECT manga.id, manga.title, manga.title_romanized, manga.url, manga.url_with_chapter, manga.chapter, manga.last_update, 
        //      manga.notes, manga.my_anime_list, 
        //      GROUP_CONCAT(tags.tag, ',') AS tags 
        //          FROM manga_to_tags_map 
        //          INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id 
        //          INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id 
        //          WHERE manga.id = 1 
        //          GROUP BY manga.id
        let mut stmt = conn.prepare("SELECT manga.id, manga.title, manga.title_romanized, manga.url, manga.url_with_chapter, manga.chapter, manga.last_update, manga.notes, GROUP_CONCAT(tags.tag, ',') AS tags, manga.my_anime_list FROM manga_to_tags_map INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id WHERE manga.id = ?1 GROUP BY manga.id")?;
        let mut stmt2= conn.prepare("SELECT id, title, title_romanized, url, url_with_chapter, chapter, last_update, notes, tags, \
                                                            my_anime_list FROM manga GROUP BY title_romanized HAVING COUNT(*) > 1")?;
        let manga_iter = stmt.query_map([], |row| {
            Ok(MangaModel {
                id: row.get(0)?,
                title: row.get(1)?,
                title_romanized: row.get(2)?,
                url: row.get(3)?,
                url_with_chapter: row.get(4)?,
                chapter: row.get(5)?,
                last_update: row.get(6)?,
                notes: row.get(7)?,
                tags: match row.get::<usize, String>(8) {
                    Ok(t) => t.split(",").map(|s| s.to_string()).collect(),
                    Err(_) => Vec::new(),
                },
                my_anime_list: row.get(9)?,
            })
        })?;

        let mut manga_vec = Vec::new();
        for manga in manga_iter {
            manga_vec.push(manga.unwrap());
        }

        Ok(manga_vec)
    }
}
