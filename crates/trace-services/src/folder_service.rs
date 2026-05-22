pub struct FolderService;

impl FolderService {
    pub fn create(&self, _name: &str, _parent_id: Option<i64>) {
        // TODO: insert folder, return new id
    }

    pub fn add_node(&self, _folder_id: i64, _node_id: &str) {
        // TODO: insert into folder_members
    }
}
