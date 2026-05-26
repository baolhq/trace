use std::io::Write;

use rand::Rng;
use tauri::State;

use crate::state::AppState;

const WORDS: &[&str] = &[
    "atlas", "beacon", "cipher", "delta", "epoch", "fusion", "ghost", "harbor", "index", "journal",
    "kernel", "latent", "matrix", "nexus", "orbit", "prism", "quorum", "relay", "signal", "trace",
    "unity", "vector", "warden", "xenon", "yield", "zenith", "anchor", "bridge", "canvas",
    "dagger", "ember", "flare", "glyph", "hinge", "infer", "juncture", "lantern", "mosaic",
    "notion", "opal",
];

#[tauri::command]
pub fn vault_path_cmd(state: State<'_, AppState>) -> String {
    state.vault_path.to_string_lossy().into_owned()
}

#[tauri::command]
pub fn gen_vault_cmd(count: usize, dest: String, state: State<'_, AppState>) -> Result<(), String> {
    let path = if dest.trim().is_empty() {
        state.vault_path.clone()
    } else {
        std::path::PathBuf::from(&dest)
    };

    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;

    let titles: Vec<String> = (0..count).map(|i| format!("note-{i:0>6}")).collect();
    let mut rng = rand::thread_rng();

    for (i, title) in titles.iter().enumerate() {
        let mut f =
            std::fs::File::create(path.join(format!("{title}.md"))).map_err(|e| e.to_string())?;

        for _ in 0..rng.gen_range(2..=5usize) {
            let words: Vec<&str> = (0..rng.gen_range(6..=14usize))
                .map(|_| WORDS[rng.gen_range(0..WORDS.len())])
                .collect();
            writeln!(f, "{}.", words.join(" ")).map_err(|e| e.to_string())?;
        }

        if count > 1 && rng.gen_bool(0.2) {
            writeln!(f).map_err(|e| e.to_string())?;
            for _ in 0..rng.gen_range(1..=3usize).min(count - 1) {
                let mut target = rng.gen_range(0..count);
                if target == i {
                    target = (target + 1) % count;
                }
                writeln!(f, "See also [[{}]].", titles[target]).map_err(|e| e.to_string())?;
            }
        }
    }

    Ok(())
}
