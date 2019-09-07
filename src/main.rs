use std::path::PathBuf;

use failure::ResultExt;
use structopt::StructOpt;

use sqlite2dir::{Db, DirSink, Sink, TableSink};

#[derive(StructOpt)]
struct Opt {
    db_filename: String,
    output_dir: PathBuf,
}

fn run(opt: &Opt) -> Result<(), failure::Error> {
    let db = Db::open(&opt.db_filename)?;
    let schema = db
        .read_schema()
        .with_context(|e| format!("could not read schema: {}", e))?;
    let mut sink = DirSink::open(&opt.output_dir)?;
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
    let opt = Opt::from_args();
    let rc = match run(&opt) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    };
    std::process::exit(rc);
}
