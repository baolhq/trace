use trace_core::model::Folder;

pub struct FoldersRepo;

impl FoldersRepo {
    pub fn create(&self, _folder: &Folder) {
        // TODO: INSERT INTO folders ...
    }

    pub fn list_children(&self, _parent_id: Option<i64>) -> Vec<Folder> {
        // TODO: SELECT * FROM folders WHERE parent_id IS ?
        Vec::new()
    }

    pub fn delete(&self, _id: i64) {
        // TODO: DELETE FROM folders WHERE id = ?
    }
}
