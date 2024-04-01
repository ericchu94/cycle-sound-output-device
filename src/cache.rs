use std::{
    fs::{self, File},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
struct Cache {
    output_devices: Vec<String>,
    input_devices: Vec<(String, u8)>,
}

fn path() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("", "", env!("CARGO_CRATE_NAME"))
        .ok_or(anyhow!("Unable to get ProjectDirs"))?;
    let mut path = project_dirs.cache_dir().to_path_buf();
    fs::create_dir_all(&path)?;

    path.push("cache.json");

    Ok(path)
}

fn read_cache() -> Result<Cache> {
    let file = File::open(path()?)?;
    let cache = serde_json::from_reader(file)?;

    Ok(cache)
}

fn write_cache(cache: &Cache) -> Result<()> {
    serde_json::to_writer(File::create(path()?)?, cache)?;
    Ok(())
}

fn cache() -> Cache {
    read_cache().unwrap_or_default()
}

pub(crate) fn get_selected_output_devices() -> Result<Vec<String>> {
    Ok(cache().output_devices)
}

pub(crate) fn set_selected_output_devices(devices: Vec<String>) -> Result<()> {
    let mut cache = cache();
    cache.output_devices = devices;

    write_cache(&cache)?;

    Ok(())
}

pub(crate) fn get_selected_input_devices() -> Result<Vec<(String, u8)>> {
    Ok(cache().input_devices)
}

pub(crate) fn set_selected_input_devices(devices: Vec<(String, u8)>) -> Result<()> {
    let mut cache = cache();
    cache.input_devices = devices;

    write_cache(&cache)?;

    Ok(())
}
