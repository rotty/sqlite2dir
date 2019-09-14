# 0.2.0

The output format remains unchanged, the version bump is due to major
code changes and several new features.

General:

- There is now a manual page documenting `sqlite2dir`'s options,
  general operation, and output format. See the [README] for how to
  create troff input for the `man` command.

- All SQLite queries now run inside a single transaction.

New git-related features:

- The new `--git` switch allows enforcing git operation the absence of
  other `--git-...` options.

- When the destination does not exist, and git operation is requested,
  a bare git repository is now created instead of bailing out.

- The email address and username used for git commits are now read
  from the git configuration, if not given on the command line.

# 0.1.0

Initial release, including the basic functionality of dumping the data
both into a directory and into a bare git repository.

[README]: ./README.md
