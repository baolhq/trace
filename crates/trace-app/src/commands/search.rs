use serde::Serialize;
use tauri::State;

use crate::state::AppState;

#[derive(Serialize)]
pub struct SearchHitDto {
    pub id: String,
    pub title: String,
    pub snippet: String,
}

#[tauri::command]
pub fn search_nodes(
    query: String,
    is_regex: bool,
    match_case: bool,
    whole_word: bool,
    state: State<'_, AppState>,
) -> Result<Vec<SearchHitDto>, String> {
    state
        .search_service
        .search(&query, is_regex, match_case, whole_word, 50)
        .map(|hits| {
            hits.into_iter()
                .map(|h| SearchHitDto {
                    id: h.id,
                    title: h.title,
                    snippet: h.snippet,
                })
                .collect()
        })
}
