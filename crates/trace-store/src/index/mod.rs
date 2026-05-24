pub mod schema;
pub mod searcher;
pub mod writer;

pub use schema::IndexSchema;
pub use searcher::SearchResult;

use std::{path::Path, sync::Mutex};

use tantivy::{
    collector::TopDocs, query::QueryParser, schema::Value, Index, IndexReader, IndexWriter,
    ReloadPolicy, TantivyDocument, Term,
};

pub struct SearchIndex {
    schema: IndexSchema,
    index: Index,
    writer: Mutex<IndexWriter>,
    reader: IndexReader,
}

impl SearchIndex {
    pub fn open_or_create(path: &Path) -> Result<Self, String> {
        let schema = IndexSchema::build();

        // If the index exists but has a stale schema, nuke and recreate it.
        let index = if path.join("meta.json").exists() {
            match Index::open_in_dir(path) {
                Ok(idx) => idx,
                Err(_) => {
                    let _ = std::fs::remove_dir_all(path);
                    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
                    Index::create_in_dir(path, schema.inner.clone()).map_err(|e| e.to_string())?
                }
            }
        } else {
            std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
            Index::create_in_dir(path, schema.inner.clone()).map_err(|e| e.to_string())?
        };

        let writer = index.writer(50_000_000).map_err(|e| e.to_string())?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .map_err(|e: tantivy::TantivyError| e.to_string())?;

        Ok(Self {
            schema,
            index,
            writer: Mutex::new(writer),
            reader,
        })
    }

    /// Index or re-index a single node. Delete-then-add handles updates.
    /// `tags` is a space-joined string of tag names.
    pub fn index_node(&self, id: &str, title: &str, body: &str, tags: &str, modified_at: i64) {
        let Ok(w) = self.writer.lock() else { return };
        w.delete_term(Term::from_field_text(self.schema.id, id));
        let mut doc = TantivyDocument::default();
        doc.add_text(self.schema.id, id);
        doc.add_text(self.schema.title, title);
        doc.add_text(self.schema.body, body);
        doc.add_text(self.schema.tags, tags);
        doc.add_i64(self.schema.modified_at, modified_at);
        let _ = w.add_document(doc);
    }

    pub fn delete_node(&self, id: &str) {
        let Ok(w) = self.writer.lock() else { return };
        w.delete_term(Term::from_field_text(self.schema.id, id));
    }

    pub fn commit(&self) {
        let Ok(mut w) = self.writer.lock() else {
            return;
        };
        let _ = w.commit();
    }

    /// Full-text search across title (boosted 3×) + body + tags.
    /// Returns hits with id and title only — snippets come from vault files.
    pub fn search(&self, query_str: &str, limit: usize) -> Vec<SearchResult> {
        let searcher = self.reader.searcher();
        let mut parser = QueryParser::for_index(
            &self.index,
            vec![self.schema.title, self.schema.body, self.schema.tags],
        );
        parser.set_conjunction_by_default();
        parser.set_field_boost(self.schema.title, 3.0);

        let Ok(query) = parser.parse_query(query_str) else {
            return Vec::new();
        };

        let Ok(top_docs) = searcher.search(&*query, &TopDocs::with_limit(limit)) else {
            return Vec::new();
        };

        top_docs
            .into_iter()
            .filter_map(|(score, addr)| {
                let doc: TantivyDocument = searcher.doc(addr).ok()?;
                let id = doc.get_first(self.schema.id)?.as_str()?.to_owned();
                let title = doc.get_first(self.schema.title)?.as_str()?.to_owned();
                Some(SearchResult { id, title, score })
            })
            .collect()
    }
}
