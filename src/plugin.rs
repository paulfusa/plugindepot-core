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
    /// Related paths discovered for this plugin
    pub related_paths: RelatedPaths,
}

/// Paths to plugin-related files and folders
#[derive(Debug, Clone, Default)]
pub struct RelatedPaths {
    /// User preset folders
    pub preset_locations: Vec<PathBuf>,
    /// Factory preset/content folders (samples, IRs, etc.)
    pub library_locations: Vec<PathBuf>,
    /// Application Support folders (settings, licenses, etc.)
    pub support_locations: Vec<PathBuf>,
    /// Preferences/config files
    pub preference_files: Vec<PathBuf>,
}

/// Vendor/manufacturer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vendor {
    pub id: String,
    pub name: String,
    pub website: Option<String>,
}

/// License information for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    /// License key or serial number
    pub key: Option<String>,
    /// License type (e.g., "Full", "Trial", "NFR")
    pub license_type: Option<String>,
    /// Email associated with license
    pub email: Option<String>,
    /// License file path
    pub license_file: Option<PathBuf>,
    /// Expiration date for trial licenses
    pub expiration: Option<String>,
}

/// Complete plugin metadata including vendor and licensing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub plugin: Plugin,
    pub vendor: Option<Vendor>,
    pub license: Option<License>,
    /// Tags for categorization (e.g., "synth", "effect", "compressor")
    pub tags: Vec<String>,
}