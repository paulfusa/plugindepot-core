//! FFI (Foreign Function Interface) bindings for cross-language interop.
//!
//! This module exposes the Rust core functionality through C-compatible functions
//! that can be called from Swift (macOS) and C# (Windows).
//!
//! # Memory Management
//!
//! - Rust allocates memory and returns pointers to Swift/C#
//! - The calling code MUST call the corresponding `_free` functions to prevent leaks
//! - Strings are null-terminated UTF-8
//!
//! # Usage from Swift (macOS)
//!
//! ```swift
//! let pluginsPtr = plugindepot_scan_plugins()
//! defer { plugindepot_free_plugin_list(pluginsPtr) }
//! 
//! let count = plugindepot_plugin_list_count(pluginsPtr)
//! for i in 0..<count {
//!     let plugin = plugindepot_plugin_list_get(pluginsPtr, i)
//!     // Use plugin data...
//! }
//! ```
//!
//! # Usage from C# (Windows)
//!
//! ```csharp
//! [DllImport("plugindepot_core.dll")]
//! private static extern IntPtr plugindepot_scan_plugins();
//! 
//! [DllImport("plugindepot_core.dll")]
//! private static extern void plugindepot_free_plugin_list(IntPtr list);
//! ```

use crate::registry::{scan_installed, detect_orphaned_files, enumerate_plugin_files};
use crate::operations::{backup_plugin, uninstall_plugin, export_plugin};
use crate::icons::{cache_icon_data, get_cached_icon_path, clear_icon_cache};
use crate::{InstalledPlugin, PluginFormat};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_int};
use std::path::PathBuf;
use std::ptr;
use std::slice;

// ============================================================================
// C-Compatible Types
// ============================================================================

/// Opaque handle to a list of plugins
#[repr(C)]
pub struct CPluginList {
    plugins: Vec<InstalledPlugin>,
}

/// C-compatible plugin information
#[repr(C)]
pub struct CPlugin {
    pub id: *mut c_char,
    pub name: *mut c_char,
    pub version: *mut c_char,
    pub description: *mut c_char,
    pub install_path: *mut c_char,
    pub format: c_int, // 0=VST2, 1=VST3, 2=AU, 3=AAX
    pub preset_count: c_int,
    pub library_count: c_int,
    pub preference_count: c_int,
    /// URL to the plugin's icon (null if not available)
    pub icon_url: *mut c_char,
}

/// C-compatible path list
#[repr(C)]
pub struct CPathList {
    paths: Vec<PathBuf>,
}

/// Result code for operations
#[repr(C)]
pub enum CResultCode {
    Success = 0,
    Error = 1,
}

// ============================================================================
// Plugin Scanning
// ============================================================================

/// Scan the system for installed plugins.
/// Returns an opaque handle to the plugin list.
/// Caller MUST call plugindepot_free_plugin_list() when done.
#[no_mangle]
pub extern "C" fn plugindepot_scan_plugins() -> *mut CPluginList {
    match scan_installed() {
        Ok(plugins) => {
            let list = Box::new(CPluginList { plugins });
            Box::into_raw(list)
        }
        Err(e) => {
            eprintln!("Error scanning plugins: {}", e);
            ptr::null_mut()
        }
    }
}

/// Get the number of plugins in a list.
#[no_mangle]
pub extern "C" fn plugindepot_plugin_list_count(list: *const CPluginList) -> c_int {
    if list.is_null() {
        return 0;
    }
    unsafe {
        (*list).plugins.len() as c_int
    }
}

/// Get plugin information at a specific index.
/// Returns a CPlugin struct. Caller MUST call plugindepot_free_plugin() when done.
#[no_mangle]
pub extern "C" fn plugindepot_plugin_list_get(list: *const CPluginList, index: c_int) -> *mut CPlugin {
    if list.is_null() {
        return ptr::null_mut();
    }
    
    unsafe {
        let plugins = &(*list).plugins;
        let idx = index as usize;
        
        if idx >= plugins.len() {
            return ptr::null_mut();
        }
        
        let plugin = &plugins[idx];
        
        // Convert to C-compatible struct
        let c_plugin = Box::new(CPlugin {
            id: string_to_c_char(&plugin.plugin.id),
            name: string_to_c_char(&plugin.plugin.name),
            version: string_to_c_char(&plugin.plugin.version),
            description: plugin.plugin.description.as_ref()
                .map(|s| string_to_c_char(s))
                .unwrap_or(ptr::null_mut()),
            install_path: string_to_c_char(&plugin.install_path.to_string_lossy()),
            format: format_to_int(&plugin.format),
            preset_count: plugin.related_paths.preset_locations.len() as c_int,
            library_count: plugin.related_paths.library_locations.len() as c_int,
            preference_count: plugin.related_paths.preference_files.len() as c_int,
            icon_url: plugin.plugin.icon_url.as_ref()
                .map(|s| string_to_c_char(s))
                .unwrap_or(ptr::null_mut()),
        });
        
        Box::into_raw(c_plugin)
    }
}

/// Free a plugin list returned by plugindepot_scan_plugins().
#[no_mangle]
pub extern "C" fn plugindepot_free_plugin_list(list: *mut CPluginList) {
    if !list.is_null() {
        unsafe {
            let _ = Box::from_raw(list);
        }
    }
}

/// Free a CPlugin struct returned by plugindepot_plugin_list_get().
#[no_mangle]
pub extern "C" fn plugindepot_free_plugin(plugin: *mut CPlugin) {
    if !plugin.is_null() {
        unsafe {
            let p = Box::from_raw(plugin);
            free_c_char(p.id);
            free_c_char(p.name);
            free_c_char(p.version);
            free_c_char(p.description);
            free_c_char(p.install_path);
            free_c_char(p.icon_url);
        }
    }
}

// ============================================================================
// Orphaned File Detection
// ============================================================================

/// Detect orphaned files from uninstalled plugins.
/// Returns an opaque handle to the path list.
/// Caller MUST call plugindepot_free_path_list() when done.
#[no_mangle]
pub extern "C" fn plugindepot_detect_orphaned() -> *mut CPathList {
    match detect_orphaned_files() {
        Ok(paths) => {
            let list = Box::new(CPathList { paths });
            Box::into_raw(list)
        }
        Err(e) => {
            eprintln!("Error detecting orphaned files: {}", e);
            ptr::null_mut()
        }
    }
}

/// Get the number of paths in a path list.
#[no_mangle]
pub extern "C" fn plugindepot_path_list_count(list: *const CPathList) -> c_int {
    if list.is_null() {
        return 0;
    }
    unsafe {
        (*list).paths.len() as c_int
    }
}

/// Get a path at a specific index.
/// Returns a null-terminated string. Caller MUST call plugindepot_free_string() when done.
#[no_mangle]
pub extern "C" fn plugindepot_path_list_get(list: *const CPathList, index: c_int) -> *mut c_char {
    if list.is_null() {
        return ptr::null_mut();
    }
    
    unsafe {
        let paths = &(*list).paths;
        let idx = index as usize;
        
        if idx >= paths.len() {
            return ptr::null_mut();
        }
        
        string_to_c_char(&paths[idx].to_string_lossy())
    }
}

/// Free a path list returned by plugindepot_detect_orphaned().
#[no_mangle]
pub extern "C" fn plugindepot_free_path_list(list: *mut CPathList) {
    if !list.is_null() {
        unsafe {
            let _ = Box::from_raw(list);
        }
    }
}

// ============================================================================
// Plugin Operations
// ============================================================================

/// Backup a plugin to the specified directory.
/// Returns the backup path on success, or null on error.
/// Caller MUST call plugindepot_free_string() when done.
#[no_mangle]
pub extern "C" fn plugindepot_backup_plugin(
    list: *const CPluginList,
    index: c_int,
    backup_dir: *const c_char,
) -> *mut c_char {
    if list.is_null() || backup_dir.is_null() {
        return ptr::null_mut();
    }
    
    unsafe {
        let plugins = &(*list).plugins;
        let idx = index as usize;
        
        if idx >= plugins.len() {
            return ptr::null_mut();
        }
        
        let plugin = &plugins[idx];
        let backup_dir_str = match CStr::from_ptr(backup_dir).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
        
        match backup_plugin(plugin, &PathBuf::from(backup_dir_str)) {
            Ok(path) => string_to_c_char(&path.to_string_lossy()),
            Err(e) => {
                eprintln!("Error backing up plugin: {}", e);
                ptr::null_mut()
            }
        }
    }
}

/// Uninstall a plugin. If dry_run is non-zero, only returns what would be deleted.
/// Returns a path list of deleted files.
/// Caller MUST call plugindepot_free_path_list() when done.
#[no_mangle]
pub extern "C" fn plugindepot_uninstall_plugin(
    list: *const CPluginList,
    index: c_int,
    dry_run: c_int,
) -> *mut CPathList {
    if list.is_null() {
        return ptr::null_mut();
    }
    
    unsafe {
        let plugins = &(*list).plugins;
        let idx = index as usize;
        
        if idx >= plugins.len() {
            return ptr::null_mut();
        }
        
        let plugin = &plugins[idx];
        match uninstall_plugin(plugin, dry_run != 0) {
            Ok(paths) => {
                let list = Box::new(CPathList { paths });
                Box::into_raw(list)
            }
            Err(e) => {
                eprintln!("Error uninstalling plugin: {}", e);
                ptr::null_mut()
            }
        }
    }
}

/// Export a plugin for migration to another machine.
/// Returns the export path on success, or null on error.
/// Caller MUST call plugindepot_free_string() when done.
#[no_mangle]
pub extern "C" fn plugindepot_export_plugin(
    list: *const CPluginList,
    index: c_int,
    export_dir: *const c_char,
) -> *mut c_char {
    if list.is_null() || export_dir.is_null() {
        return ptr::null_mut();
    }
    
    unsafe {
        let plugins = &(*list).plugins;
        let idx = index as usize;
        
        if idx >= plugins.len() {
            return ptr::null_mut();
        }
        
        let plugin = &plugins[idx];
        let export_dir_str = match CStr::from_ptr(export_dir).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
        
        match export_plugin(plugin, &PathBuf::from(export_dir_str)) {
            Ok(path) => string_to_c_char(&path.to_string_lossy()),
            Err(e) => {
                eprintln!("Error exporting plugin: {}", e);
                ptr::null_mut()
            }
        }
    }
}

/// Enumerate all files associated with a plugin.
/// Returns a path list.
/// Caller MUST call plugindepot_free_path_list() when done.
#[no_mangle]
pub extern "C" fn plugindepot_enumerate_files(
    list: *const CPluginList,
    index: c_int,
) -> *mut CPathList {
    if list.is_null() {
        return ptr::null_mut();
    }
    
    unsafe {
        let plugins = &(*list).plugins;
        let idx = index as usize;
        
        if idx >= plugins.len() {
            return ptr::null_mut();
        }
        
        let plugin = &plugins[idx];
        match enumerate_plugin_files(plugin) {
            Ok(paths) => {
                let list = Box::new(CPathList { paths });
                Box::into_raw(list)
            }
            Err(e) => {
                eprintln!("Error enumerating files: {}", e);
                ptr::null_mut()
            }
        }
    }
}

// ============================================================================
// Icon Management
// ============================================================================

/// Cache icon data for a given URL.
/// This should be called by the native UI after downloading the icon.
/// Returns the cached file path on success, or null on error.
/// Caller MUST call plugindepot_free_string() when done.
#[no_mangle]
pub extern "C" fn plugindepot_cache_icon(
    icon_url: *const c_char,
    data: *const u8,
    data_length: c_int,
) -> *mut c_char {
    if icon_url.is_null() || data.is_null() || data_length <= 0 {
        return ptr::null_mut();
    }
    
    unsafe {
        let url = match CStr::from_ptr(icon_url).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
        
        let data_slice = slice::from_raw_parts(data, data_length as usize);
        
        match cache_icon_data(url, data_slice) {
            Ok(path) => string_to_c_char(&path.to_string_lossy()),
            Err(e) => {
                eprintln!("Error caching icon: {}", e);
                ptr::null_mut()
            }
        }
    }
}

/// Get the cached icon path for a URL, if it exists.
/// Returns null if the icon is not cached.
/// Caller MUST call plugindepot_free_string() when done.
#[no_mangle]
pub extern "C" fn plugindepot_get_cached_icon_path(icon_url: *const c_char) -> *mut c_char {
    if icon_url.is_null() {
        return ptr::null_mut();
    }
    
    unsafe {
        let url = match CStr::from_ptr(icon_url).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };
        
        match get_cached_icon_path(url) {
            Some(path) => string_to_c_char(&path.to_string_lossy()),
            None => ptr::null_mut(),
        }
    }
}

/// Clear all cached icons.
/// Returns 0 on success, 1 on error.
#[no_mangle]
pub extern "C" fn plugindepot_clear_icon_cache() -> c_int {
    match clear_icon_cache() {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("Error clearing icon cache: {}", e);
            1
        }
    }
}

// ============================================================================
// String Management
// ============================================================================

/// Free a string returned by FFI functions.
#[no_mangle]
pub extern "C" fn plugindepot_free_string(s: *mut c_char) {
    free_c_char(s);
}

// ============================================================================
// Helper Functions
// ============================================================================

fn string_to_c_char(s: &str) -> *mut c_char {
    match CString::new(s) {
        Ok(c_str) => c_str.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

fn free_c_char(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

fn format_to_int(format: &PluginFormat) -> c_int {
    match format {
        PluginFormat::VST2 => 0,
        PluginFormat::VST3 => 1,
        PluginFormat::AU => 2,
        PluginFormat::AAX => 3,
    }
}
