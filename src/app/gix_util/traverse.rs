use std::{
    borrow::Cow,
    collections::{HashMap, VecDeque},
    io,
};

use gix::{
    bstr::{BStr, BString, ByteSlice, ByteVec},
    objs::tree::EntryRef,
    traverse::tree::visit::Action,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BlobEntry {
    /// The full path of the entry, relative to the repository root.
    pub path: BString,
    pub oid: gix::ObjectId,
}

impl BlobEntry {
    fn new(entry: &EntryRef<'_>, path: BString) -> Self {
        Self {
            path,
            oid: entry.oid.to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TreeEntry {
    /// The full path of the entry, relative to the repository root.
    pub path: BString,
    pub oid: gix::ObjectId,
    entries: Vec<BlobEntry>,
}

impl TreeEntry {
    fn new(entry: &EntryRef<'_>, path: BString) -> Self {
        Self {
            path,
            oid: entry.oid.to_owned(),
            entries: Vec::new(),
        }
    }

    fn add_blob(&mut self, entry: &EntryRef<'_>, path: BString) {
        self.entries.push(BlobEntry::new(entry, path));
    }
}

pub(crate) struct Traverse<'repo> {
    repo: Option<&'repo gix::Repository>,
    /// the current path that is being traversed
    path: BString,
    /// tree entries that are to be traversed, in order of traversal
    path_deque: VecDeque<BString>,
    /// entries to blobs that have been observed
    pub records: Vec<BlobEntry>,
}

// Debug
impl std::fmt::Debug for Traverse<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Traverse")
            .field("repo", &self.repo)
            .field("path", &self.path)
            .field("path_deque", &self.path_deque)
            .finish()
    }
}

impl<'repo> Traverse<'repo> {
    pub fn new(repo: Option<&'repo gix::Repository>) -> Self {
        Traverse {
            repo,
            path: BString::default(),
            path_deque: VecDeque::new(),
            records: Vec::new(),
        }
    }

    fn pop_element(&mut self) {
        if let Some(pos) = self.path.rfind_byte(b'/') {
            self.path.resize(pos, 0);
        } else {
            self.path.clear();
        }
    }

    fn push_element(&mut self, name: &BStr) {
        if !self.path.is_empty() {
            self.path.push(b'/');
        }
        self.path.push_str(name);
    }
}

impl<'repo> gix::traverse::tree::Visit for Traverse<'repo> {
    #[tracing::instrument(skip(self))]
    fn pop_front_tracked_path_and_set_current(&mut self) {
        tracing::info!(?self.path, ?self.path_deque);
        self.path = self
            .path_deque
            .pop_front()
            .expect("every call should be matched with `push_back_tracked_path_component`");
        tracing::info!(?self.path, ?self.path_deque);
    }

    #[tracing::instrument(skip(self))]
    fn push_back_tracked_path_component(&mut self, component: &BStr) {
        tracing::info!(?component, ?self.path, ?self.path_deque);
        self.push_element(component);
        self.path_deque.push_back(self.path.clone());
        tracing::info!(?component, ?self.path, ?self.path_deque);
    }

    #[tracing::instrument(skip(self))]
    fn push_path_component(&mut self, component: &BStr) {
        tracing::info!(?component, ?self.path, ?self.path_deque);
        self.push_element(component);
        tracing::info!(?component, ?self.path, ?self.path_deque);
    }

    #[tracing::instrument(skip(self))]
    fn pop_path_component(&mut self) {
        tracing::info!(?self.path, ?self.path_deque);
        let res = self.pop_element();
        tracing::info!(?self.path, ?self.path_deque);
        res
    }

    #[tracing::instrument(skip(self))]
    fn visit_tree(&mut self, entry: &EntryRef<'_>) -> Action {
        tracing::info!(?entry, ?self.path, ?self.path_deque);
        Action::Continue
    }

    #[tracing::instrument(skip(self))]
    fn visit_nontree(&mut self, entry: &EntryRef<'_>) -> Action {
        tracing::info!(?entry, ?self.path, ?self.path_deque);
        self.records.push(BlobEntry::new(entry, self.path.clone()));
        Action::Continue
    }
}

fn format_entry(
    mut out: impl io::Write,
    entry: &gix::objs::tree::EntryRef<'_>,
    filename: &gix::bstr::BStr,
    size: Option<usize>,
) -> std::io::Result<()> {
    use gix::objs::tree::EntryKind::*;
    writeln!(
        out,
        "{} {}{} {}",
        match entry.mode.kind() {
            Tree => "TREE",
            Blob => "BLOB",
            BlobExecutable => " EXE",
            Link => "LINK",
            Commit => "SUBM",
        },
        entry.oid,
        size.map_or_else(|| "".into(), |s| Cow::Owned(format!(" {s}"))),
        filename
    )
}
/*
/// A trait to allow responding to a traversal designed to observe all entries in a tree, recursively while keeping track of
/// paths if desired.
pub trait Visit {
    /// Sets the full path path in front of the queue so future calls to push and pop components affect it instead.
    fn pop_front_tracked_path_and_set_current(&mut self);
    /// Append a `component` to the end of a path, which may be empty.
    fn push_back_tracked_path_component(&mut self, component: &BStr);
    /// Append a `component` to the end of a path, which may be empty.
    fn push_path_component(&mut self, component: &BStr);
    /// Removes the last component from the path, which may leave it empty.
    fn pop_path_component(&mut self);

    /// Observe a tree entry that is a tree and return an instruction whether to continue or not.
    /// [`Action::Skip`][visit::Action::Skip] can be used to prevent traversing it, for example if it's known to the caller already.
    ///
    /// The implementation may use the current path to learn where in the tree the change is located.
    fn visit_tree(&mut self, entry: &gix_object::tree::EntryRef<'_>) -> visit::Action;

    /// Observe a tree entry that is NO tree and return an instruction whether to continue or not.
    /// [`Action::Skip`][visit::Action::Skip] has no effect here.
    ///
    /// The implementation may use the current path to learn where in the tree the change is located.
    fn visit_nontree(&mut self, entry: &gix_object::tree::EntryRef<'_>) -> visit::Action;
}
*/
