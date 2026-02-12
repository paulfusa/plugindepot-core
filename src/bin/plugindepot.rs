use plugindepot_core::registry::{scan_installed, detect_orphaned_files};

fn main() -> anyhow::Result<()> {
    let plugins = scan_installed();

    match plugins {
        Ok(list) => {
            println!("Installed plugins: {}\n", list.len());
            
            if !list.is_empty() {
                for plugin in &list {
                    println!("  [{:?}] {}", plugin.format, plugin.plugin.name);
                    println!("    Path: {}", plugin.install_path.display());
                    if let Some(desc) = &plugin.plugin.description {
                        println!("    {}", desc);
                    }
                    
                    // Show icon if available
                    if let Some(icon_url) = &plugin.plugin.icon_url {
                        println!("    Icon: {}", icon_url);
                    } else {
                        println!("    Icon: Not found");
                    }
                    
                    // Show related paths if found
                    if !plugin.related_paths.preset_locations.is_empty() {
                        println!("    Presets: {} location(s)", plugin.related_paths.preset_locations.len());
                    }
                    if !plugin.related_paths.library_locations.is_empty() {
                        println!("    Libraries: {} location(s)", plugin.related_paths.library_locations.len());
                    }
                    if !plugin.related_paths.preference_files.is_empty() {
                        println!("    Preferences: {} file(s)", plugin.related_paths.preference_files.len());
                    }
                    
                    println!();
                }
                
                // Demonstrate orphaned file detection
                println!("\n--- Checking for orphaned files ---");
                match detect_orphaned_files() {
                    Ok(orphaned) => {
                        if orphaned.is_empty() {
                            println!("No orphaned files detected.");
                        } else {
                            println!("Found {} potentially orphaned file(s):", orphaned.len());
                            for path in orphaned.iter().take(10) {
                                println!("  - {}", path.display());
                            }
                            if orphaned.len() > 10 {
                                println!("  ... and {} more", orphaned.len() - 10);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to detect orphaned files: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }

    Ok(())
}