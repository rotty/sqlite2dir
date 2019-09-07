use std::{fmt, io};

use serde_json as json;

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
    fn write_schema(&mut self, entries: &[SchemaEntry]) -> io::Result<()>;
    fn open_table(&mut self, name: impl AsRef<str>) -> io::Result<Self::TableSink>;
}

pub trait TableSink {
    fn write_row(&mut self, row: &rusqlite::Row) -> io::Result<()>;
}

fn write_json_row(mut sink: impl io::Write, row: &rusqlite::Row) -> io::Result<()> {
    use rusqlite::types::ValueRef::*;
    // TODO: this could probably be made more efficient by using a
    // lower-level serialization interface.
    let values: Vec<_> = (0..row.column_count())
        .map(|i| match row.get_raw(i) {
            Null => Ok(json::Value::Null),
            Integer(n) => Ok(json::Value::from(n)),
            Real(n) => Ok(json::Value::from(n)),
            Text(bytes) => {
                let text = String::from_utf8(bytes.to_vec())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
                Ok(json::Value::String(text))
            }
            Blob(_) => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                UnsupportedInput::Blob,
            )),
        })
        .collect::<Result<_, io::Error>>()?;
    json::to_writer(&mut sink, &values)?;
    sink.write_all(b"\n")?;
    Ok(())
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

mod dir;
mod sqlite;

pub use dir::DirSink;
pub use sqlite::Db;
