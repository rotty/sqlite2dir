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

`sqlite2dir` is *not* yet published on `crates.io`. Once that happens,
you will be able to install the latest release using:

```sh
cargo install sqlite2dir
```

[Rust toolchain]: https://www.rust-lang.org/tools/install

## Usage

Create a dump of an sqlite3 database to a directory:

```sh
sqlite2dir db.sqlite3 db-contents
```

Inside the newly created `db-contents` directory, you will find a
collection of SQL files containing the database Schema, and a JSON
file per table with the table contents.

## Example use case

This is the scenario which prompted the development of `sqlite2dir`.

The PowerDNS (aka `pdns`) authoritative nameserver
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
