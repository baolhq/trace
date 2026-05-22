/// Batched, single-writer Tantivy index writer.
pub struct IndexWriter;

impl IndexWriter {
    pub fn index_node(&self, _id: &str, _title: &str, _body: &str) {
        // TODO: add document to Tantivy index, commit in batch
    }

    pub fn delete_node(&self, _id: &str) {
        // TODO: delete_term for the node's id field
    }

    pub fn commit(&mut self) {
        // TODO: writer.commit()
    }
}
