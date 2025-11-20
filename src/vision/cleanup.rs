use std::fs;
use std::path::Path;

pub fn cleanup_temp_files() {
    if let Err(e) = cleanup_temp_directory() {
        eprintln!("Warning: Failed to cleanup temp directory: {}", e);
    }
}

fn cleanup_temp_directory() -> std::io::Result<()> {
    let temp_dir = Path::new("temp");

    if !temp_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(temp_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "jpg" || ext == "jpeg" || ext == "png" {
                    fs::remove_file(&path)?;
                }
            }
        }
    }

    Ok(())
}

pub fn ensure_temp_dir() -> std::io::Result<()> {
    fs::create_dir_all("temp")
}
