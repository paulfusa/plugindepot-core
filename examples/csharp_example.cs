// C# Example: Using PluginDepot Core from WPF
//
// 1. Build Rust as DLL: cargo build --release
// 2. Copy plugindepot_core.dll to your project's output directory
// 3. Use this wrapper class

using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

namespace PluginDepot
{
    /// <summary>
    /// C# wrapper for PluginDepot Core functionality
    /// </summary>
    public class PluginDepotCore
    {
        private const string DLL_NAME = "plugindepot_core.dll";

        // ====================================================================
        // P/Invoke Declarations
        // ====================================================================

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr plugindepot_scan_plugins();

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int plugindepot_plugin_list_count(IntPtr list);

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr plugindepot_plugin_list_get(IntPtr list, int index);

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern void plugindepot_free_plugin_list(IntPtr list);

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern void plugindepot_free_plugin(IntPtr plugin);

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr plugindepot_detect_orphaned();

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern int plugindepot_path_list_count(IntPtr list);

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr plugindepot_path_list_get(IntPtr list, int index);

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern void plugindepot_free_path_list(IntPtr list);

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr plugindepot_backup_plugin(IntPtr list, int index, 
            [MarshalAs(UnmanagedType.LPStr)] string backupDir);

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr plugindepot_uninstall_plugin(IntPtr list, int index, int dryRun);

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern IntPtr plugindepot_export_plugin(IntPtr list, int index,
            [MarshalAs(UnmanagedType.LPStr)] string exportDir);

        [DllImport(DLL_NAME, CallingConvention = CallingConvention.Cdecl)]
        private static extern void plugindepot_free_string(IntPtr str);

        // ====================================================================
        // Types
        // ====================================================================

        [StructLayout(LayoutKind.Sequential)]
        private struct CPlugin
        {
            public IntPtr id;
            public IntPtr name;
            public IntPtr version;
            public IntPtr description;
            public IntPtr install_path;
            public int format;
            public int preset_count;
            public int library_count;
            public int preference_count;
        }

        public enum PluginFormat
        {
            VST2 = 0,
            VST3 = 1,
            AU = 2,
            AAX = 3
        }

        public class Plugin
        {
            public string Id { get; set; }
            public string Name { get; set; }
            public string Version { get; set; }
            public string Description { get; set; }
            public string InstallPath { get; set; }
            public PluginFormat Format { get; set; }
            public int PresetCount { get; set; }
            public int LibraryCount { get; set; }
            public int PreferenceCount { get; set; }

            public string FormatDisplayName => Format switch
            {
                PluginFormat.VST2 => "VST2",
                PluginFormat.VST3 => "VST3",
                PluginFormat.AU => "Audio Unit",
                PluginFormat.AAX => "AAX",
                _ => "Unknown"
            };
        }

        // ====================================================================
        // Public API
        // ====================================================================

        /// <summary>
        /// Scan the system for installed plugins
        /// </summary>
        public static List<Plugin> ScanPlugins()
        {
            IntPtr listPtr = plugindepot_scan_plugins();
            if (listPtr == IntPtr.Zero)
                return null;

            try
            {
                int count = plugindepot_plugin_list_count(listPtr);
                var plugins = new List<Plugin>(count);

                for (int i = 0; i < count; i++)
                {
                    IntPtr cPluginPtr = plugindepot_plugin_list_get(listPtr, i);
                    if (cPluginPtr == IntPtr.Zero)
                        continue;

                    try
                    {
                        var cPlugin = Marshal.PtrToStructure<CPlugin>(cPluginPtr);
                        
                        plugins.Add(new Plugin
                        {
                            Id = Marshal.PtrToStringAnsi(cPlugin.id),
                            Name = Marshal.PtrToStringAnsi(cPlugin.name),
                            Version = Marshal.PtrToStringAnsi(cPlugin.version),
                            Description = cPlugin.description != IntPtr.Zero 
                                ? Marshal.PtrToStringAnsi(cPlugin.description) 
                                : null,
                            InstallPath = Marshal.PtrToStringAnsi(cPlugin.install_path),
                            Format = (PluginFormat)cPlugin.format,
                            PresetCount = cPlugin.preset_count,
                            LibraryCount = cPlugin.library_count,
                            PreferenceCount = cPlugin.preference_count
                        });
                    }
                    finally
                    {
                        plugindepot_free_plugin(cPluginPtr);
                    }
                }

                return plugins;
            }
            finally
            {
                plugindepot_free_plugin_list(listPtr);
            }
        }

        /// <summary>
        /// Detect orphaned files from uninstalled plugins
        /// </summary>
        public static List<string> DetectOrphanedFiles()
        {
            IntPtr listPtr = plugindepot_detect_orphaned();
            if (listPtr == IntPtr.Zero)
                return null;

            try
            {
                int count = plugindepot_path_list_count(listPtr);
                var paths = new List<string>(count);

                for (int i = 0; i < count; i++)
                {
                    IntPtr pathPtr = plugindepot_path_list_get(listPtr, i);
                    if (pathPtr != IntPtr.Zero)
                    {
                        try
                        {
                            paths.Add(Marshal.PtrToStringAnsi(pathPtr));
                        }
                        finally
                        {
                            plugindepot_free_string(pathPtr);
                        }
                    }
                }

                return paths;
            }
            finally
            {
                plugindepot_free_path_list(listPtr);
            }
        }

        /// <summary>
        /// Backup a plugin to the specified directory
        /// </summary>
        public static string BackupPlugin(IntPtr listPtr, int index, string backupDir)
        {
            IntPtr resultPtr = plugindepot_backup_plugin(listPtr, index, backupDir);
            if (resultPtr == IntPtr.Zero)
                return null;

            try
            {
                return Marshal.PtrToStringAnsi(resultPtr);
            }
            finally
            {
                plugindepot_free_string(resultPtr);
            }
        }

        /// <summary>
        /// Preview what files would be deleted (dry run)
        /// </summary>
        public static List<string> PreviewUninstall(IntPtr listPtr, int index)
        {
            IntPtr pathListPtr = plugindepot_uninstall_plugin(listPtr, index, 1);
            if (pathListPtr == IntPtr.Zero)
                return null;

            try
            {
                int count = plugindepot_path_list_count(pathListPtr);
                var paths = new List<string>(count);

                for (int i = 0; i < count; i++)
                {
                    IntPtr pathPtr = plugindepot_path_list_get(pathListPtr, i);
                    if (pathPtr != IntPtr.Zero)
                    {
                        try
                        {
                            paths.Add(Marshal.PtrToStringAnsi(pathPtr));
                        }
                        finally
                        {
                            plugindepot_free_string(pathPtr);
                        }
                    }
                }

                return paths;
            }
            finally
            {
                plugindepot_free_path_list(pathListPtr);
            }
        }

        /// <summary>
        /// Uninstall a plugin
        /// </summary>
        public static bool UninstallPlugin(IntPtr listPtr, int index)
        {
            IntPtr pathListPtr = plugindepot_uninstall_plugin(listPtr, index, 0);
            if (pathListPtr == IntPtr.Zero)
                return false;

            plugindepot_free_path_list(pathListPtr);
            return true;
        }

        /// <summary>
        /// Export a plugin for migration
        /// </summary>
        public static string ExportPlugin(IntPtr listPtr, int index, string exportDir)
        {
            IntPtr resultPtr = plugindepot_export_plugin(listPtr, index, exportDir);
            if (resultPtr == IntPtr.Zero)
                return null;

            try
            {
                return Marshal.PtrToStringAnsi(resultPtr);
            }
            finally
            {
                plugindepot_free_string(resultPtr);
            }
        }
    }
}

// ====================================================================
// Usage Example (WPF ViewModel)
// ====================================================================

/*
using System.Collections.ObjectModel;
using System.Windows.Input;

public class PluginListViewModel : ViewModelBase
{
    public ObservableCollection<PluginDepot.Plugin> Plugins { get; }
    public ICommand RefreshCommand { get; }
    
    private bool _isLoading;
    public bool IsLoading
    {
        get => _isLoading;
        set => SetProperty(ref _isLoading, value);
    }

    public PluginListViewModel()
    {
        Plugins = new ObservableCollection<PluginDepot.Plugin>();
        RefreshCommand = new RelayCommand(LoadPlugins);
    }

    private async void LoadPlugins()
    {
        IsLoading = true;
        
        var plugins = await Task.Run(() => PluginDepotCore.ScanPlugins());
        
        Plugins.Clear();
        if (plugins != null)
        {
            foreach (var plugin in plugins)
            {
                Plugins.Add(plugin);
            }
        }
        
        IsLoading = false;
    }
}
*/
