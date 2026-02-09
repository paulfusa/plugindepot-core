use plugindepot_core::registry::scan_installed;

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
                    println!();
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }

    Ok(())
}