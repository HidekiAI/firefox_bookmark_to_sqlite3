use crate::model_manga;
use rusqlite;

// SQLite3 Manga data model
// TABLE manga_no_id:
// id, title, title_romanized, url, chapter, url_with_chapter, last_update, notes, tags
pub mod model_sqlite3_manga {
    use rusqlite::{
        params,
        types::{FromSql, FromSqlError},
        Connection, Result, Row,
    };
    use std::{error::Error, f32::consts::E, path::Path};

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
    // Example of how UPSERT works on SQLite3:
    //      CREATE TABLE vocabulary (word TEXT PRIMARY KEY, count INT DEFAULT 1);
    //      INSERT INTO vocabulary (word) VALUES ('jovial') ON CONFLICT (word) DO UPDATE SET count=count+1;
    // the above will insert 'jovial' into vocabulary table if it doesn't exist, and if it does exist, it will increment the count by 1

    fn create_manga_table(db_full_paths: &str) -> Result<()> {
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
        create_manga_table(db_full_paths)?;
        create_manga_to_tags_map_table(db_full_paths)?;
        create_tags_table(db_full_paths)?;

        Ok(())
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
        )?; //bail on error

        // update MangaModel with the id
        let id = conn.last_insert_rowid() as u32;
        let mut manga = manga_no_id.clone();
        manga.id = id;

        #[cfg(debug_assertions)]
        {
            println!("> INSERT succeeded, new ID: {}", id);
        }
        // second, insert tags if any
        if manga.tags.len() > 0 {
            #[cfg(debug_assertions)]
            {
                println!("> INSERT tags: {:?}", manga.tags);
            }
            for tag in manga.tags.clone() {
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
            println!(">> INSERT tas succeeded for ID: {}", id);
        }
        Ok(manga.clone())
    }

    // update based on id field - note that return is stdError rather than updated MangaModel, since what's being
    // updated was/is what was passed in here
    pub fn update_manga(
        db_full_paths: &str,
        manga: &MangaModel,
    ) -> Result<(), Box<dyn std::error::Error + 'static>> {
        // fail if id (u32) is 0
        if manga.id == 0 {
            return Err("id cannot be 0".into());
        }

        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        // if we cannot locate id in manga table during update, return error (most likely got deleted)
        // query for id and title (just in case we need to return the title)
        let mut stmt = conn.prepare("SELECT id FROM manga WHERE id = ?1")?; // returns either 0 or 1 row
        let manga_iter = stmt.query_map(&[&manga.id], |row| {
            row.get::<usize, i32>(0) // id is i32 type...
        })?;
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

    // we can use this to insert manga if it doesn't exist, and if it does exist, we can update it
    // however, we need to make sure that the title and url are unique, so we need to check if it exists first
    // if it does exist, we can update it, if it doesn't exist, we can insert it
    pub fn upsert_manga(db_full_paths: &str, manga_no_id: &MangaModel) -> Result<MangaModel> {
        #[cfg(debug_assertions)]
        {
            println!("upsert_manga: {:?}", manga_no_id);
        }
        // first, check if title+url unique exists and if so, update rather than insert
        let manga = match select_manga_from_url_and_title(
            db_full_paths,
            &manga_no_id.url,
            &manga_no_id.title,
        ) {
            Ok(found_model) => {
                #[cfg(debug_assertions)]
                {
                    println!("upsert_manga: found_model: {:?}", found_model);
                }
                // because we're using exact title and url, we should only get 1 row
                if (found_model.len() > 1) {
                    // if here, it means we got more than 1 row, which is not good, so return error
                    return Err(rusqlite::Error::InvalidParameterName(
                        "more than 1 row found".to_string(),
                    ));
                }
                // if here, we can now assume that we got 1 row, so use the id from the found row and use the data of what was passed
                let top_row = found_model.get(0).unwrap();

                // row exists, use the ID from the found row and use the data of what was passed
                let mut manga = manga_no_id.clone();
                manga.id = top_row.id;

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
                            println!("# SELECT returned 0 rows while searching for title='{}'+url='{}'; inserting instead", manga_no_id.title, manga_no_id.url);
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
            return Ok(false); // just bail out with a warning...
        }

        // if here, id existed, so proceed with delete
        conn.execute("DELETE FROM manga WHERE id = ?1", &[&id])?;

        // also prune tags group table (again, if cannot find, it's OK)
        conn.execute("DELETE FROM manga_to_tags_map WHERE manga_id = ?1", &[&id])?;

        Ok(true)
    }

    // get ID based on title and url (as it's unique combination) and return in manga struct WITH the ID
    pub fn get_id(db_full_paths: &str, manga_no_id: &MangaModel) -> Result<MangaModel> {
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;

        // SELECT id from manga table based on title and url
        // SELECT id FROM manga WHERE title = ?1 AND url = ?2
        let mut stmt = conn.prepare("SELECT id FROM manga WHERE title = ?1 AND url = ?2")?;
        let manga_iter = stmt.query_map(&[&manga_no_id.title, &manga_no_id.url], |row| {
            Ok(MangaModel {
                id: row.get(0)?,
                title: manga_no_id.title.clone(),
                title_romanized: manga_no_id.title_romanized.clone(),
                url: manga_no_id.url.clone(),
                url_with_chapter: manga_no_id.url_with_chapter.clone(),
                chapter: manga_no_id.chapter.clone(),
                last_update: manga_no_id.last_update.clone(),
                notes: manga_no_id.notes.clone(),
                tags: manga_no_id.tags.clone(),
                my_anime_list: manga_no_id.my_anime_list.clone(),
            })
        })?;

        for manga in manga_iter {
            return Ok(manga.unwrap());
        }

        Err(rusqlite::Error::QueryReturnedNoRows)
    }

    fn fetch_manga_data(db_file_path: &str, title: &str, url: &str) -> Result<Vec<MangaModel>> {
        let conn = Connection::open(db_file_path)?;

        let mut stmt = conn.prepare(
            "SELECT m.id, m.title, m.title_romanized, m.url, m.url_with_chapter, m.chapter, m.last_update, m.notes, m.my_anime_list,
                    (SELECT GROUP_CONCAT(t.tag, ', ')
                        FROM manga_to_tags_map AS mt
                        JOIN tags AS t ON mt.tag_id = t.id
                        WHERE mt.manga_id = m.id) AS tags
                FROM manga AS m
                WHERE m.title LIKE ?1
                AND m.url LIKE ?2;",
        )?;

        let mut rows = stmt.query(params![title, url])?;

        let mut manga_data = Vec::new();
        while let Some(row) = rows.next()? {
            manga_data.push(MangaModel {
                id: row.get(0)?,
                title: row.get(1)?,
                title_romanized: row.get(2)?,
                url: row.get(3)?,
                url_with_chapter: row.get(4)?,
                chapter: row.get(5)?,
                last_update: row.get(6)?,
                notes: row.get(7)?,
                tags: match row.get::<usize, String>(9) {
                    Ok(t) => t.split(",").map(|s| s.to_string()).collect(),
                    Err(_) => Vec::new(),
                },
                my_anime_list: row.get(8)?,
            });
        }

        Ok(manga_data)
    }

    pub fn fetch_manga_data2(
        db_full_paths: &str,
        title: &str,
        url: &str,
    ) -> Result<Vec<MangaModel>> {
        match Connection::open(db_full_paths) {
            Ok(conn) => {
                match conn.prepare(
                        "SELECT m.id, m.title, m.title_romanized, m.url, m.url_with_chapter, m.chapter, m.last_update, m.notes, m.my_anime_list,
                                (SELECT GROUP_CONCAT(t.tag, ', ')
                                    FROM manga_to_tags_map AS mt
                                    JOIN tags AS t ON mt.tag_id = t.id
                                    WHERE mt.manga_id = m.id) AS tags
                            FROM manga AS m
                            WHERE m.title LIKE ?1
                            AND m.url LIKE ?2;",
                    )
                    {
                        Ok(mut stmt) =>{

                                    match stmt.query(params![title, url]){
                                        Ok(mut rows ) => {
                                            let mut manga_data = Vec::new();
                                            let mut possible_next_row = match rows.next() {
                                                Ok(r) => r,
                                                Err(e) => {
                                                    println!("ERROR: Failed to get next row: {}", e);
                                                    None
                                                }
                                            };
                                            while let Some(row) = possible_next_row {
                                                manga_data.push(MangaModel {
                                                    id: row.get(0)?,
                                                    title: row.get(1)?,
                                                    title_romanized: row.get(2)?,
                                                    url: row.get(3)?,
                                                    url_with_chapter: row.get(4)?,
                                                    chapter: row.get(5)?,
                                                    last_update: row.get(6)?,
                                                    notes: row.get(7)?,
                tags: match row.get::<usize, String>(9) {
                    Ok(t) => t.split(",").map(|s| s.to_string()).collect(),
                    Err(_) => Vec::new(),
                },
                                                    my_anime_list: row.get(8)?,
                                                });
                                                possible_next_row = match rows.next() {
                                                    Ok(r) => r,
                                                    Err(e) => {
                                                        println!("ERROR: Failed to get next row: {}", e);
                                                        None
                                                    }
                                                };
                                            }
                                            Ok(manga_data)
                                        }
                                        Err(e) => {
                                            println!("ERROR: Failed to query: {}", e);
                                            Err(e)
                                        }
                                    }

                        }
                        Err(e) => {
                            println!("ERROR: Failed to prepare statement: {}", e);
                            Err(e)
                        }
                    }

            } ,
            Err(e) => {
                            println!("ERROR: Failed to open dataae '{}' : {}", db_full_paths, e);
                            Err(e)
            }
        }
    }

    pub fn select_manga_from_url_and_title(
        db_full_paths: &str,
        url: &str,
        title: &str,
    ) -> Result<Vec<MangaModel>> {
        #[cfg(debug_assertions)]
        {
            println!(
                "\n\nSELCT select_manga_from_url_and_title SELECT: url: '{}', title: '{}'",
                url, title
            );
        }
        let path = Path::new(db_full_paths);
        let conn = Connection::open(path)?;
        //  sqlite> SELECT m.id, m.title, m.title_romanized, m.url, m.url_with_chapter, m.chapter, m.last_update, m.notes, m.my_anime_list,
        //         (SELECT GROUP_CONCAT(t.tag, ', ')
        //              FROM manga_to_tags_map AS mt
        //              JOIN tags AS t ON mt.tag_id = t.id
        //              WHERE mt.manga_id = m.id) AS tags
        //      FROM manga AS m
        //      WHERE m.title LIKE '%よくわからないけれど異世界に転生していたようです%' AND m.url LIKE '%https://rawkuma.com/yoku-wakaranai-keredo-isekai-ni-tensei-shiteita-you-desu/%';
        //  > 2|よくわからないけれど異世界に転生していたようです|yokuwakaranaikeredo isekai ni tenshou shiteitayoudesu|https://rawkuma.com/yoku-wakaranai-keredo-isekai-ni-tensei-shiteita-you-desu/|https://rawkuma.com/yoku-wakaranai-keredo-isekai-ni-tensei-shiteita-you-desu-chapter-88-2/|88.2|||||Mystery
        let stmt_result = conn.prepare(
            "SELECT manga.id, manga.title, manga.title_romanized, manga.url, manga.url_with_chapter, manga.chapter, \
                        manga.last_update, manga.notes, manga.my_anime_list,                                            \
                        GROUP_CONCAT(tags.tag, ',') AS tags                                                             \
                    FROM manga_to_tags_map                                                                              \
                    INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id                                           \
                    INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id                                               \
                    WHERE manga.title LIKE ?1                                                                           \
                        AND manga.url LIKE ?2                                                                           \
                    GROUP BY manga.id",
        );
        let mut stmt = match stmt_result {
            Ok(s) => s,
            Err(e) => {
                #[cfg(debug_assertions)]
                {
                    println!(
                        "ERROR: select_manga_from_url_and_title (title={}, uri={}): {}",
                        title, url, e
                    );
                }
                return Err(e); // bail out
            }
        };
        #[cfg(debug_assertions)]
        {
            println!(
                "> SELECT: conneciton opened, url: '{}', title: '{}'",
                url, title
            );
        }

        let manga_iter_result = stmt.query_map(&[&title, &url], |row| {
            Ok(MangaModel {
                id: row.get(0)?,
                title: row.get(1)?,
                title_romanized: row.get(2)?,
                url: row.get(3)?,
                url_with_chapter: row.get(4)?,
                chapter: row.get(5)?,
                last_update: row.get(6)?,
                notes: row.get(7)?,
                tags: match row.get::<usize, String>(9) {
                    Ok(t) => t.split(",").map(|s| s.to_string()).collect(),
                    Err(_) => Vec::new(),
                },
                my_anime_list: row.get(9)?,
            })
        }); // don't let it throw rusqulite::Error if any

        let manga_iter = match manga_iter_result {
            Ok(m) => {
                #[cfg(debug_assertions)]
                {
                    println!(">> SELECT: query_map succeeded");
                }
                m
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                {
                    println!(
                        "ERROR: select_manga_from_url_and_title (title={}, uri={}): {}",
                        title, url, e
                    );
                }
                return Err(e);
            }
        };

        // if at least one row found, return Ok
        let mut rows = Vec::new();
        for manga in manga_iter {
            match manga {
                Ok(m) => {
                    #[cfg(debug_assertions)]
                    {
                        println!(
                            ">>> SELECT: select_manga_from_url_and_title: found: {:?}",
                            m
                        );
                    }
                    rows.push(m)
                }
                Err(e) => {
                    #[cfg(debug_assertions)]
                    {
                        println!("select_manga_from_url_and_title: error: {}", e);
                    }
                    return Err(e);
                }
            }
        }
        #[cfg(debug_assertions)]
        {
            println!(
                ">>>> SELECT: select_manga_from_url_and_title: rows collected: {:?}",
                rows.len()
            );
        }

        if rows.len() > 0 {
            return Ok(rows);
        }

        // if here, succeeded because of no row-sets, just let the caller know there are no title+url match
        Err(rusqlite::Error::QueryReturnedNoRows)
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
        let mut stmt = conn.prepare(
            "SELECT manga.id, manga.title, manga.title_romanized, manga.url, manga.url_with_chapter, manga.chapter, manga.last_update, manga.notes, GROUP_CONCAT(tags.tag, ',') AS tags, manga.my_anime_list FROM manga_to_tags_map INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id WHERE manga.id = ?1 GROUP BY manga.id")?;

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
            //"SELECT manga.id, manga.title, manga.title_romanized,               \
            //            manga.url, manga.url_with_chapter, manga.chapter,           \
            //            manga.last_update, manga.notes,                             \
            //            GROUP_CONCAT(tags.tag, ',') AS tags,                        \
            //                manga.my_anime_list FROM manga_to_tags_map              \
            //                INNER JOIN manga ON manga_to_tags_map.manga_id = manga.id   \
            //                INNER JOIN tags ON manga_to_tags_map.tag_id = tags.id       \
            //                GROUP BY manga.id",
            "SELECT * FROM manga",
        )?;

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
                    WHERE manga.title LIKE '%' || ?1 || '%' ",
        )?;
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
                    WHERE manga.title_romanized LIKE '%' || ?1 || '%' ",
        )?;
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
                    WHERE manga.url LIKE '%' || ?1 || '%' ",
        )?;
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
                    WHERE manga.url_with_chapter LIKE '%' || ?1 || '%' ",
        )?;
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

    // unit-test modules
    #[cfg(test)]
    mod tests {
        use crate::model_manga::{self, model_manga::MangaModel};

        use super::fetch_manga_data;
        const K_MANGA_TITLE: &str = "ゲート―自衛隊彼の地にて、斯く戦えり";
        const K_MANGA_URL_WITH_CHAPTERS: &str = "https://example.com/manga/gate-chapter-10/";

        fn make_samplel_row() -> MangaModel {
            let model = MangaModel::new_from_required_elements(
                K_MANGA_TITLE.to_string(),
                K_MANGA_URL_WITH_CHAPTERS.to_string(),
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
            let mut manga_no_id = make_samplel_row();
            let manga_inserted = super::insert_manga(db_full_paths, &manga_no_id).unwrap();

            // now try inserting again, with same title+url (should fail due to unique constraints)
            let manga_inserted_wrapped = super::insert_manga(db_full_paths, &manga_inserted);
            //assert!(manga_inserted_wrapped.is_err());

            // this time, call upsert instead
            let manga_upserted = super::upsert_manga(db_full_paths, &manga_no_id); // should succeed either way, for it will attempt to "update" if exists
            assert!(manga_upserted.is_ok());

            // us search println!("\nsearching manga for all wildcards");
            match super::fetch_manga_data2(db_full_paths, "%", "%") {
                Ok(manga_vec) => {
                    println!("> Found {} rows", manga_vec.len());
                    for (manga) in manga_vec {
                        println!("manga: {:?}", manga);
                    }
                }
                Err(e) => {
                    println!("ERROR: test_insert_manga: search_manga: {}", e);
                    assert!(false);
                }
            };

            // dump the table
            let manga_vec = super::select_all_manga(db_full_paths).unwrap();
            assert_eq!(manga_vec.len(), 1);
            for m in manga_vec {
                assert_eq!(manga_inserted.id, m.id);
                assert_eq!(manga_inserted.title, m.title);
                assert_eq!(manga_inserted.title_romanized, m.title_romanized);
                assert_eq!(manga_inserted.url, m.url);
                assert_eq!(manga_inserted.url_with_chapter, m.url_with_chapter);
                assert_eq!(manga_inserted.chapter, m.chapter);
                assert_eq!(manga_inserted.last_update, m.last_update);
                assert_eq!(manga_inserted.notes, m.notes);
                assert_eq!(manga_inserted.tags, m.tags);
                assert_eq!(manga_inserted.my_anime_list, m.my_anime_list);
            }

            // clean up
            std::fs::remove_file(db_full_paths).unwrap();
        }

        #[test]
        fn test_fetch_manga_data() {
            let db_file_path = "tests/test_fetch_manga_test.sqlite3"; // Replace with your actual database file path
            let title = "%"; // Replace with the title you want to search for
            let url = "%"; // Replace with the URL you want to search for

            match super::fetch_manga_data2(db_file_path, title, url) {
                Ok(manga_data) => {
                    for manga in manga_data {
                        println!("{}|{}|{}", manga.id, manga.title, manga.url);
                    }
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                }
            }
        }
    }
}
