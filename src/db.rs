use chrono;
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Note {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub date: String,
}

const CREATE_TABLE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS notes (
        id      INTEGER PRIMARY KEY,
        title   TEXT NOT NULL,
        content TEXT NOT NULL,
        date    TEXT NOT NULL
    )";

const SELECT_ALL_SQL: &str = "SELECT id, title, content, date FROM notes ORDER BY id DESC";
const SELECT_BY_ID_SQL: &str = "SELECT id, title, content, date FROM notes WHERE id = ?1";

pub fn setup_db() -> Result<Connection> {
    let mut data_dir = dirs::data_dir().unwrap_or_else(|| PathBuf::from("./"));
    data_dir.push("hyprnotes");

    fs::create_dir_all(&data_dir).expect("Could not create data directory");
    data_dir.push("notes.db");

    let conn = Connection::open(data_dir)?;
    conn.execute(CREATE_TABLE_SQL, [])?;

    Ok(conn)
}

pub fn insert_note(conn: &Connection, title: &str, content: &str) -> Result<()> {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO notes (title, content, date) VALUES (?1, ?2, ?3)",
        params![title, content, now],
    )?;
    Ok(())
}

pub fn get_all_notes(conn: &Connection) -> Result<Vec<Note>> {
    let mut stmt = conn.prepare(SELECT_ALL_SQL)?;
    let note_iter = stmt.query_map([], map_row_to_note)?;

    note_iter.collect()
}

pub fn get_note_by_id(conn: &Connection, id: i32) -> Result<Note> {
    conn.query_row(SELECT_BY_ID_SQL, [id], map_row_to_note)
}

pub fn update_note(conn: &Connection, id: i32, title: &str, content: &str) -> Result<()> {
    conn.execute(
        "UPDATE notes SET title = ?1, content = ?2 WHERE id = ?3",
        params![title, content, id],
    )?;
    Ok(())
}

pub fn delete_note(conn: &Connection, id: i32) -> Result<()> {
    conn.execute("DELETE FROM notes WHERE id = ?1", [id])?;
    Ok(())
}

fn map_row_to_note(row: &rusqlite::Row) -> Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        date: row.get(3)?,
    })
}