use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

/// Lists every regular file under `root`, honoring `.gitignore` files.
#[must_use]
pub fn collect_files(root: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = WalkBuilder::new(root)
        .build()
        .flatten()
        .filter(|entry| entry.file_type().is_some_and(|kind| kind.is_file()))
        .map(ignore::DirEntry::into_path)
        .collect();
    files.sort();
    files
}
