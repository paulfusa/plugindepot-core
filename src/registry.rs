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

use crate::{InstalledPlugin, Plugin, PluginFormat};
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