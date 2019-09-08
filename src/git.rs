use std::{collections::HashMap, io, path::Path};

use crate::{
    util::{other_io_error, write_json_row},
    SchemaEntry, Sink, TableSink,
};

// These seem to be missing from the `git2` bindings.
const FILEMODE_BLOB: i32 = 0o100644;
const FILEMODE_TREE: i32 = 0o040000;

pub struct GitRepo {
    repo: git2::Repository,
}

impl GitRepo {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, git2::Error> {
        Ok(GitRepo {
            repo: git2::Repository::open_bare(path)?,
        })
    }

    pub fn tree(&self) -> Result<TreeSink, git2::Error> {
        self.repo.treebuilder(None).map(|builder| TreeSink {
            repo: &self.repo,
            tree: builder,
            table_entries: Default::default(),
            schema_entries: Default::default(),
        })
    }

    pub fn commit(
        &self,
        message: &str,
        authored: &git2::Signature,
        sink: TreeSink,
    ) -> Result<git2::Diff, git2::Error> {
        let head = match self.repo.head() {
            Ok(head) => Some(head),
            Err(ref e) if e.code() == git2::ErrorCode::UnbornBranch => None,
            Err(e) => return Err(e),
        };
        let oid = sink.tree.write()?;
        let tree = self.repo.find_tree(oid)?;
        let old_tree = head.as_ref().map(|r| r.peel_to_tree()).transpose()?;
        let diff = self
            .repo
            .diff_tree_to_tree(old_tree.as_ref(), Some(&tree), None)?;
        if diff.deltas().len() > 0 {
            let parent_refs: Vec<_> = head.iter().collect();
            let parent_commits: Vec<git2::Commit<'_>> = parent_refs
                .iter()
                .map(|r| r.peel_to_commit())
                .collect::<Result<_, _>>()?;
            self.repo.commit(
                Some("HEAD"),
                &authored,
                &authored,
                message,
                &tree,
                &parent_commits.iter().collect::<Vec<_>>(),
            )?;
        }
        Ok(diff)
    }
}

pub struct TreeSink<'repo> {
    repo: &'repo git2::Repository,
    tree: git2::TreeBuilder<'repo>,
    table_entries: Vec<(String, git2::Oid)>,
    schema_entries: HashMap<String, Vec<(String, git2::Oid)>>,
}

impl<'repo> Sink for TreeSink<'repo> {
    type TableSink = GitTable;

    fn write_schema_entry(&mut self, entry: &SchemaEntry) -> io::Result<()> {
        if let Some(sql) = &entry.sql {
            let oid = self.repo.blob(sql.as_bytes()).map_err(other_io_error)?;
            self.schema_entries
                .entry(entry.kind.clone())
                .or_insert_with(Vec::new)
                .push((entry.name.clone(), oid));
        }
        Ok(())
    }

    fn close(&mut self) -> io::Result<()> {
        let mut schema_dir = self.repo.treebuilder(None).map_err(other_io_error)?;
        for (kind, entries) in self.schema_entries.drain() {
            let mut schema_kind = self.repo.treebuilder(None).map_err(other_io_error)?;
            for (name, oid) in entries {
                schema_kind
                    .insert(format!("{}.sql", name), oid, FILEMODE_BLOB)
                    .map_err(other_io_error)?;
            }
            schema_dir
                .insert(
                    kind,
                    schema_kind.write().map_err(other_io_error)?,
                    FILEMODE_TREE,
                )
                .map_err(other_io_error)?;
        }
        self.tree
            .insert(
                "schema",
                schema_dir.write().map_err(other_io_error)?,
                FILEMODE_TREE,
            )
            .map_err(other_io_error)?;
        let mut table_dir = self.repo.treebuilder(None).map_err(other_io_error)?;
        for (name, oid) in self.table_entries.drain(0..) {
            table_dir
                .insert(format!("{}.json", name), oid, FILEMODE_BLOB)
                .map_err(other_io_error)?;
        }
        self.tree
            .insert(
                "table",
                table_dir.write().map_err(other_io_error)?,
                FILEMODE_TREE,
            )
            .map_err(other_io_error)?;
        Ok(())
    }

    fn open_table(&mut self, name: impl AsRef<str>) -> io::Result<Self::TableSink> {
        Ok(GitTable {
            name: name.as_ref().to_owned(),
            content: Vec::new(),
        })
    }

    fn close_table(&mut self, table: GitTable) -> io::Result<()> {
        let oid = self.repo.blob(&table.content).map_err(other_io_error)?;
        self.table_entries.push((table.name.clone(), oid));
        Ok(())
    }
}

pub struct GitTable {
    name: String,
    content: Vec<u8>,
}

impl<'repo> TableSink for GitTable {
    fn write_row(&mut self, row: &rusqlite::Row) -> io::Result<()> {
        write_json_row(&mut self.content, row)
    }
}
