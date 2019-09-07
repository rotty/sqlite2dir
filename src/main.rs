use std::{
    fmt,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use rusqlite::{Connection, OpenFlags, Row, NO_PARAMS};
use serde_json as json;

#[derive(Debug)]
struct SchemaEntry {
    kind: String,
    name: String,
    tbl_name: String,
    column_names: Vec<String>,
    sql: Option<String>,
}

#[derive(Debug)]
struct Db {
    conn: rusqlite::Connection,
}

impl Db {
    fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        Ok(Db { conn })
    }

    fn read_schema(&self) -> rusqlite::Result<Vec<SchemaEntry>> {
        let mut sqlite_master = self
            .conn
            .prepare("SELECT type, name, tbl_name, sql FROM sqlite_master")?;
        let schema_iter = sqlite_master.query_map(NO_PARAMS, |row| self.read_schema_entry(row))?;
        let entries = schema_iter.collect::<Result<_, _>>()?;
        Ok(entries)
    }

    fn read_schema_entry(&self, row: &Row) -> rusqlite::Result<SchemaEntry> {
        let name = row.get(1)?;
        let mut tbl_info = self.conn.prepare(&format!("PRAGMA table_info({})", name))?;
        let column_names = tbl_info
            .query_map(NO_PARAMS, |row| row.get(1))?
            .collect::<Result<_, _>>()?;
        Ok(SchemaEntry {
            kind: row.get(0)?,
            name,
            tbl_name: row.get(2)?,
            column_names,
            sql: row.get(3)?,
        })
    }

    fn read_table(&self, schema: &SchemaEntry) -> rusqlite::Result<TableReader> {
        Ok(TableReader {
            stmt: self
                .conn
                .prepare(&format!("SELECT * FROM {}", schema.name))?,
        })
    }
}

struct TableReader<'conn> {
    stmt: rusqlite::Statement<'conn>,
}

impl<'conn> TableReader<'conn> {
    fn query(&mut self) -> rusqlite::Result<rusqlite::Rows> {
        self.stmt.query(NO_PARAMS)
    }
}

trait Sink {
    type TableSink: TableSink;
    fn write_schema(&mut self, entries: &[SchemaEntry]) -> io::Result<()>;
    fn open_table(&mut self, name: impl AsRef<str>) -> io::Result<Self::TableSink>;
}

trait TableSink {
    fn write_row(&mut self, row: &Row) -> io::Result<()>;
}

struct DirSink {
    path: PathBuf,
}

impl DirSink {
    fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        // TODO: this should create, exclusively, and ideally obtain a handle to
        // the directory.
        Ok(DirSink {
            path: path.as_ref().to_owned(),
        })
    }

    fn open_file(&self, path: impl AsRef<Path>) -> io::Result<File> {
        let path = self.path.join(path);
        let base = path.parent().unwrap();
        fs::create_dir_all(base)?;
        File::create(path)
    }
}

impl Sink for DirSink {
    type TableSink = FileTable;

    fn write_schema(&mut self, entries: &[SchemaEntry]) -> io::Result<()> {
        for entry in entries {
            if let Some(sql) = &entry.sql {
                let mut file =
                    self.open_file(format!("schema/{}/{}.sql", entry.kind, entry.name))?;
                file.write_all(sql.as_bytes())?;
            }
        }
        Ok(())
    }
    fn open_table(&mut self, name: impl AsRef<str>) -> io::Result<FileTable> {
        Ok(FileTable {
            file: self.open_file(format!("data/table/{}.json", name.as_ref()))?,
        })
    }
}

struct FileTable {
    file: File,
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

impl TableSink for FileTable {
    fn write_row(&mut self, row: &Row) -> io::Result<()> {
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
                _ => Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    UnsupportedInput::Blob,
                )),
            })
            .collect::<Result<_, io::Error>>()?;
        json::to_writer(&mut self.file, &values)?;
        self.file.write_all(b"\n")?;
        Ok(())
    }
}

fn main() -> Result<(), failure::Error> {
    let db = Db::open("test.db")?;
    let schema = db.read_schema()?;
    let mut sink = DirSink::open("out")?;
    sink.write_schema(&schema)?;
    for entry in &schema {
        if entry.kind == "table" {
            let mut table = sink.open_table(&entry.tbl_name)?;
            let mut stmt = db.read_table(entry)?;
            let mut rows = stmt.query()?;
            while let Some(row) = rows.next()? {
                table.write_row(row)?;
            }
        }
    }
    Ok(())
}
