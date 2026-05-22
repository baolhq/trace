use xxhash_rust::xxh3::xxh3_64;

/// Returns a 16-character lowercase hex string of the xxh3-64 hash.
pub fn hash_content(bytes: &[u8]) -> String {
    format!("{:016x}", xxh3_64(bytes))
}
