use failure::ResultExt;

use sqlite2dir::{Db, DirSink, Sink, TableSink};

fn run() -> Result<(), failure::Error> {
    let db = Db::open("test.db")?;
    let schema = db
        .read_schema()
        .with_context(|e| format!("could not read schema: {}", e))?;
    let mut sink = DirSink::open("out")?;
    sink.write_schema(&schema)?;
    for entry in &schema {
        if entry.kind == "table" {
            let mut table = sink.open_table(&entry.tbl_name)?;
            let mut stmt = db.read_table(entry)?;
            let mut rows = stmt.query()?;
            while let Some(row) = rows.next()? {
                table.write_row(row).with_context(|e| {
                    format!("while writing row of table {}: {}", entry.tbl_name, e)
                })?;
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), failure::Error> {
    let rc = match run() {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    };
    std::process::exit(rc);
}
