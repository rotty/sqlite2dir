use std::path::Path;

use rusqlite::{Connection, OpenFlags, Row, NO_PARAMS};

use crate::SchemaEntry;

#[derive(Debug)]
pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        Ok(Db { conn })
    }

    pub fn read_schema(&self) -> rusqlite::Result<Vec<SchemaEntry>> {
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

    pub fn read_table(&self, schema: &SchemaEntry) -> rusqlite::Result<TableReader> {
        Ok(TableReader {
            stmt: self
                .conn
                .prepare(&format!("SELECT * FROM {}", schema.name))?,
        })
    }
}

#[derive(Debug)]
pub struct TableReader<'conn> {
    stmt: rusqlite::Statement<'conn>,
}

impl<'conn> TableReader<'conn> {
    pub fn query(&mut self) -> rusqlite::Result<rusqlite::Rows> {
        self.stmt.query(NO_PARAMS)
    }
}
