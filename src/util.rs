use std::{fs::{self, File}, path::PathBuf, env, process::Command, io::Write as _};

use anyhow::anyhow;

pub fn fetch_document(file_name: &str) -> anyhow::Result<serde_json::Value> {
    let doc = fs::read_to_string(file_name)?;
    let doc: serde_json::Value = serde_yaml::from_str(&doc)?;

    Ok(doc)
}

pub fn save_doc(file_name: &str, value: serde_json::Value) -> anyhow::Result<()> {
    let mut path = PathBuf::from(file_name);

    let stem = path.file_stem().ok_or_else(|| anyhow!("File Stem not found"))?.to_string_lossy();
    let extention = path.extension().ok_or_else(|| anyhow!("File Extension not found"))?.to_string_lossy();
    path.set_file_name(format!("new_{stem}.{extention}"));

    let mut save_file = File::create(&path)?;

    let data = serde_yaml::to_string(&value)?;
    save_file.write_all(data.as_bytes())?;

    Ok(())
}

pub fn editor(value: serde_json::Value) -> anyhow::Result<serde_json::Value> {
    let editor = env::var("EDITOR")?;
    let mut file_path = env::temp_dir();
    file_path.push("editable.json");
    let mut new_file = File::create(&file_path)?;
    new_file.write_all(serde_json::to_string_pretty(&value)?.as_bytes())?;

    Command::new(editor)
        .arg(&file_path)
        .status()?;

    let new_value = fs::read_to_string(&file_path)?;
    let result = serde_json::from_str(&new_value)?;

    Ok(result)
}

