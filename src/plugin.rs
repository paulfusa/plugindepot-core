use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub enabled: bool,
}