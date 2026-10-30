//! `SaveStore` on the file system: one directory, atomic replacement by write-then-rename.

use std::fs::{self, File};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use voxel_core::{SaveStore, SaveStoreError};

/// Save files in one directory. A write goes to `<name>.tmp`, is flushed to disk, then renamed
/// over `<name>`, so a crash leaves either the old file or the new one.
#[derive(Debug, Clone)]
pub struct FileSaveStore {
    directory: PathBuf,
}

impl FileSaveStore {
    /// A store in `directory`, created when missing.
    ///
    /// # Errors
    /// Returns [`SaveStoreError::Unavailable`] when the directory cannot be created.
    pub fn open(directory: impl Into<PathBuf>) -> Result<Self, SaveStoreError> {
        let directory = directory.into();
        fs::create_dir_all(&directory).map_err(|error| unavailable(&directory, &error))?;
        Ok(Self { directory })
    }

    fn path(&self, name: &str) -> PathBuf {
        self.directory.join(name)
    }
}

impl SaveStore for FileSaveStore {
    fn read(&self, name: &str) -> Result<Option<Vec<u8>>, SaveStoreError> {
        let path = self.path(name);
        match fs::read(&path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(unavailable(&path, &error)),
        }
    }

    fn write(&mut self, name: &str, bytes: &[u8]) -> Result<(), SaveStoreError> {
        let path = self.path(name);
        let temporary = self.path(&format!("{name}.tmp"));
        let write = || -> std::io::Result<()> {
            let mut file = File::create(&temporary)?;
            file.write_all(bytes)?;
            file.sync_all()?;
            fs::rename(&temporary, &path)
        };
        write().map_err(|error| {
            // Best effort: a stale temporary file is harmless, the real file is untouched.
            let _ = fs::remove_file(&temporary);
            unavailable(&path, &error)
        })
    }

    fn copy(&mut self, from: &str, to: &str) -> Result<(), SaveStoreError> {
        let source = self.path(from);
        if !source.exists() {
            return Ok(());
        }
        let bytes = fs::read(&source).map_err(|error| unavailable(&source, &error))?;
        self.write(to, &bytes)
    }
}

fn unavailable(path: &Path, error: &std::io::Error) -> SaveStoreError {
    SaveStoreError::Unavailable(format!("{}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh directory under the system temp dir, removed when dropped.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(test: &str) -> Self {
            let path =
                std::env::temp_dir().join(format!("voxel-save-{test}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn reads_nothing_for_a_file_never_written() {
        let dir = TempDir::new("missing");
        let store = FileSaveStore::open(&dir.0).expect("temp dir");

        assert_eq!(store.read("world.save"), Ok(None));
    }

    #[test]
    fn replaces_a_file_and_leaves_no_temporary_behind() {
        let dir = TempDir::new("replace");
        let mut store = FileSaveStore::open(&dir.0).expect("temp dir");

        store.write("world.save", b"first").expect("writes");
        store.write("world.save", b"second").expect("writes");

        assert_eq!(store.read("world.save"), Ok(Some(b"second".to_vec())));
        assert!(!dir.0.join("world.save.tmp").exists());
    }

    #[test]
    fn copies_for_backups_and_skips_missing_sources() {
        let dir = TempDir::new("copy");
        let mut store = FileSaveStore::open(&dir.0).expect("temp dir");
        store.write("world.save", b"data").expect("writes");

        store.copy("world.save", "world.save.bak").expect("copies");
        store.copy("nothing.save", "nothing.bak").expect("skips");

        assert_eq!(store.read("world.save.bak"), Ok(Some(b"data".to_vec())));
        assert!(!dir.0.join("nothing.bak").exists());
    }
}
