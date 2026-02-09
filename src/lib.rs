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
//! # TODO: FFI Bindings
//!
//! Future work will add C-compatible FFI functions:
//! - `extern "C" fn plugindepot_scan_plugins() -> *const PluginList`
//! - `extern "C" fn plugindepot_free_plugin_list(*mut PluginList)`
//! - Additional management functions with C ABI for cross-language interop

pub mod plugin;
pub mod registry;

pub use plugin::{Plugin, InstalledPlugin, PluginFormat};