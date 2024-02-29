use std::{io, path::PathBuf};

use anyhow::{format_err, Context as _};
use once_cell::unsync::Lazy;
use structopt::StructOpt;

use sqlite2dir::{dir, git, Db, Sink, TableSink};

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

// This is probably a bit overengineered, but this way, we only access the confg
// if needed.
//
// Future TODO: with named existential types (see
// <https://github.com/rust-lang/rfcs/pull/2515>), this could be nicely
// abstracted away into the `git` module. The issue here is that I don't see how
// to get a good API withou being able to assign a publicly namable type to the
// `F` instance that refers to the closure loading the config.
type LazyConfig<F> = Lazy<Result<git2::Config, git::Error>, F>;

fn config_fallback<T, F>(
    value: Option<T>,
    cfg: &LazyConfig<F>,
    name: &str,
) -> anyhow::Result<String>
where
    F: FnOnce() -> Result<git2::Config, git::Error>,
    T: Into<String>,
{
    value.map(Into::into).map_or_else(
        || match Lazy::force(cfg) {
            Ok(cfg) => Ok(cfg.get_string(name).map_err(Into::<git::Error>::into)?),
            Err(e) => Err(format_err!(r#"could not load git config: {}"#, e)),
        },
        Ok,
    )
}

impl Opt {
    fn git_name(&self) -> Option<&str> {
        self.git_name.as_deref()
    }
    fn git_email(&self) -> Option<&str> {
        self.git_email.as_deref()
    }
    fn git_authored(&self, repo: &git::Repo) -> anyhow::Result<git2::Signature> {
        let config = Lazy::new(move || -> Result<git2::Config, git::Error> {
            Ok(repo.config()?.snapshot()?)
        });
        let name = config_fallback(self.git_name(), &config, "user.name");
        let email = config_fallback(self.git_email(), &config, "user.email");

        match (name, email) {
            (Ok(name), Ok(email)) => Ok(git2::Signature::now(&name, &email)?),
            (name, email) => Err(format_err!(
                "git authorship information missing from command-line and git configuration:\n    {}",
                name.err()
                    .iter()
                    .chain(email.err().iter())
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join("\n    ")
            )),
        }
    }
    fn git_message(&self) -> &str {
        self.git_message
            .as_deref()
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

fn fill_sink(sink: &mut impl Sink, db: &mut Db) -> anyhow::Result<()> {
    let tx = db.transaction()?;
    let schema = tx.read_schema().context("unable to read schema")?;
    for entry in &schema {
        sink.write_schema_entry(entry)?;
        if entry.kind == "table" {
            let mut table = sink.open_table(&entry.tbl_name)?;
            let mut stmt = tx.read_table(entry)?;
            let column_count = stmt.column_count();
            let mut rows = stmt.query()?;
            while let Some(row) = rows.next()? {
                table
                    .write_row(row, column_count)
                    .with_context(|| format!("error writing row of table {}", entry.tbl_name))?;
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

fn run_with_git(
    db: &mut Db,
    repo: &git::Repo,
    authored: &git2::Signature,
    opt: &Opt,
) -> anyhow::Result<i32> {
    let mut tree = repo.tree()?;
    fill_sink(&mut tree, db)?;
    let diff = repo.commit(opt.git_message(), authored, tree)?;
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
    let rc = if opt.git_diff_exit_code && diff.deltas().len() > 0 {
        1
    } else {
        0
    };
    Ok(rc)
}

fn run(opt: &Opt) -> anyhow::Result<i32> {
    let mut db = Db::open(&opt.db_filename)?;
    match git::Repo::open(&opt.output_dir) {
        Ok(repo) => {
            let authored = opt.git_authored(&repo)?;
            run_with_git(&mut db, &repo, &authored, opt)
        }
        Err(e) => {
            if opt.use_git() {
                if let Some(git2::ErrorCode::NotFound) = e.code() {
                    let repo = git::Repo::create(&opt.output_dir)?;
                    let authored = opt.git_authored(&repo)?;
                    run_with_git(&mut db, &repo, &authored, opt)
                } else {
                    Err(format_err!(
                        "could not open destination as bare git repository: {}",
                        e
                    ))
                }
            } else {
                let mut sink = dir::Sink::open(&opt.output_dir)?;
                fill_sink(&mut sink, &mut db)?;
                Ok(0)
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    let rc = match run(&opt) {
        Ok(rc) => rc,
        Err(e) => {
            for (i, e) in e.chain().enumerate() {
                if i == 0 {
                    eprintln!("{}", e);
                } else {
                    eprintln!("caused by: {}", e);
                }
            }
            if opt.git_diff_exit_code {
                2
            } else {
                1
            }
        }
    };
    std::process::exit(rc);
}
