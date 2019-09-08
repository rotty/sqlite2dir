use std::path::PathBuf;

use failure::{format_err, ResultExt};
use structopt::StructOpt;

use sqlite2dir::{Db, DirSink, GitRepo, Sink, TableSink};

#[derive(StructOpt)]
struct Opt {
    db_filename: String,
    output_dir: PathBuf,
    #[structopt(long("git-name"))]
    git_name: Option<String>,
    #[structopt(long("git-email"))]
    git_email: Option<String>,
}

impl Opt {
    fn git_authored(&self) -> Result<Option<git2::Signature>, git2::Error> {
        match (&self.git_name, &self.git_email) {
            (Some(name), Some(email)) => Ok(Some(git2::Signature::now(name, email)?)),
            _ => Ok(None),
        }
    }
}

fn fill_sink(sink: &mut impl Sink, db: &Db) -> Result<(), failure::Error> {
    let schema = db
        .read_schema()
        .with_context(|e| format!("could not read schema: {}", e))?;
    for entry in &schema {
        sink.write_schema_entry(&entry)?;
        if entry.kind == "table" {
            let mut table = sink.open_table(&entry.tbl_name)?;
            let mut stmt = db.read_table(entry)?;
            let mut rows = stmt.query()?;
            while let Some(row) = rows.next()? {
                table.write_row(row).with_context(|e| {
                    format!("while writing row of table {}: {}", entry.tbl_name, e)
                })?;
            }
            sink.close_table(table)?;
        }
    }
    sink.close()?;
    Ok(())
}

fn run(opt: &Opt) -> Result<(), failure::Error> {
    let db = Db::open(&opt.db_filename)?;
    match GitRepo::open(&opt.output_dir) {
        Ok(repo) => {
            let authored = opt.git_authored()?.ok_or_else(|| {
                format_err!("git target detected, but required options not provided")
            })?;
            let message = "sqlite2dir auto-commit";
            let mut tree = repo.tree()?;
            fill_sink(&mut tree, &db)?;
            repo.commit(message, &authored, tree)?;
        }
        Err(_) => {
            let mut sink = DirSink::open(&opt.output_dir)?;
            fill_sink(&mut sink, &db)?;
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
