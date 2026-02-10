# PluginDepot Core

Cross-platform audio plugin manager core written in Rust. Manages VST2, VST3, Audio Units (AU), and AAX plugins with full backup, migration, and uninstall capabilities.

## Features

- **Plugin Discovery** - Automatically scan system for installed plugins
- **Related File Detection** - Find presets, libraries, and support files
- **Safe Uninstall** - Preview files before deletion, clean removal
- **Backup & Restore** - Create timestamped backups of plugins and data
- **Migration Support** - Export/import plugins between machines
- **Orphaned File Detection** - Find leftovers from old uninstalls
- **Cross-Platform** - macOS and Windows support
- **FFI-Ready** - C-compatible interface for Swift and C# integration

## Architecture
```
┌──────────────────────────────────────────┐
│  Native UI (SwiftUI/WPF)                 │
│  - User interface                        │
│  - Platform-specific features            │
└───────────────┬──────────────────────────┘
                │ FFI
┌───────────────▼──────────────────────────┐
│  Rust Core (This Library)               │
│  - Plugin scanning & management          │
│  - File operations                       │
│  - Cross-platform logic                  │
└──────────────────────────────────────────┘
```

## Testing
### Build
```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run CLI tool
cargo run --release
```

### Test
```bash
cargo test
cargo run --release  # See scanned plugins
```

## Core Modules
### `plugin.rs` - Data Structures
- `Plugin` - Basic plugin info
- `InstalledPlugin` - Plugin with installation details
- `RelatedPaths` - Discovered related files/folders
- `PluginMetadata` - Complete metadata with vendor/license
- `PluginFormat` - Enum for VST2/VST3/AU/AAX

### `registry.rs` - Discovery & Scanning
- `scan_installed()` - Scan all plugin directories
- `detect_orphaned_files()` - Find leftover files
- `discover_related_paths()` - Find presets/libraries
- `enumerate_plugin_files()` - List all plugin files

### `operations.rs` - Management
- `backup_plugin()` - Create backup with manifest
- `uninstall_plugin()` - Safe removal (with dry-run)
- `export_plugin()` - Package for migration
- `import_plugin()` - Restore from package (TODO)

### `ffi.rs` - Foreign Function Interface
- C-compatible functions for Swift/C# integration
- Memory-safe string handling
- Opaque pointer types for data structures
- See `FFI_GUIDE.md` for complete documentation

## Usage 
See `FFI_GUIDE.md` & `examples/` directory for complete integration examples.


## Future Ideas
- Add support for specific plugins such as *Waves* where their directories are unique or scattered all over
- Parse plugin bundle metadata (Info.plist, version info)
- Plugin → Vendor → License mapping
- Import from migration package
- Plugin validation/verification
- Duplicate detection


## Platform-Specific Directories 
### macOS
- AU: `/Library/Audio/Plug-Ins/Components/`, `~/Library/Audio/Plug-Ins/Components/`
- VST2: `/Library/Audio/Plug-Ins/VST/`, `~/Library/Audio/Plug-Ins/VST/`
- VST3: `/Library/Audio/Plug-Ins/VST3/`, `~/Library/Audio/Plug-Ins/VST3/`
- AAX: `/Library/Application Support/Avid/Audio/Plug-Ins/`

### Windows
- VST2: `C:\Program Files\VSTPlugins\`, `C:\Program Files\Steinberg\VSTPlugins\`
- VST3: `C:\Program Files\Common Files\VST3\`
- AAX: `C:\Program Files\Common Files\Avid\Audio\Plug-Ins\`

### Plugin Specific Directories
- WIP

## License
This project is source-available for review purposes only.
It is not open source at this time.

## ⚠️ AI Notice
I will admit majority of this core was made with Github CoPilot. I don't have much Rust experience as of currently and this is more of a vibe project, and want to get a working prototype soon. However if this project does become successful and useful for users, I am open to rework the core for efficiency and adding more features.

