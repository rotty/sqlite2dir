% sqlite2dir(1) sqlite2dir User Manual
% Andreas Rottmann
% September 12, 2019

# NAME

sqlite2dir - Dump the contents of an SQLite database to a directory

# SYNPOSIS

sqlite2dir [*options*] *sqlite-db-file* *output-directory*

# DESCRIPTION

__sqlite2dir__ exposes the contents of an SQLite 3 database as a
collection of plain-text files. Its intended use case is not for
database backups -- the view provided is intended to allow humans to
more easily inspect and track changes to an SQLite database. To that
end, __sqlite2dir__ also supports committing the resulting tree of
files to a bare git repository, which allows inspecting the history of
changes using regular git tools.

Normally, *output-directory* must not already exist, and is freshly
created by __sqlite2dir__. This is not the case when the git mode is
used, which can be enabled by any of the __\--git-...__ options, and
is automatically enabled when *output-directory* is identified as bare
git repository. Git mode is described in more detail in the section
"__GIT MODE__".

# OPTIONS

\--git
: Refuse operation if the destination is not a bare git repository.

\--git-diff
: When committing a change, show a diff of the changes on
  stdout. Implies __\--git__.

\--git-diff-exit-code
: After successfully committing a change, exit with status 1, in case
  of error, with status 2. Useful if __sqlite2dir__ is invoked
  programmatically. This is the same convention as used by
  diff(1). Implies __\--git__.

\--git-message=*message*
: Commit message for git commits. If not given, a default message will
  be used. Implies __\--git__.

\--git-name=*name*
: Author name to use for git commits. Implies __\--git__.

\--git-email=*git-email*
: Author email to use for git commits. Implies __\--git__.

-h, \--help
: Show usage message.

-V, \--version
: Prints version information.

# GIT MODE

When *output-directory* already exists, and it contains a git bare
repository, the git support is enabled, and a new commit will be added
to the repository when the database content changed from the
repository `HEAD` commit. The commit metadata can be influenced by the
various __\--git-...__ options, which are documented below. When a bare
git repository is detected as destination, __sqlite2dir__ will refuse
to operate unless __\--git-name__ and __\--git-email__ are given;
__sqlite2dir__ will currently not consulting the user's git
configuration for these values.

When *output-directory* does not exist, and any of the __\--git-...__
options is specified, a new bare git repository is created with the
given directory. The directory name given is taken literally, no
".git" is appended if it is missing.

Note that __sqlite2dir__ uses `libgit2` for its git support, not the
__git__ command-line executable. This mean that its resource profile
should be very lightweight, making it realistic to run it very
frequently with minimal impact to system load, at least for small
databases. Also, as might be expected, `git` doesn't need to be
installed to make use of the git support.

# OUTPUT FORMAT

__sqlite2dir__ produces a tree of files based on the content of the
database. The structure and contents of the directory tree are chosen
such that no exported information is lost or mangled, and such that
tools designed to operate on plain-text files, like diff(1) or git(1)
work reasonably well.

The format of the __sqlite2dir__ output is informally described below,
but be aware that this format does not yet come with any stability
guarantees. However, there will be an attempt to keep the format
relatively stable, and not change the format on patch version
bumps. For example, any 0.1.x release of __sqlite2dir__ should produce
the same output format.

The directories produced are:

`schema/table`
: For each table, stores a file with the SQL schema definition for
  that table.

`schema/index`
: Analogous to `schema/table`, stores the index SQL schema
  definitions, one file per index.

`data/table`
: For each table, stores a file containing the table contents, one row
  per line. Each line contains a JSON array, where the SQLite data
  types are represented like this:

  - SQL `NULL` is mapped to JSON `null` -- no surprises here.

  - Integers and reals are mapped to JSON numbers. The serialization
    mechanism used ensures faithful integer representation, but does
    not guarantee roundtrip-ability for reals.

  - Text is stored is mapped to JSON strings.

  - Blobs are currently not supported. This is considered a bug.

# EXIT STATUS

When __\--git-diff-exit-code__ is not specified, __sqlite2dir__ will exit
with status 0 if the operation succeeded, and with exit status 1 when
an error happened, such as an I/O or database error.

With the __\--git-diff-exit-code__ option, exit status 1 indicates
changes were successfully committed, while 2 is used to indicate
failure, instead of 1. A zero exit status in this case indicates that
the database exported is unchanged, compared to the last commit.

# SEE ALSO

sqlite3(1).
