use std::{io, path::PathBuf};

use failure::{format_err, ResultExt};
use structopt::StructOpt;

use sqlite2dir::{Db, DirSink, GitRepo, Sink, TableSink};

// These seem to be missing from the `git2` bindings.
const DIFF_LINE_CONTEXT: char = ' ';
const DIFF_LINE_ADDITION: char = '+';
const DIFF_LINE_DELETION: char = '-';

/// Create a dump of an SQLite database to a directory.
#[derive(StructOpt)]
struct Opt {
    /// SQLite database to read from.
    db_filename: String,
    /// Output directory or git bare repository.
    output_dir: PathBuf,
    /// Use git.
    #[structopt(long = "git")]
    git: bool,
    /// Author name to use for git commits.
    #[structopt(long = "git-name")]
    git_name: Option<String>,
    /// Author email to use for git commits.
    #[structopt(long = "git-email")]
    git_email: Option<String>,
    /// Commit message for git commits.
    ///
    /// If not given, a default message will be used.
    #[structopt(long = "git-message")]
    git_message: Option<String>,
    /// Show a diff when something changed.
    #[structopt(long = "git-diff")]
    git_diff: bool,
    /// Exit with status 1 when there were changes.
    #[structopt(long = "git-diff-exit-code")]
    git_diff_exit_code: bool,
}

impl Opt {
    fn git_authored(&self) -> Result<Option<git2::Signature>, git2::Error> {
        match (&self.git_name, &self.git_email) {
            (Some(name), Some(email)) => Ok(Some(git2::Signature::now(name, email)?)),
            _ => Ok(None),
        }
    }
    fn git_message(&self) -> &str {
        self.git_message
            .as_ref()
            .map(|msg| msg.as_str())
            .unwrap_or("sqlite2dir auto-commit")
    }
    fn use_git(&self) -> bool {
        self.git
            || self.git_diff
            || self.git_diff_exit_code
            || self.git_email.is_some()
            || self.git_name.is_some()
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

fn show_diff_line(mut writer: impl io::Write, line: &git2::DiffLine) -> io::Result<()> {
    match line.origin() {
        DIFF_LINE_ADDITION | DIFF_LINE_CONTEXT | DIFF_LINE_DELETION => {
            let mut buf = [0; 4];
            let origin = line.origin().encode_utf8(&mut buf);
            writer.write_all(origin.as_bytes())?;
            writer.write_all(line.content())?;
        }
        _ => writer.write_all(line.content())?,
    }
    Ok(())
}

fn run(opt: &Opt) -> Result<i32, failure::Error> {
    let db = Db::open(&opt.db_filename)?;
    match GitRepo::open(&opt.output_dir) {
        Ok(repo) => {
            let authored = opt.git_authored()?.ok_or_else(|| {
                format_err!("git target detected, but required options not provided")
            })?;
            let mut tree = repo.tree()?;
            fill_sink(&mut tree, &db)?;
            let diff = repo.commit(opt.git_message(), &authored, tree)?;
            if opt.git_diff {
                let stdout = io::stdout();
                let stdout = stdout.lock();
                let mut writer = io::BufWriter::new(stdout);
                diff.print(
                    git2::DiffFormat::Patch,
                    |_delta, _hunk, line| match show_diff_line(&mut writer, &line) {
                        Ok(_) => true,
                        Err(e) => {
                            eprintln!("I/O error while showing diff: {}", e);
                            false
                        }
                    },
                )?;
            }
            let rc = if opt.git_diff_exit_code {
                if diff.deltas().len() > 0 {
                    1
                } else {
                    0
                }
            } else {
                0
            };
            Ok(rc)
        }
        Err(e) => {
            if opt.use_git() {
                return Err(format_err!(
                    "could not open destination as bare git repository: {}",
                    e
                ));
            }
            let mut sink = DirSink::open(&opt.output_dir)?;
            fill_sink(&mut sink, &db)?;
            Ok(0)
        }
    }
}

fn main() -> Result<(), failure::Error> {
    let opt = Opt::from_args();
    let rc = match run(&opt) {
        Ok(rc) => rc,
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    };
    std::process::exit(rc);
}
