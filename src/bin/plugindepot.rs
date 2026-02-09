use plugindepot_core::registry::scan_installed;

fn main() -> anyhow::Result<()> {
    let plugins = scan_installed();

    match plugins {
        Ok(list) => {
            println!("Installed plugins: {}", list.len());
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }

    Ok(())
}