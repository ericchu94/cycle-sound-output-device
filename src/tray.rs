use anyhow::Result;

use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub(crate) struct TrayManager {
    tray_icon: TrayIcon,
}

impl TrayManager {
    pub(crate) fn new() -> Result<Self> {
        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("Toggle Output Device")
            .build()?;

        Ok(Self { tray_icon })
    }

    pub(crate) fn set_icon(&self, icon: Icon) -> Result<()> {
        self.tray_icon.set_icon(Some(icon))?;

        Ok(())
    }
}
