# PluginDepot Core - FFI Integration Guide

This guide explains how to integrate the Rust core with native UI frameworks (SwiftUI on macOS, WPF on Windows).

## Architecture Overview

```
┌─────────────────────────────────────────────┐
│  Native UI Layer (Swift/C#)                 │
│  - SwiftUI (macOS)                          │
│  - WPF (Windows)                            │
└────────────┬────────────────────────────────┘
             │ FFI calls
┌────────────▼────────────────────────────────┐
│  C FFI Interface (ffi.rs)                   │
│  - extern "C" functions                     │
│  - Memory management                        │
│  - Type conversion                          │
└────────────┬────────────────────────────────┘
             │
┌────────────▼────────────────────────────────┐
│  Rust Core (lib.rs)                         │
│  - Plugin scanning                          │
│  - File operations                          │
│  - Orphaned detection                       │
└─────────────────────────────────────────────┘
```

## Building

### For macOS (Static Library)

```bash
cargo build --release
```

Output: `target/release/libplugindepot_core.a`

### For Windows (DLL)

```bash
cargo build --release
```

Output: `target/release/plugindepot_core.dll` (when built on Windows)

## macOS Integration (SwiftUI)

### Step 1: Add Static Library to Xcode

1. Drag `libplugindepot_core.a` into your Xcode project
2. Add to "Link Binary With Libraries" in Build Phases

### Step 2: Create Bridging Header

Create `PluginDepot-Bridging-Header.h`:

```c
#import "plugindepot_core.h"
```

Add bridging header path in Build Settings:
```
Objective-C Bridging Header: PluginDepot-Bridging-Header.h
```

### Step 3: Use Swift Wrapper

See `examples/swift_example.swift` for complete wrapper implementation.

```swift
// Scan plugins
let plugins = PluginDepotCore.scanPlugins() ?? []

// Display in SwiftUI
List(plugins, id: \.id) { plugin in
    VStack(alignment: .leading) {
        Text(plugin.name).font(.headline)
        Text(plugin.format.displayName).font(.caption)
    }
}
```

## Windows Integration (WPF)

### Step 1: Add DLL to Project

1. Build the Rust project on Windows: `cargo build --release`
2. Copy `target/release/plugindepot_core.dll` to your project's output directory
3. Set "Copy to Output Directory" → "Copy if newer"

### Step 2: Use C# Wrapper

See `examples/csharp_example.cs` for complete wrapper implementation.

```csharp
// Scan plugins
var plugins = PluginDepotCore.ScanPlugins();

// Display in WPF
foreach (var plugin in plugins)
{
    Console.WriteLine($"{plugin.Name} - {plugin.FormatDisplayName}");
}
```

## Core Features Available via FFI

### 1. Plugin Scanning

**Function:** `plugindepot_scan_plugins()`
- Returns list of all installed plugins
- Automatically discovers related paths (presets, libraries)
- Cross-platform (AU on macOS, VST2/VST3/AAX on both)

### 2. Orphaned File Detection

**Function:** `plugindepot_detect_orphaned()`
- Finds leftover files from uninstalled plugins
- Returns list of paths that can be cleaned up

### 3. Backup Plugin

**Function:** `plugindepot_backup_plugin(list, index, backup_dir)`
- Creates timestamped backup of plugin and all related files
- Returns backup path

### 4. Uninstall Plugin

**Function:** `plugindepot_uninstall_plugin(list, index, dry_run)`
- Safe uninstall with dry-run preview
- Set `dry_run = 1` to preview files without deleting
- Set `dry_run = 0` to actually delete

### 5. Export for Migration

**Function:** `plugindepot_export_plugin(list, index, export_dir)`
- Package plugin for moving to another machine
- Creates portable export package

### 6. Enumerate Files

**Function:** `plugindepot_enumerate_files(list, index)`
- List all files associated with a plugin
- Useful for showing what will be affected by operations

## ⚠️ Memory Management Rules

**CRITICAL:** The Rust core allocates memory that **MUST** be freed by the caller.

### Always Free What You Allocate

```c
// C example
CPluginList* list = plugindepot_scan_plugins();
// ... use list ...
plugindepot_free_plugin_list(list);  // REQUIRED!

char* path = plugindepot_backup_plugin(list, 0, "/backups");
// ... use path ...
plugindepot_free_string(path);  // REQUIRED!
```

```swift
// Swift example - use defer
let listPtr = plugindepot_scan_plugins()
defer { plugindepot_free_plugin_list(listPtr) }
// ... use listPtr ...
```

```csharp
// C# example - use try/finally
IntPtr listPtr = plugindepot_scan_plugins();
try
{
    // ... use listPtr ...
}
finally
{
    plugindepot_free_plugin_list(listPtr);
}
```

### Free Functions

- `plugindepot_free_plugin_list()` - Free plugin list
- `plugindepot_free_plugin()` - Free individual plugin struct
- `plugindepot_free_path_list()` - Free path list
- `plugindepot_free_string()` - Free string returned by FFI

## UI Layer Responsibilities

The Rust core handles all heavy lifting. Your UI should:

1. **Call FFI functions** → Get data
2. **Display results** → Present to user
3. **Handle user input** → Confirmations, selections
4. **Show progress** → Long operations (backup, export)
5. **Platform-specific features:**
   - "Reveal in Finder" (macOS) → Use `NSWorkspace.shared.selectFile()`
   - "Open in Explorer" (Windows) → Use `Process.Start("explorer.exe")`

## Example Workflows

### Workflow 1: Scan and Display Plugins

```
User opens app
    ↓
UI: Call plugindepot_scan_plugins()
    ↓
UI: Display plugin list
    ↓
User clicks plugin
    ↓
UI: Show details (presets, libraries, paths)
```

### Workflow 2: Safe Uninstall with Preview

```
User selects plugin to uninstall
    ↓
UI: Call plugindepot_uninstall_plugin(dry_run=1)
    ↓
UI: Show "These files will be deleted" dialog
    ↓
User confirms
    ↓
UI: Call plugindepot_uninstall_plugin(dry_run=0)
    ↓
UI: Show success message
```

### Workflow 3: Backup Plugin

```
User selects plugin to backup
    ↓
UI: Call plugindepot_backup_plugin()
    ↓
UI: Show progress indicator
    ↓
Rust: Create backup
    ↓
UI: Show "Backup saved to: /path/to/backup"
```

## Additional Resources
- **C Header:** `include/plugindepot_core.h`
- **Swift Example:** `examples/swift_example.swift`
- **C# Example:** `examples/csharp_example.cs`

## Troubleshooting
### Swift: "Undefined symbols for architecture x86_64"
- Ensure `libplugindepot_core.a` is added to "Link Binary With Libraries"
- Check that bridging header path is correct
- Verify you're building for the correct architecture

### C#: "Unable to load DLL"
- Ensure `plugindepot_core.dll` is in the output directory
- Check that you're using the correct architecture (x64/x86)
- Verify DLL dependencies (use Dependency Walker)

### Memory leaks
- Always call corresponding `_free` functions
- Use `defer` (Swift) or `try/finally` (C#) patterns
- Never return pointers without documenting free requirements
