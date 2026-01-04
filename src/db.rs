use rusqlite::{Connection, Result};
use std::fs;
use std::path::PathBuf;
use chrono;

#[derive(Debug)]
pub struct Note {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub date: String,
}

pub fn setup_db() -> Result<Connection> {
    // 1. Determine where to save the file (~/.local/share/hyprnotes/)
    let mut data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("./"));
    data_dir.push("hyprnotes");
    
    // 2. Create the directory if it doesn't exist
    fs::create_dir_all(&data_dir).expect("Could not create data directory");
    data_dir.push("notes.db");

    // 3. Connect (this creates the file if it's missing)
    let conn = Connection::open(data_dir)?;

    // 4. Create the table using SQL
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id      INTEGER PRIMARY KEY,
            title   TEXT NOT NULL,
            content TEXT NOT NULL,
            date    TEXT NOT NULL
        )",
        (),
    )?;

    Ok(conn)
}

pub fn insert_note(conn: &rusqlite::Connection, title: &str, content: &str) -> rusqlite::Result<()> {
    let current_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    
    conn.execute(
        "INSERT INTO notes (title, content, date) VALUES (?1, ?2, ?3)",
        (title, content, current_date),
    )?;
    Ok(())
}

pub fn get_all_notes(conn: &Connection) -> Result<Vec<Note>> {
    let mut stmt = conn.prepare("SELECT id, title, content, date FROM notes ORDER BY id DESC")?;
    
    let note_iter = stmt.query_map([], |row| {
        Ok(Note {
            id: row.get(0)?,
            title: row.get(1)?,
            content: row.get(2)?,
            date: row.get(3)?,
        })
    })?;

    let mut notes = Vec::new();
    for note in note_iter {
        notes.push(note?);
    }
    Ok(notes)
}