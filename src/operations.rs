//! Plugin management operations (backup, uninstall, export, import).
//!
//! This module provides functions for managing installed plugins:
//! - Backup: Create archives of plugins and their related files
//! - Uninstall: Safely remove plugins and cleanup related files
//! - Export: Package plugins for migration to another machine
//! - Import: Restore plugins from migration packages

use crate::{InstalledPlugin, registry::enumerate_plugin_files};
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Backup a plugin and all its related files to a specified directory.
/// Creates a timestamped folder containing all plugin files.
pub fn backup_plugin(plugin: &InstalledPlugin, backup_dir: &Path) -> Result<PathBuf> {
    // Create backup directory with timestamp
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let plugin_folder_name = format!("{}_{}", plugin.plugin.name.replace(" ", "_"), timestamp);
    let backup_path = backup_dir.join(&plugin_folder_name);
    
    fs::create_dir_all(&backup_path)
        .context(format!("Failed to create backup directory: {:?}", backup_path))?;
    
    // Enumerate all files to backup
    let files = enumerate_plugin_files(plugin)?;
    
    // Copy each file, preserving relative structure
    for file in &files {
        if let Err(e) = copy_file_to_backup(file, &backup_path) {
            eprintln!("Warning: Failed to backup file {:?}: {}", file, e);
        }
    }
    
    // Create a manifest file with plugin metadata
    create_backup_manifest(plugin, &backup_path)?;
    
    Ok(backup_path)
}

/// Uninstall a plugin, removing all associated files.
/// Returns the list of files that were deleted.
pub fn uninstall_plugin(plugin: &InstalledPlugin, dry_run: bool) -> Result<Vec<PathBuf>> {
    let files = enumerate_plugin_files(plugin)?;
    let mut deleted = Vec::new();
    
    if dry_run {
        // Just return what would be deleted
        return Ok(files);
    }
    
    // Delete files in reverse order (files before directories)
    for file in &files {
        match delete_path(file) {
            Ok(_) => deleted.push(file.clone()),
            Err(e) => eprintln!("Warning: Failed to delete {:?}: {}", file, e),
        }
    }
    
    // Also try to remove the main plugin bundle/directory
    if let Err(e) = delete_path(&plugin.install_path) {
        eprintln!("Warning: Failed to delete main plugin at {:?}: {}", plugin.install_path, e);
    } else {
        deleted.push(plugin.install_path.clone());
    }
    
    Ok(deleted)
}

/// Export a plugin for migration to another machine.
/// Creates a portable package that can be imported on the target system.
pub fn export_plugin(plugin: &InstalledPlugin, export_dir: &Path) -> Result<PathBuf> {
    let export_name = format!("{}_export", plugin.plugin.name.replace(" ", "_"));
    let export_path = export_dir.join(&export_name);
    
    fs::create_dir_all(&export_path)
        .context(format!("Failed to create export directory: {:?}", export_path))?;
    
    // Copy all plugin files
    let files = enumerate_plugin_files(plugin)?;
    for file in &files {
        if let Err(e) = copy_file_to_backup(file, &export_path) {
            eprintln!("Warning: Failed to export file {:?}: {}", file, e);
        }
    }
    
    // Create metadata for import
    create_export_manifest(plugin, &export_path)?;
    
    Ok(export_path)
}

/// Import a plugin from an export package created by export_plugin().
/// Restores the plugin and its files to the appropriate system locations.
pub fn import_plugin(_package_path: &Path) -> Result<InstalledPlugin> {
    // TODO: Implement import logic
    // 1. Read manifest from package
    // 2. Determine target locations based on format and platform
    // 3. Copy files to appropriate system locations
    // 4. Verify installation
    // 5. Return InstalledPlugin instance
    
    anyhow::bail!("Import not yet implemented");
}

// Helper functions

/// Copy a file to backup directory, preserving its relative path structure.
fn copy_file_to_backup(source: &Path, backup_dir: &Path) -> Result<()> {
    if !source.exists() {
        return Ok(());
    }
    
    // Use the file name as destination (simplified for now)
    // TODO: Preserve directory structure better
    let file_name = source.file_name()
        .context("Invalid file name")?;
    let dest = backup_dir.join(file_name);
    
    if source.is_dir() {
        copy_directory_recursive(source, &dest)?;
    } else {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, dest)?;
    }
    
    Ok(())
}

/// Recursively copy a directory.
fn copy_directory_recursive(source: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dest.join(entry.file_name());
        
        if path.is_dir() {
            copy_directory_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    
    Ok(())
}

/// Delete a file or directory.
fn delete_path(path: &Path) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    
    if path.is_dir() {
        fs::remove_dir_all(path)
            .context(format!("Failed to remove directory: {:?}", path))?;
    } else {
        fs::remove_file(path)
            .context(format!("Failed to remove file: {:?}", path))?;
    }
    
    Ok(())
}

/// Create a manifest file for backup with plugin metadata.
fn create_backup_manifest(plugin: &InstalledPlugin, backup_dir: &Path) -> Result<()> {
    let manifest_path = backup_dir.join("backup_manifest.json");
    let manifest = serde_json::json!({
        "plugin_name": plugin.plugin.name,
        "plugin_id": plugin.plugin.id,
        "version": plugin.plugin.version,
        "format": format!("{:?}", plugin.format),
        "install_path": plugin.install_path.to_string_lossy(),
        "backup_date": chrono::Local::now().to_rfc3339(),
    });
    
    let content = serde_json::to_string_pretty(&manifest)?;
    fs::write(manifest_path, content)?;
    
    Ok(())
}

/// Create a manifest file for export with platform-independent metadata.
fn create_export_manifest(plugin: &InstalledPlugin, export_dir: &Path) -> Result<()> {
    let manifest_path = export_dir.join("export_manifest.json");
    let manifest = serde_json::json!({
        "plugin_name": plugin.plugin.name,
        "plugin_id": plugin.plugin.id,
        "version": plugin.plugin.version,
        "format": format!("{:?}", plugin.format),
        "description": plugin.plugin.description,
        "author": plugin.plugin.author,
        "export_date": chrono::Local::now().to_rfc3339(),
    });
    
    let content = serde_json::to_string_pretty(&manifest)?;
    fs::write(manifest_path, content)?;
    
    Ok(())
}
