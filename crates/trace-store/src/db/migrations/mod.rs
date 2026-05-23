/// Applies all pending schema migrations, tracking the version in `app_meta`.
pub fn run(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    let version: i64 = conn
        .query_row(
            "SELECT CAST(value AS INTEGER) FROM app_meta WHERE key='schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if version < 1 {
        conn.execute_batch(include_str!("../schema.sql"))?;
        conn.execute(
            "INSERT OR REPLACE INTO app_meta(key,value) VALUES('schema_version','1')",
            [],
        )?;
        tracing::info!("migrations: applied schema v1");
    }

    Ok(())
}
