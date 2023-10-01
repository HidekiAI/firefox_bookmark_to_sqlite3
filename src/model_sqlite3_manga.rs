use crate::model_manga;
use rusqlite;

// SQLite3 Manga data model
// TABLE manga_no_id:
// id, title, title_romanized, url, chapter, url_with_chapter, last_update, notes, tags
pub mod model_sqlite3_manga {
    use core::panic;
    use rusqlite::{
        params,
        types::{FromSql, FromSqlError},
        Connection, Params, Result, Row, ToSql,
    };
    use std::{collections::HashMap, error::Error, f32::consts::E, path::Path};

    use crate::{
        model_manga::model_manga::MangaModel,
        my_utils::{make_none_if_empty, trim_quotes},
    };

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
    // Example of how UPSERT works on SQLite3:
    //      CREATE TABLE vocabulary (word TEXT PRIMARY KEY, count INT DEFAULT 1);
    //      INSERT INTO vocabulary (word) VALUES ('jovial') ON CONFLICT (word) DO UPDATE SET count=count+1;
    // the above will insert 'jovial' into vocabulary table if it doesn't exist, and if it does exist, it will increment the count by 1

    fn create_manga_table(db_full_paths: &str) -> Result<()> {
        println!(">> create_manga_table('{}')", db_full_paths);
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS manga (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,    -- uniqueness of this table is based on the pair title and url, so no need for primary key
                title_romanized TEXT,
                url TEXT NOT NULL,
                url_with_chapter TEXT,
                chapter TEXT,
                last_update TEXT,
                notes TEXT,
                tags TEXT,  -- just preserve the tags that may have come from original
                my_anime_list TEXT,
                UNIQUE(title, url)
            )",
            [],
        )?;

        Ok(())
    }

    // table wich has foreign key to manga table and tags table, and is the intermediary table
    fn create_manga_to_tags_map_table(db_full_paths: &str) -> Result<()> {
        println!(">> create_manga_to_tags_map_table('{}')", db_full_paths);

        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS manga_to_tags_map (
                manga_id INTEGER,
                tag_id INTEGER,
                FOREIGN KEY(manga_id) REFERENCES manga(id),
                FOREIGN KEY(tag_id) REFERENCES tags(id),
                UNIQUE(manga_id, tag_id)
            )",
            [],
        )?;

        Ok(())
    }

    fn create_tags_table(db_full_paths: &str) -> Result<()> {
        println!(">> create_tags_table('{}')", db_full_paths);

        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tag TEXT NOT NULL UNIQUE,
                UNIQUE(tag)
            )",
            [],
        )?;

        Ok(())
    }

    pub fn create_tables(db_full_paths: &str) -> Result<()> {
        println!("> create_tables('{}')", db_full_paths);
        create_manga_table(db_full_paths)?;
        create_manga_to_tags_map_table(db_full_paths)?;
        create_tags_table(db_full_paths)?;

        Ok(())
    }

    // sql_where_clause - example: "WHERE title LIKE ?1 AND url LIKE ?2"
    fn select_manga(
        db_full_paths: &str,
        sql_where_clause: &str, /* , args: &[&str] */
    ) -> Result<Vec<MangaModel>> {
        let select_stmt =
        format!(
            "SELECT m.id, m.title, m.title_romanized, m.url, m.url_with_chapter, m.chapter, m.last_update, m.notes, m.my_anime_list,
                    (SELECT GROUP_CONCAT(t.tag, ', ')
                        FROM manga_to_tags_map AS mt
                        JOIN tags AS t ON mt.tag_id = t.id
                        WHERE mt.manga_id = m.id) AS tags
                FROM manga AS m {} ;", sql_where_clause);
        match Connection::open(db_full_paths) {
            Ok(conn) => {
                match conn.prepare(select_stmt.as_str()) {
                    Ok(mut stmt) => {
                        //let args_joined = args.join(",");   // Join the strings with a delimiter (as params)
                        //let sql_params = params![&args_joined]; // Now you can use args_joined in params!
                        let sql_params = params![]; // Empty
                        match stmt.query(sql_params) {
                            Ok(mut rows) => {
                                let mut manga_data = Vec::new();
                                let mut possible_next_row = match rows.next() {
                                    Ok(r) => r,
                                    Err(e) => {
                                        println!("ERROR: Failed to get next row: {}", e);
                                        None
                                    }
                                };
                                let transform_column = |col: Result<Option<String>, rusqlite::Error> | -> Result<Option<String>> {
                                    match col {
                                        Ok(Some(t)) => Ok(make_none_if_empty( Some(t))),
                                        Ok(None) => Ok(None),
                                        Err(e) => {
                                            println!("ERROR: Failed to get column: {}", e);
                                            Err(e)
                                        }
                                    }
                                } ;
                                let transform_row = |row: &Row| -> Result<MangaModel> {
                                    Ok(MangaModel::with_values(
                                        row.get(0)?,
                                        row.get(1)?,
                                        transform_column(row.get(2))?,
                                        row.get(3)?,
                                        transform_column(row.get(4))?,
                                        transform_column(row.get(5))?,
                                        transform_column(row.get(6))?,
                                        transform_column(row.get(7))?,
                                        match row.get::<usize, String>(9) {
                                            Ok(t) => t.split(",").map(|s| s.to_string()).collect(),
                                            Err(_) => Vec::new(),
                                        },
                                        transform_column(row.get(8))?,
                                    ))
                                };
                                while let Some(row) = possible_next_row {
                                    manga_data.push(transform_row(&row)?);
                                    possible_next_row = match rows.next() {
                                        Ok(r) => r,
                                        Err(e) => {
                                            println!("ERROR: Failed to get next row: {}", e);
                                            None
                                        }
                                    };
                                }
                                // NOTE: It is NOT AN ERROR if we get 0 rows, so we'll just return empty Vec
                                // but it is up to the caller to treat it as Error::QueryReturnedNoRows (i.e. get_id(ID) where ID SHOULD have existed)
                                Ok(manga_data)
                            }
                            Err(e) => {
                                // most likely, it's because args/parsms are not correct
                                println!("ERROR: Failed to query: {}", e);
                                Err(e.into())
                            }
                        }
                    }
                    Err(e) => {
                        println!("ERROR: Failed to prepare statement: {}", e);
                        Err(e.into())
                    }
                }
            }
            Err(e) => {
                println!("ERROR: Failed to open dataae '{}' : {}", db_full_paths, e);
                Err(e)
            }
        }
    }

    // Insert MangaModel (without id field, id=0) and associate tags if any, and return new MangaModel with real/valid id
    pub fn insert_manga(db_full_paths: &str, manga_no_id: &MangaModel) -> Result<MangaModel> {
        #[cfg(debug_assertions)]
        {
            println!("INSERT manga (no ID): {:?}", manga_no_id);
        }
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        // Option based vars needs to become concrete before we can use them in query
        // first, insert MangaModel so that we can get the id
        conn.execute(
            "INSERT INTO manga (title, title_romanized, url, url_with_chapter, chapter, last_update, notes, tags, my_anime_list) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            &[
                &manga_no_id.title(),
                match &manga_no_id.title_romanized() { Some(t) => &t.as_str(), None => ""   }, 
                &manga_no_id.url (),
                match &manga_no_id.url_with_chapter (){ Some(t) => &t.as_str(), None => ""   },
                match &manga_no_id.chapter() { Some(t) => &t.as_str(), None => ""   },
                match &manga_no_id.last_update() { Some(t) => &t.as_str(), None => ""   },
                match &manga_no_id.notes() { Some(t) => &t.as_str(), None => ""   },
                &manga_no_id.tags().join(","), 
                match &manga_no_id.my_anime_list (){ Some(t) => &t.as_str(), None => ""   },
                ],
        )?; //bail on error

        // update MangaModel with the id
        let id = conn.last_insert_rowid() as u32;
        let mut manga = manga_no_id.clone();
        manga.set_id(id);

        #[cfg(debug_assertions)]
        {
            println!("> INSERT succeeded, new ID: {}", id);
        }
        // second, insert tags if any
        if manga.tags().len() > 0 {
            #[cfg(debug_assertions)]
            {
                println!("> INSERT tags: {:?}", manga.tags());
            }
            for tag in manga.tags().clone() {
                // insert tag if not exists (case insensitive)
                conn.execute("INSERT OR IGNORE INTO tags (tag) VALUES (?1)", &[&tag])?;
                // get tag id
                let mut stmt = conn.prepare("SELECT id FROM tags WHERE tag = ?1")?;
                let mut tag_iter = stmt.query_map(&[&tag], |row| Ok(row.get(0)?))?;

                let tag_id = tag_iter.next().unwrap().unwrap();
                // insert tag id and manga id into manga_to_tags_map table if the pair does not yet exists (shouldn't exists, but just in case)
                match conn.execute(
                    "INSERT OR IGNORE INTO manga_to_tags_map (manga_id, tag_id) VALUES (?1, ?2)",
                    &[&id, &tag_id],
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        {
                            println!(
                                "ERROR: insert_manga: failed inserting into manga_to_tags_map: {}",
                                e
                            );
                        }
                    }
                }
            }
        }
        #[cfg(debug_assertions)]
        {
            println!(
                ">> INSERT tags '{}' succeeded for ID: {}",
                manga.tags().join(";"),
                id
            );
        }
        Ok(manga.clone())
    }

    // update based on id field - note that return is stdError rather than updated MangaModel, since what's being
    // updated was/is what was passed in here
    pub fn update_manga(
        db_full_paths: &str,
        manga: &MangaModel,
        //) -> Result<(), Box<dyn std::error::Error + 'static>> {
    ) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(debug_assertions)]
        {
            println!(
                "# UPDATE manga: update_manga('{}', {:?})",
                db_full_paths, manga
            )
        }
        // fail if id (u32) is 0
        if manga.id() == 0 {
            return Err("id cannot be 0".into());
        }

        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        // if we cannot locate id in manga table during update, return error (most likely got deleted)
        // query for id and title (just in case we need to return the title)
        let mut stmt = conn.prepare("SELECT id FROM manga WHERE id = ?1")?; // returns either 0 or 1 row
        let manga_iter = stmt.query_map(&[&manga.id()], |row| {
            row.get::<usize, i32>(0) // id is i32 type...
        })?;
        if manga_iter.count() == 0 {
            return Err("id not found".into());
        }

        // OK, id exists, so proceed with update
        conn.execute(
            "UPDATE manga SET title = ?1, title_romanized = ?2, url = ?3, url_with_chapter = ?4, chapter = ?5, last_update = ?6, notes = ?7, tags = ?8, my_anime_list = ?9 WHERE id = ?10",
            &[
                &manga.title(),
                match &manga.title_romanized() { Some(t) => &t.as_str(), None => ""   }, 
                &manga.url (),
                match &manga.url_with_chapter() { Some(t) => &t.as_str(), None => ""   },
                match &manga.chapter() { Some(t) => &t.as_str(), None => ""   },
                match &manga.last_update() { Some(t) => &t.as_str(), None => ""   },
                match &manga.notes() { Some(t) => &t.as_str(), None => ""   },
                &manga.tags().join(","), 
                match &manga.my_anime_list() { Some(t) => &t.as_str(), None => ""   },
                &manga.id().to_string(),
                ],
        )?;

        #[cfg(debug_assertions)]
        {
            println!("> UPDATE succeeded for ID: {}", manga.id());
        }
        Ok(())
    }

    // we can use this to insert manga if it doesn't exist, and if it does exist, we can update it
    // however, we need to make sure that the title and url are unique, so we need to check if it exists first
    // if it does exist, we can update it, if it doesn't exist, we can insert it
    pub fn upsert_manga(db_full_paths: &str, manga_no_id: &MangaModel) -> Result<MangaModel> {
        #[cfg(debug_assertions)]
        {
            println!("# upsert_manga('{}', {:?})", db_full_paths, manga_no_id);
        }
        // first, check if title+url unique exists and if so, update rather than insert
        let manga = match select_manga_from_url_and_title(
            db_full_paths,
            &manga_no_id.url(),
            &manga_no_id.title(),
        ) {
            Ok(found_model) => {
                #[cfg(debug_assertions)]
                {
                    println!("> upsert_manga: found_model: {:?}", found_model);
                }
                // because we're using exact title and url, we should only get 1 row
                if found_model.len() > 1 {
                    // if here, it means we got more than 1 row, which is not good, so return error
                    return Err(rusqlite::Error::InvalidParameterName(
                        "more than 1 row found".to_string(),
                    ));
                }
                // if here, we can now assume that we got 1 row, so use the id from the found row and use the data of what was passed
                let top_row = found_model.get(0).unwrap();

                // row exists, use the ID from the found row and use the data of what was passed
                let mut manga = manga_no_id.clone();
                manga.set_id(top_row.id());

                // if here, it means we found manga based on title and url, so update it
                match update_manga(db_full_paths, &manga) {
                    Ok(()) => {
                        // if here, it means we successfully updated manga, so return it
                        Ok(manga)
                    }
                    Err(update_error) => {
                        // if here, it means we cannot update manga, so return error
                        // return anything other than Err(rusqlite::Error::QueryReturnedNoRows)
                        let str_error = update_error.to_string();
                        Err(rusqlite::Error::InvalidParameterName(str_error)) // for now, we'll just use this error type
                    }
                }
            }
            Err(select_error) => {
                // depending on type of error, proceed to INSERT it (i.e. not found) or return error
                match select_error {
                    rusqlite::Error::QueryReturnedNoRows => {
                        #[cfg(debug_assertions)]
                        {
                            println!("# SELECT returned 0 rows while searching for title='{}'+url='{}'; inserting instead", manga_no_id.title(), manga_no_id.url());
                        }
                        // if here, it means we cannot find manga based on title and url, so insert it
                        insert_manga(db_full_paths, manga_no_id)
                    }
                    _ => {
                        #[cfg(debug_assertions)]
                        {
                            println!("ERROR: upsert_manga: failed calling select_manga_from_url_and_title: {:?}", select_error);
                        }
                        Err(select_error)
                    }
                }
            }
        };
        manga
    }

    // delete the row based on id field
    pub fn delete_manga(db_full_paths: &str, id: u32) -> Result<bool> {
        println!("DELETE: delete_manga('{}', {})", db_full_paths, id);

        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        // if we cannot locate id in manga table during delete, just return Ok(false) (most likely got deleted)
        // should only return single row since we're using id as primary key
        // haven't had time to investigate, but if I just return single column (SELECT id) I'd get an error, so I'm returning 2 columns (id and title) and just ignore the title
        let mut stmt = conn.prepare("SELECT id FROM manga WHERE id = ?1")?; // returns either 0 or 1 row
        let manga_iter = stmt.query_map(&[&id], |row| {
            row.get::<usize, i32>(0) // id is i32 type...
        })?;
        if manga_iter.count() == 0 {
            #[cfg(debug_assertions)]
            {}
            return Ok(false); // just bail out with a warning...
        }

        // if here, id existed, so proceed with delete
        match conn.execute("DELETE FROM manga WHERE id = ?1", &[&id]) {
            Ok(_) => {
                // delete tags ONLY if we were able to delete from manga table
                // also prune tags group table (again, if cannot find, it's OK)
                match conn.execute("DELETE FROM manga_to_tags_map WHERE manga_id = ?1", &[&id]) {
                    Ok(_) => {
                        println!("> DELETE succeeded for ID={}", id);
                    }
                    Err(e) => {
                        // if no rows were found in tags map table, it should still be considered as a success
                        if e != rusqlite::Error::QueryReturnedNoRows {
                            println!( "ERROR: delete_manga(id={}): failed deleting from manga_to_tags_map: {}", id, e);
                            return Err(e.into());
                        } else {
                            println!(
                                "> DELETE succeeded for ID={} (no tags mapping found for this ID)",
                                id
                            );
                        }
                    }
                }
            }
            Err(e) => {
                println!(
                    "ERROR: delete_manga(id={}): failed deleting from manga: {}",
                    id, e
                );
                return Err(e.into());
            }
        }

        Ok(true)
    }

    // get ID based on title and url (as it's unique combination) and return in manga struct WITH the ID
    pub fn get_id(db_full_paths: &str, title: &str, url: &str) -> Result<MangaModel> {
        // fail if title or url is empty, or has/contains "%" wildcards
        if title.len() == 0 || url.len() == 0 || title.contains("%") || url.contains("%") {
            return Err(rusqlite::Error::InvalidParameterName(format!(
                "get_id(): title '{}' or url '{}' is empty, or has/contains '%' wildcards",
                title, url
            )));
        }

        // title+url is the unique constraints which resolves to a single row (unique ID), note that we DO NOT use LIKE clause here
        let where_clause = format!("WHERE m.title = '{}' AND m.url = '{}'", title, url);
        let row_sets_result = select_manga(db_full_paths, where_clause.as_str());
        match row_sets_result {
            Ok(row_sets) => {
                if row_sets.len() == 0 {
                    return Err(rusqlite::Error::QueryReturnedNoRows);
                }
                if row_sets.len() > 1 {
                    // title+url SHOULD be unique, hence we should only get 1 row, something is wrong if we get more than 1 row
                    // so we'll at least warn about it, but take the first row anyway
                    println!("WARNING: get_id(): title='{}' and url='{}' returned more than 1 row, using the first row", title, url);
                    #[cfg(debug_assertions)]
                    {
                        // in debug mode, also dump all the rows round
                        for row in row_sets.clone() {
                            println!("> {:?}", row);
                        }
                    }
                }
                match row_sets.get(0) {
                    Some(row_manga) => Ok(row_manga.clone()),
                    None => Err(rusqlite::Error::QueryReturnedNoRows),
                }
            }
            Err(e) => Err(e),
        }
    }

    // return in manga struct based on ID
    pub fn select_manga_by_id(db_full_paths: &str, id: u32) -> Result<MangaModel> {
        // ID is unique, so we should only get 1 row
        let where_clause = format!("WHERE m.id = {}", id);
        let row_sets_result = select_manga(db_full_paths, where_clause.as_str());
        match row_sets_result {
            Ok(row_sets) => {
                if row_sets.len() == 0 {
                    return Err(rusqlite::Error::QueryReturnedNoRows);
                }
                if row_sets.len() > 1 {
                    // ID should be unique, should NEVER happen, so panic!() here (this is serious)
                    panic!("select_manga_by_id(): ID='{}' returned more than 1 row, using the first row for database '{}'", id, db_full_paths);
                }
                match row_sets.get(0) {
                    Some(row_manga) => Ok(row_manga.clone()),
                    None => Err(rusqlite::Error::QueryReturnedNoRows),
                }
            }
            Err(e) => Err(e),
        }
    }

    // return in manga struct array
    pub fn select_all_manga(db_full_paths: &str) -> Result<Vec<MangaModel>> {
        // just seek/query for all rows where ID > 0 (i.e. all rows)
        let where_clause = format!("WHERE m.id > 0");
        let row_sets_result = select_manga(db_full_paths, where_clause.as_str());
        match row_sets_result {
            Ok(row_sets) => {
                if row_sets.len() == 0 {
                    return Err(rusqlite::Error::QueryReturnedNoRows);
                }
                Ok(row_sets)
            }
            Err(e) => Err(e),
        }
    }

    // in most cases, we do not need to specialize a method since all one has to do is setup their WHERE clause
    // to their likings as query fits their needs, but since it's mostly common to seek/query for row-sets based
    // on either/or title and/or url, we'll provide a specialized method for that here with boiler plate error
    // handling.  When other than title/url is needed, one can also just use this function as a template on
    // how to query
    pub fn select_manga_from_url_and_title(
        db_full_paths: &str,
        url: &str,
        title: &str,
    ) -> Result<Vec<MangaModel>> {
        // unlike get_id(), this method allows wildcards in title and url, BUT neither can be empty (caller should
        // opt to set it to "%" wildcard ir only care about one or the other)
        if url.len() == 0 || title.len() == 0 {
            return Err(rusqlite::Error::InvalidParameterName(format!(
                "select_manga_from_url_and_title: url '{}' or title '{}' is empty",
                url, title
            )));
        }
        // Note that we'll use LIKE clause here instead of "=" in case caller wants to use wildcards
        let where_clause = format!("WHERE m.url LIKE '{}' AND m.title LIKE '{}'", url, title);
        let row_set_result = select_manga(db_full_paths, where_clause.as_str());
        match row_set_result {
            Ok(row_sets) => {
                // NOTE: Though it is NOT an error to get 0 rows, we'll return it as error anyway so that calling method doesn't need to check for 0 rows
                if row_sets.len() == 0 {
                    return Err(rusqlite::Error::QueryReturnedNoRows);
                }
                Ok(row_sets)
            }
            Err(e) => Err(e),
        }
    }

    // locate titles that are in JA_JP and see if it can find rows that have same URL but title is in
    // romanized or EN_US; and if so, drop/delete the EN_US row and report that it's removed it
    // from the database
    //    pub fn prune_duplicates(db_full_paths: &str) -> Result<Vec<MangaModel>> // returns lists of deleted rows
    //    {
    //        // for each rows that has title in JA_JP, locate rows (manga.id) which has same BASE url or romanized title
    //        // and map manga_id to manga_ids (i.e. single ID mapped to 1 or more IDs)
    //
    //    }

    #[cfg(test)]
    mod tests {
        use crate::model_manga::{self, model_manga::MangaModel};
        const K_MANGA_TITLE: &str = "ゲート―自衛隊彼の地にて、斯く戦えり";
        const K_MANGA_URL_WITH_CHAPTERS: &str = "https://example.com/manga/gate-chapter-10/";

        fn make_sample_row() -> MangaModel {
            let model = MangaModel::new_from_required_elements(
                K_MANGA_TITLE,
                K_MANGA_URL_WITH_CHAPTERS,
                model_manga::CASTAGNOLI.checksum(K_MANGA_TITLE.as_bytes()),
            )
            .unwrap(); // just unwrap(), in unit-test, we make assumptions that it will always succeed
            model
        }

        // test for create_tables
        #[test]
        fn test_create_tables() {
            //let db_full_paths = "/dev/shm/test_create_tables.db"; // only on Linux...
            let db_full_paths = "tests/test_create_tables.db";
            // first, delete it if exists
            std::fs::remove_file(db_full_paths).unwrap_or_default();

            // now create it
            super::create_tables(db_full_paths).unwrap();

            // now try creating again, this time with the file existing
            super::create_tables(db_full_paths).unwrap();

            // clean up
            std::fs::remove_file(db_full_paths).unwrap();
        }

        // test for insert_manga
        #[test]
        fn test_insert_manga() {
            let db_full_paths = "tests/test_insert_manga.db";
            // first, delete it if exists
            std::fs::remove_file(db_full_paths).unwrap_or_default();

            // now create it
            super::create_tables(db_full_paths).unwrap();

            // now try inserting
            let mut manga_no_id = make_sample_row();
            let manga_inserted = super::insert_manga(db_full_paths, &manga_no_id).unwrap();

            // now try inserting again, with same title+url (should fail due to unique constraints)
            let manga_inserted_wrapped = super::insert_manga(db_full_paths, &manga_inserted);
            assert!(manga_inserted_wrapped.is_err());

            // this time, call upsert instead
            let manga_upserted = super::upsert_manga(db_full_paths, &manga_no_id); // should succeed either way, for it will attempt to "update" if exists
            assert!(manga_upserted.is_ok());
            println!("manga_upserted: {:?}", manga_upserted);

            //// us search println!("\nsearching manga for all wildcards");
            //match super::fetch_manga_data2(db_full_paths, "%", "%") {
            //    Ok(manga_vec) => {
            //        println!("> Found {} rows", manga_vec.len());
            //        for (manga) in manga_vec {
            //            println!("manga: {:?}", manga);
            //        }
            //    }
            //    Err(e) => {
            //        println!("ERROR: test_insert_manga: search_manga: {}", e);
            //        assert!(false);
            //    }
            //};

            let manga_vec = super::select_all_manga(db_full_paths).unwrap();
            for m in manga_vec {
                println!("test_insert_manga(id={}): {:?}", m.id(), m);
                //println!("manga: {}|{}|{}|{:?}", m.id, m.title, m.url, m.title_romanized);

                // make sure we do NOT have any parsing issue of Some("") -> should be None
                if m.title_romanized().is_some() {
                    // fail if title_romanized is Some("")
                    assert!(!m.title_romanized().clone().unwrap().is_empty());
                }
                if m.url_with_chapter().is_some() {
                    // fail if url_with_chapter is Some("")
                    assert!(!m.url_with_chapter().clone().unwrap().is_empty());
                }
                if m.chapter().is_some() {
                    // fail if chapter is Some("")
                    assert!(!m.chapter().clone().unwrap().is_empty());
                }
                if m.last_update().is_some() {
                    // fail if last_update is Some("")
                    assert!(!m.last_update().clone().unwrap().is_empty());
                }
                if m.notes().is_some() {
                    // fail if notes is Some("")
                    assert!(!m.notes().clone().unwrap().is_empty());
                }
                if m.my_anime_list().is_some() {
                    // fail if my_anime_list is Some("")
                    assert!(!m.my_anime_list().clone().unwrap().is_empty());
                }

                if manga_inserted.id() == m.id() {
                    assert_eq!(manga_inserted.id(), m.id());
                    assert_eq!(manga_inserted.title(), m.title());
                    assert_eq!(manga_inserted.title_romanized(), m.title_romanized());
                    assert_eq!(manga_inserted.url(), m.url());
                    assert_eq!(manga_inserted.url_with_chapter(), m.url_with_chapter());
                    assert_eq!(manga_inserted.chapter(), m.chapter());
                    assert_eq!(manga_inserted.last_update(), m.last_update());
                    assert_eq!(manga_inserted.notes(), m.notes());
                    assert_eq!(manga_inserted.tags(), m.tags());
                    assert_eq!(manga_inserted.my_anime_list(), m.my_anime_list());
                }
            }

            // clean up
            std::fs::remove_file(db_full_paths).unwrap();
        }

        //#[test]
        //fn test_fetch_manga_data() {
        //    let db_file_path = "tests/test_fetch_manga_test.sqlite3"; // Replace with your actual database file path
        //    let title = "%"; // Replace with the title you want to search for
        //    let url = "%"; // Replace with the URL you want to search for

        //    match super::fetch_manga_data2(db_file_path, title, url) {
        //        Ok(manga_data) => {
        //            for manga in manga_data {
        //                println!("{}|{}|{}", manga.id, manga.title, manga.url);
        //            }
        //        }
        //        Err(err) => {
        //            eprintln!("Error: {:?}", err);
        //        }
        //    }
        //}

        #[test]
        fn test_select_manga() {
            let db_file_path = "tests/test_select_manga.sqlite3"; // Replace with your actual database file path
                                                                  //let sql_where_clause = format!("WHERE m.url LIKE '{}' AND m.title LIKE '{}'", "%", "%フロンティア%");
            let sql_where_clause = format!("WHERE m.url LIKE '{}' AND m.title LIKE '{}'", "%", "%");
            match super::select_manga(db_file_path, sql_where_clause.as_str()) {
                Ok(manga_data) => {
                    for manga in manga_data {
                        println!("{}|{}|{}", manga.id(), manga.title(), manga.url());
                    }
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                }
            }
        }
    }
}
