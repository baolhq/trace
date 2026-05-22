use std::path::Path;

pub struct VaultWriter {
    root: std::path::PathBuf,
}

impl VaultWriter {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_owned() }
    }

    /// Atomic write via write-to-temp-then-rename to prevent partial reads.
    pub fn write_node(&self, relative_path: &str, content: &str) -> std::io::Result<()> {
        let dest = self.root.join(relative_path);
        let tmp = dest.with_extension("tmp");
        std::fs::write(&tmp, content)?;
        std::fs::rename(tmp, dest)
    }

    pub fn delete_node(&self, relative_path: &str) -> std::io::Result<()> {
        std::fs::remove_file(self.root.join(relative_path))
    }
}
