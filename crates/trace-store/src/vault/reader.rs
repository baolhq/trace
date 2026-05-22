use std::path::Path;

pub struct VaultReader {
    root: std::path::PathBuf,
}

impl VaultReader {
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self { root: root.as_ref().to_owned() }
    }

    pub fn read_node(&self, relative_path: &str) -> std::io::Result<String> {
        std::fs::read_to_string(self.root.join(relative_path))
    }

    pub fn exists(&self, relative_path: &str) -> bool {
        self.root.join(relative_path).exists()
    }
}
