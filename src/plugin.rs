use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Audio plugin format types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PluginFormat {
    VST2,
    VST3,
    AU,       // Audio Units (macOS)
    AAX,      // Avid AAX
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InstalledPlugin {
    pub plugin: Plugin,
    pub install_path: PathBuf,
    pub format: PluginFormat,
    pub enabled: bool,
}