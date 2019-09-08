# sqlite2dir

`sqlite2dir` exposes the contents of an SQLite 3 database as a
collection of plain-text files. It's intended use case is not for
database backups -- the view provided is intended to allow humans to
more easily inspect and track changes to an SQLite database. To that
end, `sqlite2dir` also supports committing the resulting tree of files
to a bare git repository, which allows inspecting the history of
changes using regular git tools.

Note that `sqlite2dir` is currently in its initial development phase,
and hasn't even been deployed by its author. The usual caveats apply.

## Installation

As `sqlite2dir` is written in Rust, you need a [Rust toolchain]. Rust
1.37 or newer is required. Once you have that, you can run it directly
from the source checkout:

```sh
cargo run -- --help
cargo run -- db.sqlite3 db-contents
```

To install, use `cargo install --path .` from the source checkout,
which will end up installing the executable in
`~/.cargo/bin/sqlite2dir`, which should be in your `PATH` environment
variable, if you followed the Rust toolchain installations
instructions.

`sqlite2dir` is *not* yet published on [`crates.io`]. Once that happens,
you will be able to install the latest release using:

```sh
cargo install sqlite2dir
```

### Static build

For deployment to a Linux target, an attractive option is to create a
statically linked binary using Rust's MUSL target. This will result in
a completely standalone binary, which depends only on the Linux
kernel's system call ABI.

In this case, you need to enable the `vendored-sqlite` feature flag to
link against an embedded, newly-compiled, copy of `libsqlite3`:

```sh
# If you haven't installed the MUSL target already, let's do that now":
rustup target add x86_64-unknown-linux-musl
# Build using a compiled-in copy of libsqlite3
cargo build --target x86_64-unknown-linux-musl --features vendored-sqlite --release
# Let's check it's really a static binary
file file target/x86_64-unknown-linux-musl/release/sqlite2dir | grep -q 'statically linked' || echo "nope"
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
perceived importance.

- [X] **Only commit when the tree actually changed**.
- [ ] Create the bare git repository if it doesn't exist.
- [ ] Pull email and username from the git config, if not given on the
      command line. Add a `--git` switch to enforce git operation the
      absence of other `--git-...` options.
- [X] Add options `--git-diff` and `--git-diff-exit-code`, which can
      be used for monitoring purposes.
- [ ] Write a proper manual page.
- [ ] A test harness including some basic smoke tests.
- [ ] Support for the SQLite "blob" data type. A basic implementation
      would be to hash the blob content, and spit it out disk as its
      own file. The DB column would then contain a reference like
      `{"blob-sha3-256": "SHA-3-here"}`. An improvement would be to
      base64-encode small blobs, and store them inline.

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
