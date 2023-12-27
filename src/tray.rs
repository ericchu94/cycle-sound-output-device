use anyhow::Result;

use tray_icon::{
    menu::{CheckMenuItem, Menu, MenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

pub(crate) struct TrayManager {
    tray_icon: TrayIcon,
    menu: Menu,
}

impl TrayManager {
    pub(crate) fn new() -> Result<Self> {
        let menu = Menu::new();
        menu.append(&MenuItem::with_id("exit", "Exit", true, None))?;
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip("Cycle Sound Output Device")
            .build()?;

        Ok(Self {
            tray_icon,
            menu: menu,
        })
    }

    pub(crate) fn set_icon(&self, icon: Icon) -> Result<()> {
        self.tray_icon.set_icon(Some(icon))?;

        Ok(())
    }

    pub(crate) fn update_check_menu(&self, devices: Vec<(String, String, bool)>) -> Result<()> {
        for _ in 1..self.menu.items().len() {
            self.menu.remove_at(0);
        }
        for (id, text, checked) in devices {
            self.menu
                .prepend(&CheckMenuItem::with_id(id, text, true, checked, None))?;
        }

        Ok(())
    }
}
