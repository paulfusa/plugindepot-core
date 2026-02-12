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
    
    // Share icons between VST2 and VST3 versions of the same plugin
    share_icons_between_formats(&mut installed);
    
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
                    
                    // Discover icon from plugin bundle or local files
                    let icon_url = discover_plugin_icon(&path, &plugin_name);
                    
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

/// Share icons between VST2 and VST3 versions of the same plugin.
/// If a VST2 plugin doesn't have an icon but a VST3 version exists with an icon, use it.
fn share_icons_between_formats(plugins: &mut [InstalledPlugin]) {
    // Build a map of plugin names to their icons by format
    let mut vst3_icons: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    
    // First pass: collect VST3 icons
    for plugin in plugins.iter() {
        if matches!(plugin.format, PluginFormat::VST3) {
            if let Some(icon_url) = &plugin.plugin.icon_url {
                // Normalize the plugin name for matching
                let normalized_name = normalize_plugin_name(&plugin.plugin.name);
                vst3_icons.insert(normalized_name, icon_url.clone());
            }
        }
    }
    
    // Second pass: apply VST3 icons to VST2 plugins that don't have icons
    for plugin in plugins.iter_mut() {
        if matches!(plugin.format, PluginFormat::VST2) {
            if plugin.plugin.icon_url.is_none() {
                let normalized_name = normalize_plugin_name(&plugin.plugin.name);
                if let Some(vst3_icon) = vst3_icons.get(&normalized_name) {
                    plugin.plugin.icon_url = Some(vst3_icon.clone());
                }
            }
        }
    }
}

/// Normalize a plugin name for matching between formats.
/// Removes common suffixes, spaces, and converts to lowercase.
fn normalize_plugin_name(name: &str) -> String {
    let mut normalized = name.to_lowercase();
    
    // Remove common version suffixes and architecture markers
    let suffixes_to_remove = [
        " vst", " vst2", " vst3",
        "vst", "vst2", "vst3",
        " x64", " x86", "_x64", "_x86",
        " 64", " 32", "_64", "_32",
        " fx", " effect", "_fx", "_effect",
    ];
    
    for suffix in &suffixes_to_remove {
        // Remove from the end
        while normalized.ends_with(suffix) {
            normalized = normalized.trim_end_matches(suffix).to_string();
        }
    }
    
    // Remove spaces, dashes, and underscores for better matching
    normalized = normalized
        .replace(" ", "")
        .replace("-", "")
        .replace("_", "");
    
    normalized
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

/// Discovers an icon for a plugin by searching in the plugin bundle.
/// Returns a file:// URL to the local icon if found.
fn discover_plugin_icon(plugin_path: &PathBuf, plugin_name: &str) -> Option<String> {
    // For bundle-based plugins (macOS .component, .vst3, .vst, Windows .vst3)
    if plugin_path.is_dir() {
        // Common icon locations within bundles
        let icon_search_paths = vec![
            "Contents/Resources",
            "Resources",
            "Contents",
        ];
        
        // Common icon file extensions
        let icon_extensions = vec!["icns", "png", "ico", "jpg", "jpeg"];
        
        for search_path in &icon_search_paths {
            let resource_dir = plugin_path.join(search_path);
            if !resource_dir.exists() {
                continue;
            }
            
            // Try to find icon files
            if let Ok(entries) = fs::read_dir(&resource_dir) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        if let Some(ext) = entry_path.extension() {
                            let ext_str = ext.to_string_lossy().to_lowercase();
                            if icon_extensions.contains(&ext_str.as_str()) {
                                let file_name = entry_path.file_stem()
                                    .map(|n| n.to_string_lossy().to_lowercase())
                                    .unwrap_or_default();
                                
                                // Prefer files named after the plugin or common icon names
                                let plugin_name_lower = plugin_name.to_lowercase().replace(" ", "");
                                if file_name.contains(&plugin_name_lower) 
                                    || file_name == "icon" 
                                    || file_name == "logo"
                                    || file_name == "appicon"
                                    || file_name.starts_with("icon")
                                    || file_name.starts_with("logo") {
                                    // Return file:// URL
                                    return Some(format!("file://{}", entry_path.display()));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // If no specific icon found, try to find ANY icon file
        for search_path in &icon_search_paths {
            let resource_dir = plugin_path.join(search_path);
            if !resource_dir.exists() {
                continue;
            }
            
            if let Ok(entries) = fs::read_dir(&resource_dir) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        if let Some(ext) = entry_path.extension() {
                            let ext_str = ext.to_string_lossy().to_lowercase();
                            if icon_extensions.contains(&ext_str.as_str()) {
                                // Return the first icon we find
                                return Some(format!("file://{}", entry_path.display()));
                            }
                        }
                    }
                }
            }
        }
    }
    
    // For file-based plugins or if no icon found in bundle,
    // look for icon files in the same directory
    if let Some(parent_dir) = plugin_path.parent() {
        let plugin_name_lower = plugin_name.to_lowercase().replace(" ", "");
        let icon_extensions = vec!["png", "ico", "jpg", "jpeg"];
        
        for ext in icon_extensions {
            // Try plugin-name-specific icon
            let icon_path = parent_dir.join(format!("{}.{}", plugin_name, ext));
            if icon_path.exists() {
                return Some(format!("file://{}", icon_path.display()));
            }
            
            // Try with spaces removed
            let icon_path = parent_dir.join(format!("{}.{}", plugin_name_lower, ext));
            if icon_path.exists() {
                return Some(format!("file://{}", icon_path.display()));
            }
        }
    }
    
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