#[derive(Debug, Clone)]
pub enum CoreEvent {
    FileChanged {
        path: String,
        kind: String,
    },
    NodeSaved {
        id: String,
        hash: String,
    },
    LinksUpdated {
        id: String,
    },
    ScanProgress {
        done: u64,
        total: u64,
    },
    ScanComplete,
    /// Fired by FileSync whenever a node is inserted or removed due to external
    /// file-system changes. The UI should refresh its node list on this event.
    NodesChanged,
}
