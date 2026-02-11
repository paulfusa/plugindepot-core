//! Plugin registry and scanning functionality for PluginDepot.
//!
//! This module provides cross-platform audio plugin discovery and management.
//! 
//! # Architecture
//! 
//! This Rust core is designed to be consumed via FFI by native UI frontends:
//! - **macOS**: SwiftUI app using static lib via Swift ↔ Rust FFI
//! - **Windows**: WPF app using DLL via C# ↔ Rust P/Invoke
//! 
//! # Platform-Specific Behavior
//! 
//! ## macOS
//! - Plugins are bundles (directories with extensions like .vst, .vst3, .component, .aaxplugin)
//! - Scans both system `/Library` and user `~/Library` locations
//! - Supports AU (Audio Units), VST2, VST3, and AAX formats
//! 
//! ## Windows
//! - VST2 and AAX plugins are DLL files (.dll, .aax)
//! - VST3 can be either .vst3 bundles or files in VST3 directory
//! - Scans Program Files and Common Files locations
//! - No AU support (macOS-only format)

use crate::{InstalledPlugin, Plugin, PluginFormat, RelatedPaths};
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

impl PluginFormat {
    /// Returns the file extension for this plugin format on the current platform
    #[cfg(target_os = "macos")]
    fn extension(&self) -> &str {
        match self {
            PluginFormat::VST2 => "vst",
            PluginFormat::VST3 => "vst3",
            PluginFormat::AU => "component",
            PluginFormat::AAX => "aaxplugin",
        }
    }
    
    #[cfg(target_os = "windows")]
    fn extension(&self) -> &str {
        match self {
            PluginFormat::VST2 => "dll",
            PluginFormat::VST3 => "vst3",  // VST3 on Windows can be either .vst3 bundle or in VST3 folder
            PluginFormat::AU => "component", // AU doesn't exist on Windows, but keep for completeness
            PluginFormat::AAX => "aax",
        }
    }
    
    /// Returns true if plugins of this format are bundles (directories) on the current platform
    fn is_bundle(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            true  // All macOS plugins are bundles
        }
        
        #[cfg(target_os = "windows")]
        {
            matches!(self, PluginFormat::VST3)  // Only VST3 can be a bundle on Windows
        }
    }
}

/// Returns all standard audio plugin directories for the current platform.
/// 
/// macOS: Includes both system-wide (/Library) and user-specific (~/Library) locations.
/// Windows: Includes Program Files and Common Files locations.
fn get_plugin_directories() -> Result<Vec<(PathBuf, PluginFormat)>> {
    let mut dirs = Vec::new();
    
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        
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
    }
    
    #[cfg(target_os = "windows")]
    {
        // VST2 - Legacy Steinberg format (common locations on Windows)
        dirs.push((PathBuf::from(r"C:\Program Files\VSTPlugins"), PluginFormat::VST2));
        dirs.push((PathBuf::from(r"C:\Program Files\Steinberg\VSTPlugins"), PluginFormat::VST2));
        dirs.push((PathBuf::from(r"C:\Program Files\Common Files\VST2"), PluginFormat::VST2));
        dirs.push((PathBuf::from(r"C:\Program Files (x86)\VSTPlugins"), PluginFormat::VST2));
        dirs.push((PathBuf::from(r"C:\Program Files (x86)\Steinberg\VSTPlugins"), PluginFormat::VST2));
        
        // VST3 - Modern Steinberg format
        dirs.push((PathBuf::from(r"C:\Program Files\Common Files\VST3"), PluginFormat::VST3));
        
        // AAX - Avid Pro Tools format
        dirs.push((PathBuf::from(r"C:\Program Files\Common Files\Avid\Audio\Plug-Ins"), PluginFormat::AAX));
        dirs.push((PathBuf::from(r"C:\Program Files (x86)\Common Files\Avid\Audio\Plug-Ins"), PluginFormat::AAX));
    }
    
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
    let is_bundle = format.is_bundle();
    
    let entries = fs::read_dir(dir)
        .context(format!("Failed to read directory: {:?}", dir))?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        // On macOS, plugins are bundles (directories)
        // On Windows, most plugins are DLLs (files), except some VST3 can be bundles
        let is_expected_type = if is_bundle {
            path.is_dir()
        } else {
            path.is_file()
        };
        
        if !is_expected_type {
            continue;
        }
        
        // Check the extension matches (e.g., Plugin.vst3, Plugin.component, Plugin.dll)
        if let Some(ext) = path.extension() {
            if ext.to_str() == Some(extension) {
                // Extract plugin metadata
                // TODO: Parse actual plugin bundle metadata (Info.plist for AU/VST3, etc.)
                if let Some(name) = path.file_stem() {
                    let plugin_name = name.to_string_lossy().to_string();
                    
                    // Generate icon URL (this could be customized to fetch from a plugin database)
                    let icon_url = generate_icon_url(&plugin_name);
                    
                    // Create a minimal Plugin entry
                    let plugin = Plugin {
                        id: format!("{}.{}", format!("{:?}", format).to_lowercase(), plugin_name.to_lowercase().replace(" ", "-")),
                        name: plugin_name.clone(),
                        version: String::from("unknown"), // TODO: Extract from bundle
                        description: Some(format!("{:?} plugin", format)),
                        author: None, // TODO: Extract from bundle
                        icon_url,
                    };
                    
                    // Discover related files for this plugin
                    let related_paths = discover_related_paths(&plugin_name, format);
                    
                    plugins.push(InstalledPlugin {
                        plugin,
                        install_path: path,
                        format: format.clone(),
                        enabled: true, // TODO: Check if plugin is disabled in DAW settings
                        related_paths,
                    });
                }
            }
        }
    }
    
    Ok(plugins)
}

/// Discovers related files and folders for a plugin (presets, libraries, support files).
/// This scans common locations where plugins store their data.
fn discover_related_paths(plugin_name: &str, _format: &PluginFormat) -> RelatedPaths {
    let mut paths = RelatedPaths::default();
    
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            // Common preset locations on macOS
            let preset_candidates = vec![
                format!("{}/Music/{}", home, plugin_name),
                format!("{}/Library/Audio/Presets/{}", home, plugin_name),
                format!("{}/Documents/{}", home, plugin_name),
                format!("{}/Documents/{} Library", home, plugin_name),
            ];
            
            // Common library/content locations
            let library_candidates = vec![
                format!("/Library/Application Support/{}", plugin_name),
                format!("{}/Library/Application Support/{}", home, plugin_name),
                format!("/Library/Audio/Sounds/{}", plugin_name),
                format!("{}/Library/Audio/Sounds/{}", home, plugin_name),
            ];
            
            // Preferences locations
            let pref_candidates = vec![
                format!("{}/Library/Preferences/com.{}.plist", home, plugin_name.to_lowercase().replace(" ", "")),
                format!("{}/Library/Preferences/{}.plist", home, plugin_name.replace(" ", "")),
            ];
            
            paths.preset_locations = preset_candidates.into_iter()
                .map(PathBuf::from)
                .filter(|p| p.exists())
                .collect();
            
            paths.library_locations = library_candidates.into_iter()
                .map(PathBuf::from)
                .filter(|p| p.exists())
                .collect();
            
            paths.preference_files = pref_candidates.into_iter()
                .map(PathBuf::from)
                .filter(|p| p.exists())
                .collect();
        }
    }
    
    #[cfg(target_os = "windows")]
    {
        // Common locations on Windows
        if let Ok(appdata) = std::env::var("APPDATA") {
            let preset_candidates = vec![
                format!(r"{}\{}", appdata, plugin_name),
                format!(r"{}\{}\Presets", appdata, plugin_name),
            ];
            
            paths.preset_locations = preset_candidates.into_iter()
                .map(PathBuf::from)
                .filter(|p| p.exists())
                .collect();
        }
        
        if let Ok(programdata) = std::env::var("PROGRAMDATA") {
            let library_candidates = vec![
                format!(r"{}\{}", programdata, plugin_name),
            ];
            
            paths.library_locations = library_candidates.into_iter()
                .map(PathBuf::from)
                .filter(|p| p.exists())
                .collect();
        }
        
        // TODO: Check registry for additional paths
    }
    
    paths
}

/// Generates an icon URL for a plugin.
/// This can be customized to fetch from a plugin database or API.
/// For now, it returns a placeholder URL that can be populated from metadata or external sources.
fn generate_icon_url(_plugin_name: &str) -> Option<String> {
    // TODO: Implement actual icon URL lookup from:
    // - Plugin bundle metadata (Info.plist)
    // - External plugin database/API
    // - Local icon storage
    // - Vendor website
    
    // For now, return None - icons should be populated from external sources
    // Example format if you had a plugin database:
    // Some(format!("https://pluginicons.example.com/{}.png", 
    //     plugin_name.to_lowercase().replace(" ", "-")))
    
    None
}

/// Enumerates all files associated with a plugin for uninstall or backup.
/// Returns a complete list of paths that should be removed/backed up.
pub fn enumerate_plugin_files(plugin: &InstalledPlugin) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    // Add the main plugin binary/bundle
    files.push(plugin.install_path.clone());
    
    // Add all discovered related paths
    for path in &plugin.related_paths.preset_locations {
        files.extend(enumerate_directory_recursive(path)?);
    }
    
    for path in &plugin.related_paths.library_locations {
        files.extend(enumerate_directory_recursive(path)?);
    }
    
    for path in &plugin.related_paths.support_locations {
        files.extend(enumerate_directory_recursive(path)?);
    }
    
    files.extend(plugin.related_paths.preference_files.clone());
    
    Ok(files)
}

/// Recursively enumerates all files in a directory.
fn enumerate_directory_recursive(dir: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    if !dir.exists() {
        return Ok(files);
    }
    
    if dir.is_file() {
        files.push(dir.clone());
        return Ok(files);
    }
    
    let entries = fs::read_dir(dir)
        .context(format!("Failed to read directory: {:?}", dir))?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            files.extend(enumerate_directory_recursive(&path)?);
        } else {
            files.push(path);
        }
    }
    
    Ok(files)
}

// TODO: Future functions to implement:
// - extract_bundle_metadata() - Parse Info.plist from AU/VST3 bundles
// - backup_plugin() - Create backup of plugin and its libraries
// - remove_plugin() - Safely remove plugin and clean up libraries
// - export_for_migration() - Package plugin for moving to another machine
// - import_plugin() - Restore plugin from migration package

/// Detects orphaned files - files in plugin directories that don't belong to any installed plugin.
/// This helps identify leftovers from uninstalled plugins.
pub fn detect_orphaned_files() -> Result<Vec<PathBuf>> {
    let plugin_dirs = get_plugin_directories()?;
    let installed = scan_installed()?;
    let mut orphaned = Vec::new();
    
    // Build a set of all known plugin paths
    let mut known_paths = std::collections::HashSet::new();
    for plugin in &installed {
        known_paths.insert(plugin.install_path.clone());
        // Also add all related paths
        for path in &plugin.related_paths.preset_locations {
            known_paths.insert(path.clone());
        }
        for path in &plugin.related_paths.library_locations {
            known_paths.insert(path.clone());
        }
    }
    
    // Scan each plugin directory for files not in our known set
    for (dir, _format) in plugin_dirs {
        if !dir.exists() {
            continue;
        }
        
        match fs::read_dir(&dir) {
            Ok(entries) => {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        // If this path is not in our known plugins, it's orphaned
                        if !known_paths.contains(&path) {
                            orphaned.push(path);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to scan directory {:?}: {}", dir, e);
            }
        }
    }
    
    Ok(orphaned)
}