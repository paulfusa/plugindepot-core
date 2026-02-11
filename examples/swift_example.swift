// Swift Example: Using PluginDepot Core from SwiftUI
//
// 1. Add libplugindepot_core.a to your Xcode project
// 2. Create a bridging header with #include "plugindepot_core.h"
// 3. Use this wrapper class

import Foundation

/// Swift wrapper for PluginDepot Core functionality
class PluginDepotCore {
    
    // MARK: - Types
    
    struct Plugin {
        let id: String
        let name: String
        let version: String
        let description: String?
        let installPath: String
        let format: PluginFormat
        let presetCount: Int
        let libraryCount: Int
        let preferenceCount: Int
        let iconUrl: String?  // URL to plugin icon
    }
    
    enum PluginFormat: Int32 {
        case vst2 = 0
        case vst3 = 1
        case au = 2
        case aax = 3
        
        var displayName: String {
            switch self {
            case .vst2: return "VST2"
            case .vst3: return "VST3"
            case .au: return "Audio Unit"
            case .aax: return "AAX"
            }
        }
    }
    
    // MARK: - Plugin Scanning
    
    /// Scan the system for installed plugins
    static func scanPlugins() -> [Plugin]? {
        guard let listPtr = plugindepot_scan_plugins() else {
            return nil
        }
        defer { plugindepot_free_plugin_list(listPtr) }
        
        let count = Int(plugindepot_plugin_list_count(listPtr))
        var plugins: [Plugin] = []
        
        for i in 0..<count {
            if let cPlugin = plugindepot_plugin_list_get(listPtr, Int32(i)) {
                defer { plugindepot_free_plugin(cPlugin) }
                
                let plugin = Plugin(
                    id: String(cString: cPlugin.pointee.id),
                    name: String(cString: cPlugin.pointee.name),
                    version: String(cString: cPlugin.pointee.version),
                    description: cPlugin.pointee.description != nil ? String(cString: cPlugin.pointee.description) : nil,
                    installPath: String(cString: cPlugin.pointee.install_path),
                    format: PluginFormat(rawValue: cPlugin.pointee.format) ?? .vst2,
                    presetCount: Int(cPlugin.pointee.preset_count),
                    libraryCount: Int(cPlugin.pointee.library_count),
                    preferenceCount: Int(cPlugin.pointee.preference_count),
                    iconUrl: cPlugin.pointee.icon_url != nil ? String(cString: cPlugin.pointee.icon_url) : nil
                )
                plugins.append(plugin)
            }
        }
        
        return plugins
    }
    
    // MARK: - Orphaned Files
    
    /// Detect orphaned files from uninstalled plugins
    static func detectOrphanedFiles() -> [String]? {
        guard let listPtr = plugindepot_detect_orphaned() else {
            return nil
        }
        defer { plugindepot_free_path_list(listPtr) }
        
        let count = Int(plugindepot_path_list_count(listPtr))
        var paths: [String] = []
        
        for i in 0..<count {
            if let pathPtr = plugindepot_path_list_get(listPtr, Int32(i)) {
                defer { plugindepot_free_string(pathPtr) }
                paths.append(String(cString: pathPtr))
            }
        }
        
        return paths
    }
    
    // MARK: - Plugin Operations
    
    /// Backup a plugin to the specified directory
    static func backupPlugin(at index: Int, listPtr: OpaquePointer, to backupDir: String) -> String? {
        guard let resultPtr = plugindepot_backup_plugin(listPtr, Int32(index), backupDir) else {
            return nil
        }
        defer { plugindepot_free_string(resultPtr) }
        return String(cString: resultPtr)
    }
    
    /// Preview what files would be deleted (dry run)
    static func previewUninstall(at index: Int, listPtr: OpaquePointer) -> [String]? {
        guard let pathListPtr = plugindepot_uninstall_plugin(listPtr, Int32(index), 1) else {
            return nil
        }
        defer { plugindepot_free_path_list(pathListPtr) }
        
        let count = Int(plugindepot_path_list_count(pathListPtr))
        var paths: [String] = []
        
        for i in 0..<count {
            if let pathPtr = plugindepot_path_list_get(pathListPtr, Int32(i)) {
                defer { plugindepot_free_string(pathPtr) }
                paths.append(String(cString: pathPtr))
            }
        }
        
        return paths
    }
    
    /// Uninstall a plugin
    static func uninstallPlugin(at index: Int, listPtr: OpaquePointer) -> Bool {
        guard let pathListPtr = plugindepot_uninstall_plugin(listPtr, Int32(index), 0) else {
            return false
        }
        plugindepot_free_path_list(pathListPtr)
        return true
    }
    
    /// Export a plugin for migration
    static func exportPlugin(at index: Int, listPtr: OpaquePointer, to exportDir: String) -> String? {
        guard let resultPtr = plugindepot_export_plugin(listPtr, Int32(index), exportDir) else {
            return nil
        }
        defer { plugindepot_free_string(resultPtr) }
        return String(cString: resultPtr)
    }
    
    // MARK: - Icon Management
    
    /// Cache icon data after downloading it
    static func cacheIcon(url: String, data: Data) -> String? {
        return data.withUnsafeBytes { bufferPtr in
            guard let baseAddress = bufferPtr.baseAddress else { return nil }
            guard let resultPtr = plugindepot_cache_icon(
                url,
                baseAddress.assumingMemoryBound(to: UInt8.self),
                Int32(data.count)
            ) else {
                return nil
            }
            defer { plugindepot_free_string(resultPtr) }
            return String(cString: resultPtr)
        }
    }
    
    /// Get the cached icon path if it exists
    static func getCachedIconPath(url: String) -> String? {
        guard let resultPtr = plugindepot_get_cached_icon_path(url) else {
            return nil
        }
        defer { plugindepot_free_string(resultPtr) }
        return String(cString: resultPtr)
    }
    
    /// Clear all cached icons
    static func clearIconCache() -> Bool {
        return plugindepot_clear_icon_cache() == 0
    }
}

// MARK: - Usage Example

/*
// In your SwiftUI view:

struct PluginListView: View {
    @State private var plugins: [PluginDepotCore.Plugin] = []
    @State private var isLoading = false
    
    var body: some View {
        List(plugins, id: \.id) { plugin in
            VStack(alignment: .leading) {
                Text(plugin.name).font(.headline)
                Text(plugin.format.displayName).font(.caption)
                Text(plugin.installPath).font(.caption2).foregroundColor(.gray)
            }
        }
        .onAppear {
            loadPlugins()
        }
    }
    
    func loadPlugins() {
        isLoading = true
        DispatchQueue.global(qos: .userInitiated).async {
            let scanned = PluginDepotCore.scanPlugins() ?? []
            DispatchQueue.main.async {
                self.plugins = scanned
                self.isLoading = false
            }
        }
    }
}
*/
