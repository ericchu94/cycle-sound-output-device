use anyhow::Result;

use tray_icon::{
    menu::{CheckMenuItem, Menu, MenuItem, Submenu},
    Icon, TrayIcon, TrayIconBuilder,
};

pub(crate) struct TrayManager {
    tray_icon: TrayIcon,
    input_devices: Submenu,
    output_devices: Submenu,
}

impl TrayManager {
    pub(crate) fn new() -> Result<Self> {
        let menu = Menu::new();

        let input_devices = Submenu::new("Input Devices", true);
        menu.append(&input_devices)?;

        let output_devices = Submenu::new("Output Devices", true);
        menu.append(&output_devices)?;

        menu.append(&MenuItem::with_id("exit", "Exit", true, None))?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip("Cycle Sound Output Device")
            .build()?;

        Ok(Self {
            tray_icon,
            input_devices,
            output_devices,
        })
    }

    pub(crate) fn set_icon(&self, icon: Icon) -> Result<()> {
        println!("Entering");
        self.tray_icon.set_icon(Some(icon))?;
        println!("Exit");

        Ok(())
    }

    pub(crate) fn update_output_devices(&self, devices: Vec<(String, String, bool)>) -> Result<()> {
        while !self.output_devices.items().is_empty() {
            self.output_devices.remove_at(0);
        }
        for (id, text, checked) in devices {
            self.output_devices
                .prepend(&CheckMenuItem::with_id(id, text, true, checked, None))?;
        }

        Ok(())
    }

    pub(crate) fn update_input_devices(&self, devices: Vec<(String, String, bool)>) -> Result<()> {
        while !self.input_devices.items().is_empty() {
            self.input_devices.remove_at(0);
        }
        for (id, text, checked) in devices {
            self.input_devices
                .prepend(&CheckMenuItem::with_id(id, text, true, checked, None))?;
        }

        Ok(())
    }
}
