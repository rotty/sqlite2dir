% sqlite2dir(1) sqlite2dir User Manual
% Andreas Rottmann
% September 8, 2019

# NAME

sqlite2dir - Dump the contents of an SQLite database to a directory

# SYNPOSIS

sqlite2dir [*options*] *sqlite-db-file* *output-directory*

# DESCRIPTION

`sqlite2dir` exposes the contents of an SQLite 3 database as a
collection of plain-text files. It's intended use case is not for
database backups -- the view provided is intended to allow humans to
more easily inspect and track changes to an SQLite database. To that
end, `sqlite2dir` also supports committing the resulting tree of files
to a bare git repository, which allows inspecting the history of
changes using regular git tools.

When *output-directory* already exists, and it contains a git bare
repository, the git support is enabled, and a new commit will be added
to the repository when the database content changed from the
repository `HEAD` commit. The commit metadata can be influenced by the
various `--git-...` options, which are documented below. When a bare
git repository is detected as destination, `sqlite2dir` will refuse to
operate unless `--git-name` and `--git-email` are given; `sqlite2dir`
will currently not consulting the user's git configuration for these
values.

Note that `sqlite2dir` uses `libgit2` for its git support, not the
`git` command-line executable. This mean that its resource profile
should be very lightweight, making it realistic to run it very
frequently with minimal impact to system load, at least for small
databases. Also, as might be expected, `git` doesn't need to be
installed to make use of the git support.

# OPTIONS

\--git
: Refuse operation if the destination is not a bare git repository.

\--git-diff
: When committing a change, show a diff of the changes on
  stdout. Implies `--git`.

\--git-diff-exit-code
: After committing a change, exit with status 1. This is useful for
  reacting to this condition, for example from shell scripts. Implies
  `--git`.

\--git-message=*message*
: Commit message for git commits. If not given, a default message will
  be used. Implies `--git`.

\--git-name=*name*
: Author name to use for git commits. Implies `--git`.

\--git-email=*git-email*
: Author email to use for git commits. Implies `--git`.

-h, \--help
: Show usage message.

-V, \--version
: Prints version information.

# SEE ALSO

`sqlite3`(1).
