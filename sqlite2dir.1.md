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
various `--git-...` options, which are documented below.

# OPTIONS

\--git-diff
: When committing a change, show a diff of the changes on stdout.

\--git-diff-exit-code
: After committing a change, exit with status 1. This is useful for
  reacting to this condition, for example from shell scripts.

\--git-message=*message*
: Commit message for git commits. If not given, a default message will be used.

\--git-name=*name*
: Author name to use for git commits.

\--git-email=*git-email*
: Author email to use for git commits.

-h, \--help
: Show usage message.

-V, \--version
: Prints version information.

# SEE ALSO

`sqlite3`(1).
