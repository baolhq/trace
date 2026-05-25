use std::sync::{atomic::Ordering, Arc};

use serde::Serialize;
use tauri::{ipc::Channel, State};

use crate::state::AppState;

#[derive(Serialize, Clone)]
pub struct SearchHitDto {
    pub id: String,
    pub title: String,
    pub snippet: String,
}

#[tauri::command]
pub fn search_nodes_async(
    query: String,
    is_regex: bool,
    match_case: bool,
    whole_word: bool,
    channel: Channel<SearchHitDto>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let epoch = state.search_epoch.fetch_add(1, Ordering::SeqCst) + 1;

    state.search_service.search_async(
        &query,
        is_regex,
        match_case,
        whole_word,
        500,
        epoch,
        Arc::clone(&state.search_epoch),
        |hit| {
            let _ = channel.send(SearchHitDto {
                id: hit.id,
                title: hit.title,
                snippet: hit.snippet,
            });
        },
    )
}
