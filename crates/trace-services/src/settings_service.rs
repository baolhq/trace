use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

// ── Enum types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LineHeight {
    Compact,
    Standard,
    Comfortable,
}

impl Default for LineHeight {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EpochPrecision {
    Seconds,
    Milliseconds,
}

impl Default for EpochPrecision {
    fn default() -> Self {
        Self::Seconds
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BackupFrequency {
    Daily,
    Weekly,
    Monthly,
}

impl Default for BackupFrequency {
    fn default() -> Self {
        Self::Daily
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ZenVisibility {
    Show,
    Hide,
    Peek,
}

impl Default for ZenVisibility {
    fn default() -> Self {
        Self::Peek
    }
}

// ── Settings struct ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    // Fonts
    pub ui_font: String,
    pub ui_font_size: u32,
    pub content_font: String,
    pub content_font_size: u32,
    pub mono_font: String,
    pub mono_font_size: u32,

    // Display
    pub zoom_level: f64,
    pub inline_title: bool,
    pub content_line_height: LineHeight,
    pub theme: String,

    // Markdown / editor
    pub wikilink_brackets: bool,
    pub download_remote_images: bool,
    pub date_format: String,
    pub epoch_precision: EpochPrecision,

    // Spellcheck
    pub spellcheck: bool,
    pub spellcheck_languages: Vec<String>,

    // System behavior
    pub close_to_tray: bool,
    pub autostart: bool,
    pub native_app_bar: bool,
    pub multiple_instances: bool,
    pub soft_delete: bool,
    pub auto_backup: bool,
    pub backup_frequency: BackupFrequency,

    // Zen mode
    pub zen_app_bar: ZenVisibility,
    pub zen_sidebar: ZenVisibility,
    pub zen_right_panel: ZenVisibility,
    pub zen_status_bar: ZenVisibility,

    // Misc
    pub vim_escaper: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ui_font: "'Roboto Serif', serif".to_string(),
            ui_font_size: 16,
            content_font: "'Atkinson Hyperlegible', sans-serif".to_string(),
            content_font_size: 16,
            mono_font: "'Martian Mono', monospace".to_string(),
            mono_font_size: 16,
            zoom_level: 1.0,
            inline_title: true,
            content_line_height: LineHeight::Standard,
            theme: "trace-dark".to_string(),
            wikilink_brackets: true,
            download_remote_images: true,
            date_format: "MMMM d, yyyy".to_string(),
            epoch_precision: EpochPrecision::Seconds,
            spellcheck: false,
            spellcheck_languages: vec!["en-US".to_string()],
            close_to_tray: true,
            autostart: false,
            native_app_bar: false,
            multiple_instances: true,
            soft_delete: true,
            auto_backup: false,
            backup_frequency: BackupFrequency::Daily,
            zen_app_bar: ZenVisibility::Peek,
            zen_sidebar: ZenVisibility::Peek,
            zen_right_panel: ZenVisibility::Peek,
            zen_status_bar: ZenVisibility::Peek,
            vim_escaper: false,
        }
    }
}

// ── Service ───────────────────────────────────────────────────────────────────

pub struct SettingsService {
    global_path: PathBuf,
    vault_path: PathBuf,
}

impl SettingsService {
    pub fn new(global_path: PathBuf, vault_path: PathBuf) -> Self {
        Self {
            global_path,
            vault_path,
        }
    }

    pub fn global_path(&self) -> &Path {
        &self.global_path
    }

    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }

    pub fn load_global(&self) -> Settings {
        load_from_path(&self.global_path)
    }

    pub fn load_vault(&self) -> Settings {
        load_from_path(&self.vault_path)
    }

    /// Vault keys override global keys at the TOML level before deserialization,
    /// so absent vault keys inherit from global rather than from hardcoded defaults.
    pub fn merged(&self) -> Settings {
        let global_raw = load_raw(&self.global_path);
        let vault_raw = load_raw(&self.vault_path);
        let merged_raw = merge_tables(global_raw, vault_raw);
        let merged_str = toml::to_string(&merged_raw).unwrap_or_default();
        toml::from_str(&merged_str).unwrap_or_default()
    }

    pub fn save_global(&self, settings: &Settings) -> Result<(), String> {
        let content = toml::to_string_pretty(settings).map_err(|e| e.to_string())?;
        atomic_write(&self.global_path, &content).map_err(|e| e.to_string())
    }

    pub fn save_vault(&self, settings: &Settings) -> Result<(), String> {
        let content = toml::to_string_pretty(settings).map_err(|e| e.to_string())?;
        atomic_write(&self.vault_path, &content).map_err(|e| e.to_string())
    }

    /// Ensures the file for `scope` exists (writes defaults if absent), then
    /// returns its path so the caller can open it in an external editor.
    pub fn ensure_and_get_path(&self, scope: &str) -> Result<PathBuf, String> {
        let path = match scope {
            "global" => &self.global_path,
            "vault" => &self.vault_path,
            _ => return Err(format!("unknown scope: {scope}")),
        };
        if !path.exists() {
            let content =
                toml::to_string_pretty(&Settings::default()).map_err(|e| e.to_string())?;
            atomic_write(path, &content).map_err(|e| e.to_string())?;
        }
        Ok(path.clone())
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn load_from_path(path: &Path) -> Settings {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| toml::from_str(&s).ok())
        .unwrap_or_default()
}

fn load_raw(path: &Path) -> toml::Table {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.parse::<toml::Table>().ok())
        .unwrap_or_default()
}

fn merge_tables(mut base: toml::Table, overlay: toml::Table) -> toml::Table {
    for (k, v) in overlay {
        base.insert(k, v);
    }
    base
}

fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("toml.tmp");
    fs::write(&tmp, content)?;
    fs::rename(&tmp, path)
}
