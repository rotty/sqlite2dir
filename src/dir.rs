use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::{util::write_json_row, SchemaEntry, Sink, TableSink};

#[derive(Debug)]
pub struct DirSink {
    path: PathBuf,
}

impl DirSink {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
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

    fn write_schema_entry(&mut self, entry: &SchemaEntry) -> io::Result<()> {
        if let Some(sql) = &entry.sql {
            let mut file = self.open_file(format!("schema/{}/{}.sql", entry.kind, entry.name))?;
            file.write_all(sql.as_bytes())?;
        }
        Ok(())
    }
    fn open_table(&mut self, name: impl AsRef<str>) -> io::Result<FileTable> {
        Ok(FileTable {
            file: self.open_file(format!("data/table/{}.json", name.as_ref()))?,
        })
    }
    fn close_table(&mut self, _table: FileTable) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct FileTable {
    file: File,
}

impl TableSink for FileTable {
    fn write_row(&mut self, row: &rusqlite::Row) -> io::Result<()> {
        write_json_row(&mut self.file, row)
    }
}
