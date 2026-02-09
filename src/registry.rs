use crate::{InstalledPlugin, Plugin, PluginFormat};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

impl PluginFormat {
    /// Returns the file extension for this plugin format
    fn extension(&self) -> &str {
        match self {
            PluginFormat::VST2 => "vst",
            PluginFormat::VST3 => "vst3",
            PluginFormat::AU => "component",
            PluginFormat::AAX => "aaxplugin",
        }
    }
}

/// Returns all standard audio plugin directories on macOS.
/// Includes both system-wide (/Library) and user-specific (~/Library) locations.
fn get_plugin_directories() -> Result<Vec<(PathBuf, PluginFormat)>> {
    let home = std::env::var("HOME").context("HOME environment variable not set")?;
    
    let mut dirs = Vec::new();
    
    // Audio Units (AU) - macOS native format
    dirs.push((PathBuf::from("/Library/Audio/Plug-Ins/Components"), PluginFormat::AU));
    dirs.push((PathBuf::from(format!("{}/Library/Audio/Plug-Ins/Components", home)), PluginFormat::AU));
    
    // VST2 - Legacy Steinberg format
    dirs.push((PathBuf::from("/Library/Audio/Plug-Ins/VST"), PluginFormat::VST2));
    dirs.push((PathBuf::from(format!("{}/Library/Audio/Plug-Ins/VST", home)), PluginFormat::VST2));
    
    // VST3 - Modern Steinberg format
    dirs.push((PathBuf::from("/Library/Audio/Plug-Ins/VST3"), PluginFormat::VST3));
    dirs.push((PathBuf::from(format!("{}/Library/Audio/Plug-Ins/VST3", home)), PluginFormat::VST3));
    
    // AAX - Avid Pro Tools format
    dirs.push((PathBuf::from("/Library/Application Support/Avid/Audio/Plug-Ins"), PluginFormat::AAX));
    
    Ok(dirs)
}

/// Scans all standard audio plugin directories and returns a list of installed plugins.
/// Returns an empty list if no plugins are found.
pub fn scan_installed() -> Result<Vec<InstalledPlugin>> {
    let plugin_dirs = get_plugin_directories()?;
    let mut installed = Vec::new();
    
    for (dir, format) in plugin_dirs {
        // Skip directories that don't exist
        if !dir.exists() {
            continue;
        }
        
        // Scan this directory for plugins
        match scan_directory(&dir, &format) {
            Ok(mut plugins) => installed.append(&mut plugins),
            Err(e) => {
                eprintln!("Warning: Failed to scan directory {:?}: {}", dir, e);
            }
        }
    }
    
    Ok(installed)
}

/// Scans a single directory for plugins of a specific format.
fn scan_directory(dir: &PathBuf, format: &PluginFormat) -> Result<Vec<InstalledPlugin>> {
    let mut plugins = Vec::new();
    let extension = format.extension();
    
    let entries = fs::read_dir(dir)
        .context(format!("Failed to read directory: {:?}", dir))?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        // Check if this matches the expected plugin format
        // Audio plugins are typically bundles (directories with specific extensions)
        if !path.is_dir() {
            continue;
        }
        
        // Check the extension matches (e.g., Plugin.vst3, Plugin.component)
        if let Some(ext) = path.extension() {
            if ext.to_str() == Some(extension) {
                // Extract plugin metadata
                // TODO: Parse actual plugin bundle metadata (Info.plist for AU/VST3, etc.)
                if let Some(name) = path.file_stem() {
                    let plugin_name = name.to_string_lossy().to_string();
                    
                    // Create a minimal Plugin entry
                    let plugin = Plugin {
                        id: format!("{}.{}", format!("{:?}", format).to_lowercase(), plugin_name.to_lowercase().replace(" ", "-")),
                        name: plugin_name,
                        version: String::from("unknown"), // TODO: Extract from bundle
                        description: Some(format!("{:?} plugin", format)),
                        author: None, // TODO: Extract from bundle
                    };
                    
                    plugins.push(InstalledPlugin {
                        plugin,
                        install_path: path,
                        format: format.clone(),
                        enabled: true, // TODO: Check if plugin is disabled in DAW settings
                    });
                }
            }
        }
    }
    
    Ok(plugins)
}

// TODO: Future functions to implement:
// - extract_bundle_metadata() - Parse Info.plist from AU/VST3 bundles
// - find_plugin_libraries() - Locate associated library files scattered across system
// - backup_plugin() - Create backup of plugin and its libraries
// - remove_plugin() - Safely remove plugin and clean up libraries