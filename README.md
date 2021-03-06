# sqlite2dir

`sqlite2dir` exposes the contents of an SQLite 3 database as a
collection of plain-text files. It's intended use case is not for
database backups -- the view provided is intended to allow humans to
more easily inspect and track changes to an SQLite database. The
output format is chosen so that tools designed to operate on
plain-text files, like `diff` and `git` should work well.

To allow for change tracking, `sqlite2dir` supports committing the
tree of files resulting from the database export directly to a bare
git repository, which allows inspecting the history of changes using
regular git tools.

Note that `sqlite2dir` is currently in its initial development phase,
and hasn't even been deployed by its author. The usual caveats apply.

## Documentation

The documentation for `sqlite2dir` comes in the form of [man
page](./sqlite2dir.1.md). The markdown file can be turned in to troff
format for viewing with the `man` command using [pandoc]. Note that to
the markdown source is tailored toward producing good output when fed
through pandoc, and will not be rendered nicely on github or alike,
and is not ideal to read in plain, either.

Generate and view the man page using the Unix `man` command:

```sh
pandoc -s -t man sqlite2dir.1.md -o sqlite2dir.1
man -l sqlite2dir.1
```

You can also find a pandoc HTML rendering of the manpage
[online](https://r0tty.org/software/sqlite2dir.1.html).

## Installation

As `sqlite2dir` is written in Rust, you need a [Rust toolchain]. Rust
1.37 or newer is required. To obtain the latest release from
[crates.io], use:

```sh
cargo install sqlite2dir
```

Alternatively, you can run it directly from the source checkout:

```sh
cargo run -- --help
cargo run -- db.sqlite3 db-contents
```

To install from locally checked-out source, use `cargo install --path
.`, which will end up installing the executable in
`~/.cargo/bin/sqlite2dir`, which should already be in your `PATH`
environment variable, if you followed the Rust toolchain installations
instructions.

### Static build

For deployment to a Linux target, an attractive option is to create a
statically linked binary using Rust's MUSL target. This will result in
a completely standalone binary, which depends only on the Linux
kernel's system call ABI.

In this case, you need to enable the `vendored-sqlite` feature flag to
link against an embedded, newly-compiled, copy of `libsqlite3`:

```sh
# If you haven't installed the MUSL target already, let's do that now:
rustup target add x86_64-unknown-linux-musl
# Build using a compiled-in copy of libsqlite3
cargo build --target x86_64-unknown-linux-musl --features vendored-sqlite --release
# Let's check it's really a static binary
file target/x86_64-unknown-linux-musl/release/sqlite2dir \
  | grep -q 'statically linked' || echo "nope"
```

## Usage

Create a dump of an sqlite3 database to a directory:

```sh
sqlite2dir db.sqlite3 db-contents
```

Inside the newly created `db-contents` directory, you will find a
collection of SQL files containing the database Schema, and a JSON
file per table with the table contents.

The format of the SQL table data files is a stream of JSON arrays,
each row being a single line containing a stand-alone JSON array
containing the column data for a single database row. This format has
been chosen to fulfill the following criteria:

- Reasonable diff output, while preserving the type of the values. In
  particular, NULL values are represented as JSON `null`, and so can
  be disambiguated from a "NULL" string or an empty string.
- Allow streaming creation and consumption with JSON parsers and
  serializers that operate on whole values.

Note that the SQLite "blob" data type is not yet supported, and the
database dump will be aborted if a blob is encountered. See "Planned
features" below for details.

## Planned features

These features are planned, roughly in the order of the author's
perceived importance. During development, items will be moved from
below into the [changelog](./NEWS.md) upon completion.

- An option to generate a short report, suitable as an email message
  body.
- A test harness including some basic smoke tests.
- Support for the SQLite "blob" data type. A basic implementation
  would be to hash the blob content, and spit it out disk as its own
  file. The DB column would then contain a reference like
  `{"blob-sha3-256": "SHA-3-here"}`. An improvement would be to
  base64-encode small blobs, and store them inline.

## Possible future features

- Add support for a `--run` argument, to specify a config file
   allowing for multiple DB extractions in a single run.
- With `--run`, add possibility for multi-threaded operation.
- Additional database backends. I don't anticipate having the need for
  this feature, so I probably won't add it myself. Pull requests
  welcome!

## Non-features

`sqlite2dir` is not, is not intended to be, and will, in all
likelihood, never become a database backup tool. SQLite provides the
`.dump` and `.backup` meta-commands its command-line tool, these
should be used instead. That way, it is even possible to restore the
data!

## Example use case

This is the scenario which prompted the development of `sqlite2dir`.

The [PowerDNS] (aka `pdns`) authoritative nameserver
provides several database backend, in addition to the "bind" backend,
which operates on plain-text zone files. The use of a database backend
is more flexible, but prevents easily tracking changes to the zone
content. When using plain text zone files, change tracking is easily
achieved by just putting the zone files into a git repository. Using
`sqlite2dir`, you can recover that functionality when using the SQLite
pdns backend.

The following command will extract the database and commit to a bare
git repository:

```sh
sqlite2dir --git-name="Clara Root" --git-email="root@localhost" \
    /var/lib/pdns/pdns.sqlite3 /var/lib/pdns/pdns.git
```

By adding a periodic job executing the above command, e.g., via `cron`
or `systemd` timers, one can accumulate history in a bare git
repository, which can be cloned and inspected for troubleshooting or
other analysis.

## Licensing

The code and documentation in the `sqlite2dir` crate is [free
software](https://www.gnu.org/philosophy/free-sw.html), licensed under
the [GNU GPL](./LICENSE), version 3.0 or later, at your option.

[Rust toolchain]: https://www.rust-lang.org/tools/install
[PowerDNS]: https://www.powerdns.com/
[crates.io]: https://crates.io/
[pandoc]: https://pandoc.org/
