use std::io::Cursor;

use anyhow::{anyhow, Result};
use ico::IconDir;
use tray_icon::{TrayIcon, TrayIconBuilder};

use crate::OutputDevice;

pub(crate) struct TrayManager {
    tray_icon: TrayIcon,
}

impl TrayManager {
    pub(crate) fn new() -> Result<Self> {
        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("Toggle Output Device")
            .with_icon(get_icon(OutputDevice::Unknown)?)
            .build()?;

        Ok(Self { tray_icon })
    }

    pub(crate) fn set_icon(&self, output_device: OutputDevice) -> Result<()> {
        self.tray_icon.set_icon(Some(get_icon(output_device)?))?;

        Ok(())
    }
}

fn get_icon(device: OutputDevice) -> Result<tray_icon::Icon> {
    let bytes = device.icon_bytes();
    let cursor = Cursor::new(bytes);

    let icon_dir = IconDir::read(cursor)?;

    let size = 64;

    let entry = icon_dir
        .entries()
        .into_iter()
        .filter(|entry| entry.width() == size)
        .next()
        .ok_or(anyhow!("Entry of size {size}x{size} not found"))?;

    let image = entry.decode()?;

    let data = image.rgba_data();

    Ok(tray_icon::Icon::from_rgba(data.to_vec(), size, size)?)
}
