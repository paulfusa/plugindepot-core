//! PluginDepot Core Library
//!
//! Cross-platform audio plugin manager core for VST2, VST3, AU, and AAX plugins.
//!
//! # Architecture
//!
//! This library is designed to be consumed via FFI by native UI frontends:
//! - **macOS**: SwiftUI app using static lib via Swift ↔ Rust FFI
//! - **Windows**: WPF app using DLL via C# ↔ Rust P/Invoke
//!
//! # Core Features Implemented
//!
//! ## Plugin Discovery (`registry` module)
//! - `scan_installed()` - Scan system for installed plugins
//! - `detect_orphaned_files()` - Find leftover files from uninstalled plugins
//! - Automatic discovery of related paths (presets, libraries, preferences)
//!
//! ## Plugin Management (`operations` module)
//! - `backup_plugin()` - Create timestamped backups of plugins and related files
//! - `uninstall_plugin()` - Safe uninstall with dry-run support
//! - `export_plugin()` - Package plugins for migration to another machine
//! - `import_plugin()` - Restore plugins from export packages (TODO)
//!
//! ## Data Structures (`plugin` module)
//! - `Plugin` - Basic plugin information (name, version, description)
//! - `InstalledPlugin` - Plugin with installation path and related files
//! - `RelatedPaths` - Discovered preset, library, and preference locations
//! - `Vendor` - Manufacturer information
//! - `License` - License key and activation information
//! - `PluginMetadata` - Complete plugin info with vendor and licensing
//!
//! # TODO: FFI Bindings
//!
//! Future work will add C-compatible FFI functions:
//! - `extern "C" fn plugindepot_scan_plugins() -> *const PluginList`
//! - `extern "C" fn plugindepot_backup_plugin() -> bool`
//! - `extern "C" fn plugindepot_uninstall_plugin() -> bool`
//! - `extern "C" fn plugindepot_free_*()` - Memory management functions
//! - Additional management functions with C ABI for cross-language interop

pub mod plugin;
pub mod registry;
pub mod operations;

pub use plugin::{Plugin, InstalledPlugin, PluginFormat, RelatedPaths, Vendor, License, PluginMetadata};