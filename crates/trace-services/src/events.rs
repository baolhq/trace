#[derive(Debug, Clone)]
pub enum CoreEvent {
    FileChanged { path: String, kind: String },
    NodeSaved { id: String, hash: String },
    LinksUpdated { id: String },
    IndexUpdated { generation: u64, doc_count: u64 },
    ScanProgress { done: u64, total: u64 },
    ScanComplete,
}
