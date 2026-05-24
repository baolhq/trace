use tantivy::schema::{Field, NumericOptions, SchemaBuilder, STORED, STRING, TEXT};

pub struct IndexSchema {
    pub inner: tantivy::schema::Schema,
    /// Exact node ID — STRING | STORED (used as doc key for delete-by-id).
    pub id: Field,
    /// Full-text, boosted in queries — TEXT | STORED (title shown in results).
    pub title: Field,
    /// Full-text — TEXT only, NOT stored. Snippets are pulled from vault on demand.
    pub body: Field,
    /// Space-joined tag names — TEXT only (facet-like filtering/search).
    pub tags: Field,
    /// Unix timestamp ms — FAST only (recency sort/boost, never retrieved).
    pub modified_at: Field,
}

impl IndexSchema {
    pub fn build() -> Self {
        let mut builder = SchemaBuilder::new();
        let id = builder.add_text_field("id", STRING | STORED);
        let title = builder.add_text_field("title", TEXT | STORED);
        let body = builder.add_text_field("body", TEXT);
        let tags = builder.add_text_field("tags", TEXT);
        let modified_at =
            builder.add_i64_field("modified_at", NumericOptions::default().set_fast());
        let inner = builder.build();
        Self {
            inner,
            id,
            title,
            body,
            tags,
            modified_at,
        }
    }
}
