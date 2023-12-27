#![windows_subsystem = "windows"]

mod audio;
mod cache;
mod tray;

use std::{collections::HashSet, process};

use anyhow::Result;
use tao::{
    event::Event,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
};
use tray_icon::{menu::MenuEvent, ClickType, Icon, TrayIconEvent};

use crate::{audio::AudioInterface, tray::TrayManager};

struct Handler {
    tray_manager: TrayManager,
    audio_interface: AudioInterface,
}

impl Handler {
    fn new() -> Result<Self> {
        let audio_interface = AudioInterface::new()?;

        let tray_manager = TrayManager::new()?;

        let handler = Self {
            tray_manager,
            audio_interface,
        };

        handler.update()?;

        Ok(handler)
    }

    fn update_icon(&self) -> Result<()> {
        let cur = self.audio_interface.get_default_output_device()?;

        self.tray_manager
            .set_icon(Icon::from_handle(cur.icon_handle()?.0))?;

        Ok(())
    }

    fn next_device(&mut self) -> Result<()> {
        let selected_devices = cache::get_selected_devices()?;
        let all_devices = self
            .audio_interface
            .get_output_devices()?
            .into_iter()
            .map(|d| d.id())
            .collect::<Result<HashSet<String>>>()?;
        let devices = selected_devices
            .into_iter()
            .filter(|d| all_devices.contains(d))
            .collect::<Vec<String>>();

        if devices.len() > 0 {
            let cur = self.audio_interface.get_default_output_device()?.id()?;

            let index: usize = devices.iter().position(|x| x == &cur).unwrap_or(0);
            let next = (index + 1) % devices.len();

            self.audio_interface
                .set_default_output_device(&devices[next])?;

            self.update()?;
        }

        Ok(())
    }

    fn update_tray_menu(&self) -> Result<()> {
        let devices = self.audio_interface.get_output_devices()?;
        let selected_devices = cache::get_selected_devices()?;

        let devices = devices
            .into_iter()
            .map(|d| {
                let id = d.id()?;
                let name = d.device_friendly_name()?;
                let selected = selected_devices.contains(&id);
                Ok((id, name, selected))
            })
            .collect::<Result<Vec<(String, String, bool)>>>()?;

        self.tray_manager.update_check_menu(devices)?;

        Ok(())
    }

    fn update(&self) -> Result<()> {
        self.update_icon()?;
        self.update_tray_menu()?;

        Ok(())
    }

    fn device_clicked(&self, id: String) -> Result<()> {
        let mut selected = cache::get_selected_devices()?;
        if let Some(index) = selected.iter().position(|x| x == &id) {
            selected.remove(index);
        } else {
            selected.push(id);
        }
        cache::set_selected_devices(selected)?;

        self.update()
    }

    fn handle(&mut self, event: TrayMenuEvent) -> Result<()> {
        println!("{event:?}");
        match event {
            TrayMenuEvent::TrayIconEvent(event) => {
                if event.click_type == ClickType::Left {
                    self.next_device().expect("next device failed");
                }
            }
            TrayMenuEvent::MenuEvent(event) => {
                if event.id.0 == "exit" {
                    process::exit(0);
                }

                self.device_clicked(event.id.0)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
enum TrayMenuEvent {
    TrayIconEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
}

fn main() -> Result<()> {
    let event_loop: EventLoop<TrayMenuEvent> = EventLoopBuilder::with_user_event().build();

    let proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |e: TrayIconEvent| {
        proxy
            .send_event(TrayMenuEvent::TrayIconEvent(e))
            .expect("send event failed");
    }));

    let proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
        proxy
            .send_event(TrayMenuEvent::MenuEvent(e))
            .expect("send event failed");
    }));

    let mut handler = Handler::new()?;

    event_loop.run(
        move |event, _, control_flow: &mut tao::event_loop::ControlFlow| {
            // println!("{event:?}, {:?}", std::thread::current().name());
            if let Event::UserEvent(event) = event {
                handler.handle(event).expect("handle failed");
            }
            *control_flow = ControlFlow::Wait;
        },
    );
}
