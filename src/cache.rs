use std::{
    fs::{self, File},
    path::PathBuf,
};

use anyhow::{anyhow, Result};
use directories::ProjectDirs;

fn path() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("", "", env!("CARGO_CRATE_NAME"))
        .ok_or(anyhow!("Unable to get ProjectDirs"))?;
    let mut path = project_dirs.cache_dir().to_path_buf();
    fs::create_dir_all(&path)?;

    path.push("cache.json");

    Ok(path)
}

pub(crate) fn get_selected_devices() -> Result<Vec<String>> {
    let Ok(file) = File::open(path()?) else {
        return Ok(vec![]);
    };
    let devices = serde_json::from_reader(file)?;
    Ok(devices)
}

pub(crate) fn set_selected_devices(devices: Vec<String>) -> Result<()> {
    serde_json::to_writer(File::create(path()?)?, &devices)?;

    Ok(())
}
