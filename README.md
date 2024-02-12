# firefox_bookmark_to_sqlite3

Reads (via CLI) Firefox generated bookmarks (JSON created via export) and upserts (URL and title) into sqlite3 on Linux and Windows with data-model of:

```rust
    pub struct MangaModel {
        id: u32,       // primary key - either prune or ignore if id is 0
        title: String, // UTF8 encoded
        possible_title_romanized: Option<String>, // is Some() ONLY if title was in Japanese
        url: String,   // home page of manga (see impl of to_url and from_url validation)
        possible_url_with_chapter: Option<String>, // last read/updated chapter
        possible_chapter: Option<String>, // last read/updated chapter
        possible_last_update: Option<String>, // "YYYY-MM-DDTHH:mm:ss" (24hr)
        possible_last_update_millis: Option<i64>, // see chrono::NaiveDateTime
        possible_notes: Option<String>,
        tags: Vec<String>, // i.e. "#アニメ化" ; empty vec[] is same as None
        possible_my_anime_list: Option<String>, // provides author and artist
    }
```

The original intentions was to be able to have my own local database for lookup in which I can query from my laptop and desktop for (as you can see from my data-model) manga.  If I was on eBookJapan, because the books I buy are persistent on their (yahoo.co.jp) server, I've no worries.  It's the other manga sites in which, if Iupdate my bookmark to the latest chapter I've finished reading, that bookmark is ONLY on that host (i.e. my laptop).  Then if I had to switch over to my desktop, I would then have to export the bookmark from my laptop as JSON, then import it into my desktop, and so on (also on my Amazon Fire (Android) tablet which is the most ideal for reading tankoubon).  It's not too much of an issue if there are only a handful of links, but when it starts getting larger (i.e. free manga such as [ワンパンマン](http://galaxyheavyblow.web.fc2.com/)) as well as light-novels from [小説家になろう](https://syosetu.com/), I have started to get confused on what I've read last months...  So I wrote this to just export my bookmarks and have it upsert (update and/or insert) the URL and the title into sqlite3, in which I have a BASH script such as:

- get_recent.sh
- find_title.sh
- update_chapter.sh

Note that I just ssh to the main host that has the sqlite3 and copy-and-paste the URL to firefox (for get and find), and to the CLI (for update).  I've also considered HTML version to access the centralized host via apache/nginx but truth be told, I have terminal always open to ssh to that host anyway from ALL my devices that has keyboards (including my Android), so for me, I am fine with what I have.

## Side notes

As mentioned, the data-model is biased towards manga and light-novels, for I have only two purposes for browsers (GUI) on my devices that are non-work-related (either for manga/la-no-be reading or for tech-docs and coding assistance for references).  In any case, other libs I've used (directly in code and/or via my BASH) is `kakasi`;  There probably are few more (handful?) libs that can romanize hiragana/katakana/kanji, but this is my GOTO lib since the late-90's but it works on Debian (not too sure on Windows anymore, nor do I probably care) for decades back in C/C++ days as well as now in Rust.  My other project lenzu, I think I'm using it as well (along with tesseract) and on another project, along with mecab for TTS.  All in all, `kakasi` has never let me down...
