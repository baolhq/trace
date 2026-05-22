use crate::error::CoreError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NodeId(String);

impl NodeId {
    pub fn new(s: impl Into<String>) -> Result<Self, CoreError> {
        let s = s.into();
        if s.is_empty() {
            return Err(CoreError::InvalidId(s));
        }
        Ok(Self(s))
    }

    /// Generates a short (16-char hex), collision-resistant, URL-safe node ID.
    pub fn generate() -> Self {
        use rand::RngCore;
        let mut bytes = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self(format!("{:016x}", u64::from_le_bytes(bytes)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
