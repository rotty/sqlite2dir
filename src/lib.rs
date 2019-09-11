use std::{fmt, io};

#[derive(Debug)]
pub struct SchemaEntry {
    pub kind: String,
    pub name: String,
    pub tbl_name: String,
    pub column_names: Vec<String>,
    pub sql: Option<String>,
}

pub trait Sink {
    type TableSink: TableSink;
    fn write_schema_entry(&mut self, entries: &SchemaEntry) -> io::Result<()>;
    fn close(&mut self) -> io::Result<()> {
        Ok(())
    }
    fn open_table(&mut self, name: impl AsRef<str>) -> io::Result<Self::TableSink>;
    fn close_table(&mut self, table: Self::TableSink) -> io::Result<()>;
}

pub trait TableSink {
    fn write_row(&mut self, row: &rusqlite::Row) -> io::Result<()>;
}

#[derive(Debug)]
enum UnsupportedInput {
    Blob,
}

impl fmt::Display for UnsupportedInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            UnsupportedInput::Blob => write!(f, "blobs not yet supported"),
        }
    }
}

impl std::error::Error for UnsupportedInput {}

pub mod dir;
pub mod git;
mod sqlite;
mod util;

pub use sqlite::Db;
