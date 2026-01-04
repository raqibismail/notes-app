use rusqlite::{Connection, Result};
use std::fs;
use std::path::PathBuf;

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
    // We use 'TEXT' for titles and content.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id      INTEGER PRIMARY KEY,
            title   TEXT NOT NULL,
            content TEXT NOT NULL,
            date    TEXT NOT NULL
        )",
        (), // empty list of parameters
    )?;

    Ok(conn)
}