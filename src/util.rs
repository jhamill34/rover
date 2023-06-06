//!

use std::{
    env,
    fs::{self, File},
    io::Write as _,
    path::PathBuf,
    process::Command,
};

use anyhow::{anyhow, bail};

///
pub fn fetch_document(file_name: &str) -> anyhow::Result<serde_json::Value> {
    let doc = fs::read_to_string(file_name)?;

    let extention = PathBuf::from(file_name);
    let extention = extention
        .extension()
        .ok_or_else(|| anyhow!("File Extension not found"))?
        .to_string_lossy();

    match extention.as_ref() {
        "yaml" | "yml" => {
            let doc = serde_yaml::from_str(&doc)?;
            Ok(doc)

        }
        "json" => {
            let doc = serde_json::from_str(&doc)?;
            Ok(doc)
        }
        _ => bail!("File Extension not supported"),
    } 
}

///
pub fn save_doc(file_name: &str, value: &serde_json::Value) -> anyhow::Result<()> {
    let mut path = PathBuf::from(file_name);

    let stem = path
        .file_stem()
        .ok_or_else(|| anyhow!("File Stem not found"))?
        .to_string_lossy();

    let extention = path
        .extension()
        .ok_or_else(|| anyhow!("File Extension not found"))?
        .to_string_lossy();

    let data = match extention.as_ref() {
        "yaml" | "yml" => serde_yaml::to_string(&value)?,
        "json" => serde_json::to_string_pretty(&value)?,
        _ => bail!("File Extension not supported"),
    };

    path.set_file_name(format!("new_{stem}.{extention}"));

    let mut save_file = File::create(&path)?;
    save_file.write_all(data.as_bytes())?;

    Ok(())
}

///
pub fn editor(value: &serde_json::Value, file_path: &str) -> anyhow::Result<serde_json::Value> {
    let editor = env::var("EDITOR")?;
    let mut temp_file_path = env::temp_dir();

    let file_name = PathBuf::from(file_path);
    let file_name = file_name
        .file_name()
        .ok_or_else(|| anyhow!("File Name not found"))?;

    temp_file_path.push(file_name);

    let extension = PathBuf::from(file_path);
    let extension = extension
        .extension()
        .ok_or_else(|| anyhow!("File Extension not found"))?
        .to_string_lossy();

    let mut new_file = File::create(&temp_file_path)?;

    match extension.as_ref() {
        "yaml" | "yml" => {
            new_file.write_all(serde_yaml::to_string(&value)?.as_bytes())?;
        }
        "json" => {
            new_file.write_all(serde_json::to_string_pretty(&value)?.as_bytes())?;
        }
        _ => bail!("File Extension not supported"),
    }

    Command::new(editor).arg(&temp_file_path).status()?;

    let new_value = fs::read_to_string(&temp_file_path)?;

    match extension.as_ref() {
        "yaml" | "yml" => {
            let result = serde_yaml::from_str(&new_value)?;
            Ok(result)
        }
        "json" => {
            let result = serde_json::from_str(&new_value)?;
            Ok(result)
        }
        _ => bail!("File Extension not supported"),
    }
}

