/*
 * PluginDepot Core - C Header for FFI
 * 
 * This header can be used directly from C, or adapted for:
 * - Swift via bridging header
 * - C# via P/Invoke declarations
 */

#ifndef PLUGINDEPOT_CORE_H
#define PLUGINDEPOT_CORE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ============================================================================
 * Opaque Types
 * ============================================================================ */

typedef struct CPluginList CPluginList;
typedef struct CPathList CPathList;

/* ============================================================================
 * Data Structures
 * ============================================================================ */

typedef struct {
    char* id;
    char* name;
    char* version;
    char* description;      /* May be NULL */
    char* install_path;
    int32_t format;         /* 0=VST2, 1=VST3, 2=AU, 3=AAX */
    int32_t preset_count;
    int32_t library_count;
    int32_t preference_count;
    char* icon_url;         /* URL to plugin icon. May be NULL. */
} CPlugin;

typedef enum {
    CResultSuccess = 0,
    CResultError = 1
} CResultCode;

/* ============================================================================
 * Plugin Scanning
 * ============================================================================ */

/**
 * Scan the system for installed plugins.
 * @return Opaque handle to plugin list. Caller must call plugindepot_free_plugin_list().
 */
CPluginList* plugindepot_scan_plugins(void);

/**
 * Get the number of plugins in a list.
 * @param list Plugin list handle
 * @return Number of plugins, or 0 if list is NULL
 */
int32_t plugindepot_plugin_list_count(const CPluginList* list);

/**
 * Get plugin information at a specific index.
 * @param list Plugin list handle
 * @param index Zero-based index
 * @return Plugin structure. Caller must call plugindepot_free_plugin().
 */
CPlugin* plugindepot_plugin_list_get(const CPluginList* list, int32_t index);

/**
 * Free a plugin list.
 * @param list Plugin list handle (may be NULL)
 */
void plugindepot_free_plugin_list(CPluginList* list);

/**
 * Free a CPlugin structure.
 * @param plugin Plugin structure (may be NULL)
 */
void plugindepot_free_plugin(CPlugin* plugin);

/* ============================================================================
 * Orphaned File Detection
 * ============================================================================ */

/**
 * Detect orphaned files from uninstalled plugins.
 * @return Opaque handle to path list. Caller must call plugindepot_free_path_list().
 */
CPathList* plugindepot_detect_orphaned(void);

/**
 * Get the number of paths in a path list.
 * @param list Path list handle
 * @return Number of paths, or 0 if list is NULL
 */
int32_t plugindepot_path_list_count(const CPathList* list);

/**
 * Get a path at a specific index.
 * @param list Path list handle
 * @param index Zero-based index
 * @return Null-terminated string. Caller must call plugindepot_free_string().
 */
char* plugindepot_path_list_get(const CPathList* list, int32_t index);

/**
 * Free a path list.
 * @param list Path list handle (may be NULL)
 */
void plugindepot_free_path_list(CPathList* list);

/* ============================================================================
 * Plugin Operations
 * ============================================================================ */

/**
 * Backup a plugin to the specified directory.
 * @param list Plugin list handle
 * @param index Plugin index
 * @param backup_dir Target directory path (null-terminated string)
 * @return Backup path on success, or NULL on error. Caller must call plugindepot_free_string().
 */
char* plugindepot_backup_plugin(const CPluginList* list, int32_t index, const char* backup_dir);

/**
 * Uninstall a plugin.
 * @param list Plugin list handle
 * @param index Plugin index
 * @param dry_run If non-zero, only returns what would be deleted without actually deleting
 * @return Path list of deleted (or would-be deleted) files. Caller must call plugindepot_free_path_list().
 */
CPathList* plugindepot_uninstall_plugin(const CPluginList* list, int32_t index, int32_t dry_run);

/**
 * Export a plugin for migration to another machine.
 * @param list Plugin list handle
 * @param index Plugin index
 * @param export_dir Target directory path (null-terminated string)
 * @return Export path on success, or NULL on error. Caller must call plugindepot_free_string().
 */
char* plugindepot_export_plugin(const CPluginList* list, int32_t index, const char* export_dir);

/**
 * Enumerate all files associated with a plugin.
 * @param list Plugin list handle
 * @param index Plugin index
 * @return Path list. Caller must call plugindepot_free_path_list().
 */
CPathList* plugindepot_enumerate_files(const CPluginList* list, int32_t index);

/* ============================================================================
 * Icon Management
 * ============================================================================ */

/**
 * Cache icon data for a given URL.
 * This should be called by the native UI after downloading the icon.
 * @param icon_url Icon URL (null-terminated string)
 * @param data Raw icon data bytes
 * @param data_length Length of data in bytes
 * @return Cached file path on success, or NULL on error. Caller must call plugindepot_free_string().
 */
char* plugindepot_cache_icon(const char* icon_url, const uint8_t* data, int32_t data_length);

/**
 * Get the cached icon path for a URL, if it exists.
 * @param icon_url Icon URL (null-terminated string)
 * @return Cached file path, or NULL if not cached. Caller must call plugindepot_free_string().
 */
char* plugindepot_get_cached_icon_path(const char* icon_url);

/**
 * Clear all cached icons.
 * @return 0 on success, 1 on error
 */
int32_t plugindepot_clear_icon_cache(void);

/* ============================================================================
 * Memory Management
 * ============================================================================ */

/**
 * Free a string returned by FFI functions.
 * @param s String pointer (may be NULL)
 */
void plugindepot_free_string(char* s);

#ifdef __cplusplus
}
#endif

#endif /* PLUGINDEPOT_CORE_H */
