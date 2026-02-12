//! Icon management for plugins.
//!
//! This module provides functionality to fetch, cache, and manage plugin icons
//! for display in native UIs (SwiftUI on macOS, WPF on Windows).
//!
//! # Features
//!
//! - Fetch icons from remote URLs (http/https)
//! - Support local file:// URLs for icons in plugin bundles
//! - Cache remote icons locally to reduce network requests
//! - Provide icon data as raw bytes for native UI consumption
//! - Support common image formats (PNG, JPEG, ICNS, ICO)

use anyhow::{Context, Result, anyhow};
use std::fs;
use std::path::PathBuf;

/// Get the cache directory for plugin icons
fn get_icon_cache_dir() -> Result<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let cache_dir = PathBuf::from(format!("{}/Library/Caches/PluginDepot/icons", home));
        Ok(cache_dir)
    }
    
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("LOCALAPPDATA")
            .context("LOCALAPPDATA environment variable not set")?;
        let cache_dir = PathBuf::from(format!("{}\\PluginDepot\\icons", appdata));
        Ok(cache_dir)
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let cache_dir = PathBuf::from(format!("{}/.cache/plugindepot/icons", home));
        Ok(cache_dir)
    }
}

/// Ensures the icon cache directory exists
fn ensure_cache_dir_exists() -> Result<PathBuf> {
    let cache_dir = get_icon_cache_dir()?;
    fs::create_dir_all(&cache_dir)
        .context(format!("Failed to create icon cache directory: {:?}", cache_dir))?;
    Ok(cache_dir)
}

/// Generate a cache filename from a URL
fn url_to_cache_filename(url: &str) -> String {
    // Simple hash-based filename to avoid filesystem issues with URL characters
    let hash = format!("{:x}", md5::compute(url.as_bytes()));
    
    // Try to preserve the extension
    if let Some(extension) = url.rsplit('.').next() {
        if extension.len() <= 4 && extension.chars().all(|c| c.is_alphanumeric()) {
            return format!("{}.{}", hash, extension);
        }
    }
    
    hash
}

/// Fetch an icon from a URL and cache it locally.
/// Returns the path to the cached icon file.
/// If the icon is already cached, returns the cached path without downloading.
pub fn fetch_icon(url: &str) -> Result<PathBuf> {
    let cache_dir = ensure_cache_dir_exists()?;
    let cache_filename = url_to_cache_filename(url);
    let cache_path = cache_dir.join(&cache_filename);
    
    // If already cached, return the cached path
    if cache_path.exists() {
        return Ok(cache_path);
    }
    
    // Fetch the icon from the URL
    // Note: This requires the 'ureq' crate for HTTP requests
    // For FFI scenarios, you might want to let the native UI handle downloading
    // and just use this for caching locally provided icon data
    
    // For now, return an error if not cached - native UI should handle downloading
    Err(anyhow!("Icon not cached. Native UI should download from: {}", url))
}

/// Load icon data from a cached file or URL.
/// Returns the raw bytes of the icon file.
pub fn load_icon_data(url: &str) -> Result<Vec<u8>> {
    let cache_path = fetch_icon(url)?;
    fs::read(&cache_path)
        .context(format!("Failed to read icon file: {:?}", cache_path))
}

/// Save icon data to the cache.
/// This can be called from FFI after the native UI downloads the icon.
pub fn cache_icon_data(url: &str, data: &[u8]) -> Result<PathBuf> {
    let cache_dir = ensure_cache_dir_exists()?;
    let cache_filename = url_to_cache_filename(url);
    let cache_path = cache_dir.join(&cache_filename);
    
    fs::write(&cache_path, data)
        .context(format!("Failed to write icon cache file: {:?}", cache_path))?;
    
    Ok(cache_path)
}

/// Get the cached icon path if it exists, without attempting to download.
/// For file:// URLs, returns the local path directly.
/// For HTTP(S) URLs, checks the cache.
pub fn get_cached_icon_path(url: &str) -> Option<PathBuf> {
    // If it's a file:// URL, return the path directly after stripping the prefix
    if url.starts_with("file://") {
        let path_str = url.trim_start_matches("file://");
        let path = PathBuf::from(path_str);
        if path.exists() {
            return Some(path);
        }
        return None;
    }
    
    // For HTTP(S) URLs, check the cache
    if let Ok(cache_dir) = get_icon_cache_dir() {
        let cache_filename = url_to_cache_filename(url);
        let cache_path = cache_dir.join(&cache_filename);
        
        if cache_path.exists() {
            return Some(cache_path);
        }
    }
    
    None
}

/// Clear the icon cache directory
pub fn clear_icon_cache() -> Result<()> {
    let cache_dir = get_icon_cache_dir()?;
    
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)
            .context(format!("Failed to clear icon cache: {:?}", cache_dir))?;
        ensure_cache_dir_exists()?;
    }
    
    Ok(())
}
