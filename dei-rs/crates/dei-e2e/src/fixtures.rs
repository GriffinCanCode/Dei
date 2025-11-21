//! Test fixtures management

use anyhow::Result;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use std::fs;

/// Manages temporary test fixtures
pub struct FixtureManager {
    temp_dir: TempDir,
}

impl FixtureManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            temp_dir: TempDir::new()?,
        })
    }

    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Copy fixture directory to temp location
    pub fn copy_fixture(&self, name: &str) -> Result<PathBuf> {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("fixtures")
            .join(name);
        
        let dest = self.temp_dir.path().join(name);
        copy_dir_all(&source, &dest)?;
        Ok(dest)
    }

    /// Create a file in the temp directory
    pub fn create_file(&self, relative_path: &str, content: &str) -> Result<PathBuf> {
        let path = self.temp_dir.path().join(relative_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
        Ok(path)
    }
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

